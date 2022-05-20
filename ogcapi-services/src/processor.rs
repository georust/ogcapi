use std::collections::HashMap;

use anyhow::Context;
use axum::{
    async_trait,
    response::{IntoResponse, Response},
};

use hyper::StatusCode;
use ogcapi_drivers::s3::{ByteStream, S3};
use ogcapi_types::{
    common::Crs,
    features::Feature,
    processes::{
        Execute, InlineOrRefData, Input, InputValue, InputValueNoObject, Process,
        QualifiedInputValue,
    },
    stac::Asset,
};

use crate::{Error, Result, State};

#[async_trait]
/// Trait for defining and executing a [Process]
pub trait Processor: Send + Sync {
    /// Returns the process id (must be unique)
    fn id(&self) -> String;

    /// Returns the Process description
    fn process(&self) -> Process;

    /// Executes the Process and returns a response
    async fn execute(&self, execute: &Execute, state: &State) -> Result<Response>;
}

/// Example Processor
///
/// ```bash
/// curl -k https://localhost:8484/processes/greet/execution \
///         -u 'user:password'
///         -H 'Content-Type: application/json' \
///         -d '{"inputs": { "name": "World" } }'
/// ```
pub(crate) struct Greeter;

#[async_trait]
impl Processor for Greeter {
    fn id(&self) -> String {
        "greet".to_string()
    }
    fn process(&self) -> Process {
        Process::new(
            self.id(),
            "0.1.0",
            &serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string"
                    }
                }
            }),
            &serde_json::json!({ "type": "string" }),
        )
    }

    async fn execute(&self, execute: &Execute, _state: &State) -> Result<Response> {
        tracing::debug!("{:#?}", execute);

        let input = execute.inputs.get("name").unwrap();
        match input {
            Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                InputValueNoObject::String(name),
            )) => Ok(format!("Hello, {}!", name).into_response()),
            _ => todo!(),
        }
    }
}

/// STAC Asset loader
pub(crate) struct AssetLoader;

#[async_trait]
impl Processor for AssetLoader {
    fn id(&self) -> String {
        "load-asset".to_string()
    }
    fn process(&self) -> Process {
        Process::new(
            self.id(),
            "0.1.0",
            &serde_json::json!({
                "type": "object",
                "properties": {
                    "file": {
                        "type": "object",
                        "properties": {
                            "value": {
                                "type": "string"
                            },
                            "encoding": {
                                "type": "string"
                            },
                            "mediaType": {
                                "type": "string"
                            },
                        },
                    },
                    "key": {
                        "type": "string"
                    },
                    "title": {
                        "description": "The displayed title for clients and users.",
                        "type": "string",
                    },
                    "description": {
                        "description": "A description of the Asset providing additional details, such as how it was processed or created.",
                        "type": "string",
                    },
                    "roles": {
                        "description": "The semantic roles of the asset.",
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                    },
                    "collection": {
                        "description": "Collection `id`",
                        "type": "string",
                    },
                    "item": {
                        "description": "Item object to create or existing Item `id`",
                        "oneOf": [
                            { "type": "string" },
                            { "type": "object" },
                        ]
                    },
                },
                "required": [
                    "file",
                    "key",
                    "type",
                    "collection",
                    "item",
                ],
            }),
            &serde_json::json!({
                "description": "URI of the crated/updated Item.",
                "type": "string"
            }),
        )
    }

    async fn execute(&self, execute: &Execute, state: &State) -> Result<Response> {
        // tracing::debug!("{:#?}", execute);
        let (data, meta) = match execute.inputs.get("file") {
            Some(Input::InlineOrRefData(InlineOrRefData::QualifiedInputValue(
                QualifiedInputValue {
                    value: InputValue::InputValueNoObject(InputValueNoObject::String(data)),
                    format,
                },
            ))) => (data, format),
            _ => {
                return Err(Error::Exception(
                    StatusCode::BAD_REQUEST,
                    "Missing required `file` property".to_string(),
                ))
            }
        };

        if meta.media_type.is_none() {
            return Err(Error::Exception(
                StatusCode::BAD_REQUEST,
                "Missing required `mediaType` property".to_string(),
            ));
        }

        let key = match execute.inputs.get("key") {
            Some(Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                InputValueNoObject::String(key),
            ))) => key,
            _ => todo!(),
        };

        let collection = match execute.inputs.get("collection") {
            Some(Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                InputValueNoObject::String(collection),
            ))) => collection,
            _ => todo!(),
        };

        // Setup S3 driver
        let s3 = S3::setup().await;

        // Upload asset
        let bytes = base64::decode(data).expect("decode base64 string");
        s3.client
            .put_object()
            .bucket("test-bucket")
            .key(format!("assets/{key}"))
            .body(ByteStream::from(bytes))
            .content_type(meta.media_type.as_ref().unwrap())
            .send()
            .await
            .context("Failed to put object to S3")?;

        let asset = Asset::new(key);

        let asset_map = HashMap::from([(key.to_string(), asset)]);

        let id = match execute.inputs.get("item").unwrap() {
            Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                InputValueNoObject::String(id),
            )) => {
                let mut item = state
                    .drivers
                    .features
                    .read_feature(collection, id, &Crs::default())
                    .await?;

                for (k, v) in asset_map.into_iter() {
                    item.assets.insert(k, v);
                }

                state.drivers.features.update_feature(&item).await?;

                id.to_owned()
            }
            Input::InlineOrRefData(InlineOrRefData::QualifiedInputValue(QualifiedInputValue {
                value: InputValue::Object(object),
                format: _,
            })) => {
                let mut item: Feature = serde_json::from_value(object.to_owned().into()).unwrap();
                item.assets = asset_map;

                state.drivers.features.create_feature(&item).await?
            }
            _ => unreachable!(),
        };

        Ok(format!("../../collections/{}/items/{}", collection, id).into_response())
    }
}

impl AssetLoader {
    async fn load_asset_from_path(
        &self,
        path: &std::path::PathBuf,
        media_type: &str,
    ) -> anyhow::Result<std::collections::HashMap<String, ogcapi_types::stac::Asset>> {
        // Setup S3 driver
        let s3 = S3::setup().await;

        let stream = ByteStream::from_path(&path).await?;

        // Upload asset
        let filename = path.file_name().unwrap().to_str().unwrap();

        let key = format!("assets/{filename}");

        s3.client
            .put_object()
            .bucket("test-bucket")
            .key(&key)
            .body(stream)
            .content_type(media_type)
            .send()
            .await?;

        let asset = ogcapi_types::stac::Asset::new(key);

        let file_stem = path.file_stem().unwrap().to_str().unwrap();

        Ok(std::collections::HashMap::from([(
            file_stem.to_string(),
            asset,
        )]))
    }
}
