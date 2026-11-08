use std::collections::HashMap;

use anyhow::Result;
use geojson::FeatureCollection;
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use sqlx::types::Json;
use url::Url;

use ogcapi_drivers::{CollectionTransactions, postgres::Db};
use ogcapi_types::{
    common::{Bbox, Collection, Crs, Extent, SpatialExtent},
    processes::{
        Execute, ExecuteResult, ExecuteResults, Format, InlineOrRefData, Input, InputValueNoObject,
        JobControlOptions, Output, Process, ProcessSummary, TransmissionMode,
        description::{DescriptionType, InputDescription, OutputDescription},
    },
};

use crate::Processor;

/// `GeoJSON` loader `Processor`
///
/// Process to load a `GeoJSON` feature collection, in CRS84, into a
/// collection of a Postgres database, which it replaces.
#[derive(Clone)]
pub struct GeoJsonLoader;

/// Inputs for the `geojson-loader` process
#[derive(Deserialize, Debug, JsonSchema)]
pub struct GeoJsonLoaderInputs {
    /// Input file
    pub input: String,

    /// Set the collection name
    pub collection: String,

    /// Postgres database url
    pub database_url: String,
}

impl GeoJsonLoaderInputs {
    #[must_use]
    pub fn execute_input(&self) -> HashMap<String, Input> {
        let text = |value: &str| {
            Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                InputValueNoObject::String(value.to_owned()),
            ))
        };
        HashMap::from([
            ("input".to_string(), text(&self.input)),
            ("collection".to_string(), text(&self.collection)),
            ("database_url".to_string(), text(&self.database_url)),
        ])
    }
}

/// Outputs for the `geojson-loader` process
#[derive(Clone, Debug, JsonSchema)]
pub struct GeoJsonLoaderOutputs {
    /// The id of the collection loaded
    pub collection: String,
}

impl TryFrom<GeoJsonLoaderOutputs> for ExecuteResults {
    type Error = anyhow::Error;

    fn try_from(value: GeoJsonLoaderOutputs) -> Result<Self, Self::Error> {
        Ok(HashMap::from([(
            "collection".to_string(),
            ExecuteResult {
                data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                    value.collection,
                )),
                output: Output {
                    format: Some(Format {
                        media_type: Some("text/plain".to_string()),
                        encoding: Some("utf8".to_string()),
                        schema: None,
                    }),
                    transmission_mode: TransmissionMode::Value,
                },
            },
        )]))
    }
}

#[async_trait::async_trait]
impl Processor for GeoJsonLoader {
    type Input = GeoJsonLoaderInputs;
    type Output = GeoJsonLoaderOutputs;

    fn id(&self) -> &'static str {
        "geojson-loader"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    async fn process(&self) -> Result<Process> {
        let input = |title: &str| InputDescription {
            description_type: DescriptionType {
                title: Some(title.to_string()),
                ..Default::default()
            },
            schema: schema_for!(String).to_value(),
            ..Default::default()
        };
        Ok(Process {
            summary: ProcessSummary {
                id: self.id().to_string(),
                version: self.version().to_string(),
                description: DescriptionType {
                    title: Some("GeoJSON loader".to_string()),
                    description: Some(
                        "Loads a GeoJSON feature collection into a collection, which it replaces."
                            .to_string(),
                    ),
                    ..Default::default()
                },
                job_control_options: vec![
                    JobControlOptions::SyncExecute,
                    JobControlOptions::AsyncExecute,
                ],
                output_transmission: vec![TransmissionMode::Value],
                links: Vec::new(),
            },
            inputs: HashMap::from([
                ("input".to_string(), input("Input file")),
                ("collection".to_string(), input("Collection id")),
                ("database_url".to_string(), input("PostgreSQL database URL")),
            ]),
            outputs: HashMap::from([(
                "collection".to_string(),
                OutputDescription {
                    description_type: DescriptionType::default(),
                    schema: schema_for!(String).to_value(),
                },
            )]),
        })
    }

    async fn parse(&self, execute: Execute) -> Result<Self::Input> {
        let value = serde_json::to_value(execute.inputs)?;
        Ok(serde_json::from_value(value)?)
    }

    async fn execute(&self, inputs: Self::Input) -> Result<Self::Output> {
        // Setup driver
        let db = Db::setup(&Url::parse(&inputs.database_url)?).await?;

        // Extract data
        let geojson_str = std::fs::read_to_string(&inputs.input)?;
        let geojson = geojson_str.parse::<FeatureCollection>()?;

        // Create collection (replace an existing one)
        let mut collection = Collection::new(inputs.collection.clone());
        collection.crs = vec![Crs::default2d(), Crs::from_epsg(3857)];
        collection.storage_crs = Some(Crs::default2d());
        collection.extent = geojson
            .bbox
            .as_deref()
            .and_then(|bbox| Bbox::try_from(bbox).ok())
            .map(|bbox| Extent {
                spatial: Some(SpatialExtent {
                    bbox: vec![bbox],
                    crs: Some(Crs::default2d()),
                }),
                temporal: None,
            });

        if db.read_collection(&collection.id).await?.is_some() {
            db.delete_collection(&collection.id).await?;
        }
        // the driver refuses an id that can't be a table's, so the table
        // below is named safely
        db.create_collection(&collection).await?;

        // Load features, a thousand at a time
        let chunk_size = 1000;
        for (i, chunk) in geojson.features.chunks(chunk_size).enumerate() {
            let mut ids = Vec::with_capacity(chunk.len());
            let mut properties = Vec::with_capacity(chunk.len());
            let mut geometries = Vec::with_capacity(chunk.len());

            for (ii, feature) in chunk.iter().enumerate() {
                ids.push(match &feature.id {
                    Some(geojson::feature::Id::String(s)) => s.to_owned(),
                    Some(geojson::feature::Id::Number(n)) => n.to_string(),
                    None => (i * chunk_size + ii).to_string(),
                });
                properties.push(feature.properties.clone().map(Json));
                // a feature without a geometry keeps none
                geometries.push(feature.geometry.clone().map(Json));
            }

            sqlx::query(&format!(
                r#"
                INSERT INTO items."{}" (id, properties, geom)
                SELECT id, properties, ST_SetSRID(ST_GeomFromGeoJSON(geometry), 4326)
                FROM UNNEST($1::text[], $2::jsonb[], $3::jsonb[]) AS t(id, properties, geometry)
                "#,
                collection.id
            ))
            .bind(ids)
            .bind(properties)
            .bind(geometries)
            .execute(&db.pool)
            .await?;
        }

        Ok(GeoJsonLoaderOutputs {
            collection: inputs.collection,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Processor;

    use super::*;

    #[tokio::test]
    #[ignore = "needs a PostgreSQL database"]
    async fn test_loader() {
        let loader = GeoJsonLoader;
        assert_eq!(loader.id(), "geojson-loader");

        let input = GeoJsonLoaderInputs {
            input: "../data/ne_10m_railroads_north_america.geojson".to_owned(),
            collection: "streets-geojson".to_string(),
            database_url: "postgresql://postgres:password@localhost:5432/ogcapi".to_string(),
        };

        let execute = Execute {
            inputs: input.execute_input(),
            ..Default::default()
        };

        let output = loader
            .execute(loader.parse(execute).await.unwrap())
            .await
            .unwrap();

        assert_eq!(output.collection, "streets-geojson");
    }
}
