use std::{collections::HashMap, io::Cursor};

use anyhow::Result;
use geo::Geometry;
use geojson::FeatureCollection;
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use sqlx::types::Json;
use url::Url;
use wkb::{Endianness, writer::WriteOptions};

use ogcapi_drivers::{CollectionTransactions, postgres::Db};
use ogcapi_types::{
    common::{Collection, Crs, Extent, SpatialExtent},
    processes::{
        Execute, Format, InlineOrRefData, Input, InputValueNoObject, Output, Process,
        TransmissionMode,
    },
};

use crate::{ProcessResponseBody, Processor};

/// GeoJson loader `Processor`
///
/// Process to load vector data.
#[derive(Clone)]
pub struct GeoJsonLoader;

/// Inputs for the `geojson-loader` process
#[derive(Deserialize, Debug, JsonSchema)]
pub struct GeoJsonLoaderInputs {
    /// Input file
    pub input: String,

    /// Set the collection name
    pub collection: String,

    /// Source srs, if omitted tries to derive from the input layer
    pub s_srs: Option<u32>,

    /// Postgres database url
    pub database_url: String,
}

impl GeoJsonLoaderInputs {
    pub fn execute_input(&self) -> HashMap<String, Input> {
        let mut input = HashMap::from_iter([
            (
                "input".to_string(),
                Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                    InputValueNoObject::String(self.input.to_owned()),
                )),
            ),
            (
                "collection".to_string(),
                Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                    InputValueNoObject::String(self.collection.to_owned()),
                )),
            ),
            (
                "database_url".to_string(),
                Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                    InputValueNoObject::String(self.database_url.to_owned()),
                )),
            ),
        ]);

        if let Some(s_srs) = &self.s_srs {
            input.insert(
                "s_srs".to_owned(),
                Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                    InputValueNoObject::Integer(*s_srs as i64),
                )),
            );
        }

        input
    }
}

/// Outputs for the `gdal-loader` process
#[derive(Clone, Debug, JsonSchema)]
pub struct GeoJsonLoaderOutputs {
    pub collection_id: String,
}

impl GeoJsonLoaderOutputs {
    pub fn execute_output() -> HashMap<String, Output> {
        HashMap::from([(
            "greeting".to_string(),
            Output {
                format: Some(Format {
                    media_type: Some("text/plain".to_string()),
                    encoding: Some("utf8".to_string()),
                    schema: None,
                }),
                transmission_mode: TransmissionMode::Value,
            },
        )])
    }
}

#[async_trait::async_trait]
impl Processor for GeoJsonLoader {
    fn id(&self) -> &'static str {
        "geojson-loader"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn process(&self) -> Result<Process> {
        Process::try_new(
            self.id(),
            self.version(),
            &schema_for!(GeoJsonLoaderInputs),
            &schema_for!(GeoJsonLoaderOutputs),
        )
        .map_err(Into::into)
    }

    async fn execute(&self, execute: Execute) -> Result<ProcessResponseBody> {
        let value = serde_json::to_value(execute.inputs)?;
        let inputs: GeoJsonLoaderInputs = serde_json::from_value(value)?;

        // Setup driver
        let db = Db::setup(&Url::parse(&inputs.database_url).unwrap()).await?;

        // Extract data
        let geojson_str = std::fs::read_to_string(&inputs.input)?;
        let geojson = geojson_str.parse::<FeatureCollection>()?;

        // Create collection
        let collection = Collection {
            id: inputs.collection.to_owned(),
            item_type: Some("Feature".to_string()),
            extent: geojson
                .bbox
                .map(|bbox| Extent {
                    spatial: SpatialExtent {
                        bbox: vec![
                            bbox.as_slice()
                                .try_into()
                                .unwrap_or_else(|_| [-180.0, -90.0, 180.0, 90.0].into()),
                        ],
                        crs: Crs::default(),
                    },
                    ..Default::default()
                })
                .or_else(|| Some(Extent::default())),
            crs: vec![Crs::default(), Crs::from_epsg(3857), Crs::from_epsg(2056)],
            storage_crs: Some(Crs::default()),
            // #[cfg(feature = "stac")]
            // assets: crate::asset::load_asset_from_path(&args.input).await?,
            ..Default::default()
        };

        db.delete_collection(&collection.id).await?;
        db.create_collection(&collection).await?;

        // Load features
        let chunk_size = 1000;
        let chunks: Vec<_> = geojson
            .features
            .chunks(chunk_size)
            .enumerate()
            .map(|(i, chunk)| {
                let mut ids = Vec::with_capacity(chunk_size);
                let mut properties = Vec::with_capacity(chunk_size);
                let mut geoms = Vec::with_capacity(chunk_size);

                for (ii, feature) in chunk.iter().enumerate() {
                    // id
                    let id = if let Some(id) = &feature.id {
                        match id {
                            geojson::feature::Id::String(s) => s.to_owned(),
                            geojson::feature::Id::Number(n) => n.to_string(),
                        }
                    } else {
                        ((i * chunk_size) + ii).to_string()
                    };
                    ids.push(id);

                    // properties
                    let props = feature.properties.to_owned().map(Json);
                    properties.push(props);

                    // geometry
                    let geom =
                        Geometry::try_from(feature.geometry.to_owned().unwrap().value).unwrap();

                    let mut wkb = Cursor::new(Vec::new());
                    wkb::writer::write_geometry(
                        &mut wkb,
                        &geom,
                        &WriteOptions {
                            endianness: Endianness::LittleEndian,
                        },
                    )
                    .unwrap();
                    geoms.push(wkb.into_inner());
                }

                (ids, properties, geoms)
            })
            .collect();

        for (ids, properties, geoms) in chunks {
            sqlx::query(&format!(
                r#"
            INSERT INTO items."{}" (id, properties, geom)
            SELECT * FROM UNNEST($1::text[], $2::jsonb[], $3::bytea[])
            "#,
                collection.id
            ))
            .bind(ids)
            .bind(properties)
            .bind(geoms)
            .execute(&db.pool)
            .await?;
        }
        Ok(ProcessResponseBody::Requested {
            outputs: GeoJsonLoaderOutputs::execute_output(),
            parts: vec![inputs.collection.as_bytes().to_owned()],
        })
    }
}

#[cfg(test)]
mod tests {
    use ogcapi_types::processes::Execute;

    use crate::Processor;

    use super::*;

    #[tokio::test]
    async fn test_loader() {
        let loader = GeoJsonLoader;
        assert_eq!(loader.id(), "geojson-loader");

        println!(
            "Process:\n{}",
            serde_json::to_string_pretty(&loader.process().unwrap()).unwrap()
        );

        let input = GeoJsonLoaderInputs {
            input: "../data/ne_10m_railroads_north_america.geojson".to_owned(),
            collection: "streets-geojson".to_string(),
            s_srs: None,
            database_url: "postgresql://postgres:password@localhost:5433/ogcapi".to_string(),
        };

        let execute = Execute {
            inputs: input.execute_input(),
            ..Default::default()
        };

        let output = loader.execute(execute).await.unwrap();

        let ProcessResponseBody::Requested {
            outputs: _outputs,
            parts,
        } = output
        else {
            panic!()
        };

        assert_eq!(
            String::from_utf8(parts[0].clone()).unwrap(),
            "streets-geojson"
        );
    }
}
