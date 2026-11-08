use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result, bail};
use arrow::{
    array::{Array, BinaryArray, RecordBatchReader, StringArray},
    compute::cast,
    datatypes::DataType,
    ffi_stream::{ArrowArrayStreamReader, FFI_ArrowArrayStream},
    json::ArrayWriter,
};
use gdal::{
    ArrowArrayStream, Dataset, cpl::CslStringList, spatial_ref::SpatialRef, vector::LayerAccess,
};
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
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

/// GDAL loader `Processor`
///
/// Process to load a layer of vector data GDAL reads into a collection of a
/// Postgres database, which it replaces.
#[derive(Clone)]
pub struct GdalLoader;

/// Inputs for the `gdal-loader` process
#[derive(Deserialize, Debug, JsonSchema)]
pub struct GdalLoaderInputs {
    /// Input file
    pub input: String,

    /// Set the collection name
    pub collection: String,

    /// Filter input by layer name, required where it has several
    pub filter: Option<String>,

    /// Source srs, if omitted tries to derive from the input layer
    pub s_srs: Option<u32>,

    /// Postgres database url
    pub database_url: String,
}

impl GdalLoaderInputs {
    #[must_use]
    pub fn execute_input(&self) -> HashMap<String, Input> {
        let value = |value: InputValueNoObject| {
            Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(value))
        };
        let text = |text: &str| value(InputValueNoObject::String(text.to_owned()));
        let mut input = HashMap::from([
            ("input".to_string(), text(&self.input)),
            ("collection".to_string(), text(&self.collection)),
            ("database_url".to_string(), text(&self.database_url)),
        ]);
        if let Some(filter) = &self.filter {
            input.insert("filter".to_owned(), text(filter));
        }
        if let Some(s_srs) = self.s_srs {
            input.insert(
                "s_srs".to_owned(),
                value(InputValueNoObject::Integer(i64::from(s_srs))),
            );
        }
        input
    }
}

/// Outputs for the `gdal-loader` process
#[derive(Clone, Debug, JsonSchema)]
pub struct GdalLoaderOutputs {
    /// The id of the collection loaded
    pub collection: String,
}

impl TryFrom<GdalLoaderOutputs> for ExecuteResults {
    type Error = anyhow::Error;

    fn try_from(value: GdalLoaderOutputs) -> Result<Self, Self::Error> {
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
impl Processor for GdalLoader {
    type Input = GdalLoaderInputs;
    type Output = GdalLoaderOutputs;

    fn id(&self) -> &'static str {
        "gdal-loader"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    async fn process(&self) -> Result<Process> {
        let input = |title: &str, schema: serde_json::Value| InputDescription {
            description_type: DescriptionType {
                title: Some(title.to_string()),
                ..Default::default()
            },
            schema,
            ..Default::default()
        };
        let text = || schema_for!(String).to_value();
        Ok(Process {
            summary: ProcessSummary {
                id: self.id().to_string(),
                version: self.version().to_string(),
                description: DescriptionType {
                    title: Some("GDAL loader".to_string()),
                    description: Some(
                        "Loads a layer of vector data GDAL reads into a collection, which it \
                        replaces."
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
                ("input".to_string(), input("Input file or URL", text())),
                ("collection".to_string(), input("Collection id", text())),
                ("filter".to_string(), input("Layer name", text())),
                (
                    "s_srs".to_string(),
                    input("Source EPSG code", schema_for!(u32).to_value()),
                ),
                (
                    "database_url".to_string(),
                    input("PostgreSQL database URL", text()),
                ),
            ]),
            outputs: HashMap::from([(
                "collection".to_string(),
                OutputDescription {
                    description_type: DescriptionType::default(),
                    schema: text(),
                },
            )]),
        })
    }

    async fn parse(&self, execute: Execute) -> Result<Self::Input> {
        let value = serde_json::to_value(execute.inputs)?;
        let mut inputs: GdalLoaderInputs = serde_json::from_value(value)?;

        // Handle http & zip
        if inputs.input.to_lowercase().starts_with("http") {
            inputs.input = format!("/vsicurl/{}", inputs.input);
        }
        if inputs.input.to_lowercase().ends_with("zip") {
            inputs.input = format!("/vsizip/{}", inputs.input);
        }
        Ok(inputs)
    }

    #[allow(clippy::too_many_lines, reason = "Complex loading logic")]
    async fn execute(&self, inputs: Self::Input) -> Result<Self::Output> {
        // GDAL's dataset can't be sent between threads, so it is opened in a
        // block that awaits nothing, and again for the features
        let (layer_name, collection, geometry_type) = {
            let dataset = Dataset::open(&inputs.input)?;

            let layer_name = match &inputs.filter {
                Some(filter) => filter.clone(),
                None if dataset.layer_count() == 1 => dataset.layer(0)?.name(),
                None => bail!(
                    "Found multiple layers! Use the 'filter' option to specify one of:\n\t- {}",
                    dataset
                        .layers()
                        .map(|l| l.name())
                        .collect::<Vec<String>>()
                        .join("\n\t- ")
                ),
            };
            let layer = dataset.layer_by_name(&layer_name)?;

            // Get coordinate reference system
            let spatial_ref_src = if let Some(epsg) = inputs.s_srs {
                SpatialRef::from_epsg(epsg)?
            } else if let Some(srs) = layer.spatial_ref() {
                srs
            } else {
                tracing::warn!("Unknown spatial reference, falling back to `4326`");
                SpatialRef::from_epsg(4326)?
            };
            let storage_crs = Crs::from_srid(spatial_ref_src.auth_code()?);

            let mut collection = Collection::new(inputs.collection.clone());
            collection.crs = Vec::from_iter(HashSet::from([
                Crs::default2d(),
                storage_crs.clone(),
                Crs::from_epsg(3857),
            ]));
            collection.extent = layer.try_get_extent()?.map(|e| Extent {
                spatial: Some(SpatialExtent {
                    bbox: vec![Bbox::Bbox2D([e.MinX, e.MinY, e.MaxX, e.MaxY])],
                    crs: Some(storage_crs.clone()),
                }),
                temporal: None,
            });
            collection.storage_crs = Some(storage_crs);

            // A concrete geometry type where there is one, https://github.com/georust/gdal/blob/00adecc94361228a2197224205fc9260d14d7549/gdal-sys/prebuilt-bindings/gdal_3.4.rs#L3454
            let geometry_type = match layer
                .defn()
                .geom_fields()
                .next()
                .context("the layer has no geometry")?
                .field_type()
            {
                1 => Some(("POINT", 2)),
                2 => Some(("LINESTRING", 2)),
                3 => Some(("POLYGON", 2)),
                4 => Some(("MULTIPOINT", 2)),
                5 => Some(("MULTILINESTRING", 2)),
                6 => Some(("MULTIPOLYGON", 2)),
                2_147_483_653 => Some(("MULTILINESTRINGZ", 3)),
                2_147_483_654 => Some(("MULTIPOLYGONZ", 3)),
                // any geometry
                _ => None,
            };

            (layer_name, collection, geometry_type)
        };

        // Setup driver, and create the collection (replace an existing one);
        // the driver refuses an id that can't be a table's, so the tables
        // below are named safely
        let db = Db::setup(&Url::parse(&inputs.database_url)?).await?;
        if db.read_collection(&collection.id).await?.is_some() {
            db.delete_collection(&collection.id).await?;
        }
        db.create_collection(&collection).await?;

        if let Some((geometry_type, dimensions)) = geometry_type {
            let srid = collection
                .storage_crs
                .as_ref()
                .and_then(Crs::as_srid)
                .context("cannot determine SRID")?;
            sqlx::query("SELECT DropGeometryColumn ('items', $1, 'geom')")
                .bind(&collection.id)
                .execute(&db.pool)
                .await?;
            sqlx::query("SELECT AddGeometryColumn ('items', $1, 'geom', $2, $3, $4)")
                .bind(&collection.id)
                .bind(srid)
                .bind(geometry_type)
                .bind(dimensions)
                .execute(&db.pool)
                .await?;
            sqlx::query(&format!(
                r#"CREATE INDEX ON items."{}" USING gist (geom)"#,
                collection.id
            ))
            .execute(&db.pool)
            .await?;
        }

        // Load features
        let dataset = Dataset::open(&inputs.input)?;
        let mut layer = dataset.layer_by_name(&layer_name)?;

        // Instantiate an `ArrowArrayStream` for OGR to write into
        let mut output_stream = FFI_ArrowArrayStream::empty();

        // Take a pointer to it
        let output_stream_ptr = &raw mut output_stream;

        // GDAL includes its own copy of the ArrowArrayStream struct definition. These are guaranteed
        // to be the same across implementations, but we need to manually cast between the two for Rust
        // to allow it.
        let gdal_pointer: *mut ArrowArrayStream = output_stream_ptr.cast();

        // Read the layer's data into our provisioned pointer
        unsafe { layer.read_arrow_stream(gdal_pointer, &CslStringList::new())? }

        let arrow_stream_reader = ArrowArrayStreamReader::try_new(output_stream)?;
        let schema = arrow_stream_reader.schema();

        // Get the index of the fid and geom column
        let fid_column_index = schema
            .column_with_name("OGC_FID")
            .context("no column OGC_FID")?
            .0;
        let mut geom_column_index = schema
            .column_with_name("wkb_geometry")
            .context("no column wkb_geometry")?
            .0;

        // adjust for later column removal
        if fid_column_index < geom_column_index {
            geom_column_index -= 1;
        }

        let id = &collection.id;
        let pool = &db.pool;

        for result in arrow_stream_reader {
            let mut batch = result?;

            // Get the id column
            let fid_column = batch.remove_column(fid_column_index);
            let fid_column = cast(&fid_column, &DataType::Utf8)?;
            let fid_array = fid_column
                .as_any()
                .downcast_ref::<StringArray>()
                .context("OGC_FID as text")?;
            let fid_vec: Vec<String> = (0..fid_array.len())
                .map(|i| fid_array.value(i).to_owned())
                .collect();

            // Get the geometry column
            let geom_column = batch.remove_column(geom_column_index);
            let geom_array = geom_column
                .as_any()
                .downcast_ref::<BinaryArray>()
                .context("wkb_geometry as WKB")?;
            let geom_vec: Vec<Vec<u8>> = (0..geom_array.len())
                .map(|i| geom_array.value(i).to_owned())
                .collect();

            // Get the properties
            let mut writer = ArrayWriter::new(Vec::new());
            writer.write_batches(&[&batch])?;
            writer.finish()?;
            let properties = String::from_utf8(writer.into_inner())?;

            // no await while the dataset is open
            tokio::task::block_in_place(move || {
                tokio::runtime::Handle::current().block_on(
                    sqlx::query(&format!(
                        r#"
                        INSERT INTO items."{id}" (id, properties, geom)
                        SELECT * FROM UNNEST(
                            $1::text[],
                            (SELECT
                                array_agg(properties)
                            FROM (
                                SELECT jsonb_array_elements($2::jsonb) AS properties
                            )),
                            $3::bytea[]
                        )
                        "#
                    ))
                    .bind(fid_vec)
                    .bind(properties)
                    .bind(geom_vec)
                    .execute(pool),
                )
            })?;
        }

        Ok(GdalLoaderOutputs {
            collection: inputs.collection,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Processor;

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "needs a PostgreSQL database"]
    async fn test_loader() {
        let loader = GdalLoader;
        assert_eq!(loader.id(), "gdal-loader");

        let input = GdalLoaderInputs {
            input: "../data/ne_10m_railroads_north_america.geojson".to_owned(),
            collection: "streets-gdal".to_string(),
            filter: None,
            s_srs: None,
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

        assert_eq!(output.collection, "streets-gdal");
    }
}
