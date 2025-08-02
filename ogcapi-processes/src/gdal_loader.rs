use std::collections::{HashMap, HashSet};

use anyhow::Result;
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
    common::{Bbox, Collection, Crs, Exception, Extent, SpatialExtent},
    processes::{
        Execute, Format, InlineOrRefData, Input, InputValueNoObject, Output, Process,
        TransmissionMode,
    },
};

use crate::{ProcessResponseBody, Processor};

/// GDAL loader `Processor`
///
/// Process to load vector data.
#[derive(Clone)]
pub struct GdalLoader;

/// Inputs for the `gdal-loader` process
#[derive(Deserialize, Debug, JsonSchema)]
pub struct GdalLoaderInputs {
    /// Input file
    pub input: String,

    /// Set the collection name, defaults to layer name or `osm`
    pub collection: String,

    /// Filter input by layer name, imports all if not present
    pub filter: Option<String>,

    /// Source srs, if omitted tries to derive from the input layer
    pub s_srs: Option<u32>,

    /// Postgres database url
    pub database_url: String,
}

impl GdalLoaderInputs {
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

        if let Some(filter) = &self.filter {
            input.insert(
                "filter".to_owned(),
                Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
                    InputValueNoObject::String(filter.to_owned()),
                )),
            );
        }

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
pub struct GdalLoaderOutputs {
    pub collection: String,
}

impl GdalLoaderOutputs {
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

impl TryFrom<ProcessResponseBody> for GdalLoaderOutputs {
    type Error = Exception;

    fn try_from(value: ProcessResponseBody) -> Result<Self, Self::Error> {
        if let ProcessResponseBody::Requested(buf) = value {
            Ok(GdalLoaderOutputs {
                collection: String::from_utf8(buf).unwrap(),
            })
        } else {
            Err(Exception::new("500"))
        }
    }
}

#[async_trait::async_trait]
impl Processor for GdalLoader {
    fn id(&self) -> &'static str {
        "gdal-loader"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn process(&self) -> Result<Process> {
        Process::try_new(
            self.id(),
            self.version(),
            &schema_for!(GdalLoaderInputs).schema,
            &schema_for!(GdalLoaderOutputs).schema,
        )
        .map_err(Into::into)
    }

    async fn execute(&self, execute: Execute) -> Result<ProcessResponseBody> {
        // Parse input
        let value = serde_json::to_value(execute.inputs)?;
        let mut inputs: GdalLoaderInputs = serde_json::from_value(value)?;

        // Handle http & zip
        if inputs.input.to_lowercase().starts_with("http") {
            inputs.input = format!("/vsicurl/{}", inputs.input);
        }
        if inputs.input.to_lowercase().ends_with("zip") {
            inputs.input = format!("/vsizip/{}", inputs.input);
        }

        // Get collection
        let collection = {
            let dataset = Dataset::open(&inputs.input)?;

            // Get layer
            if dataset.layer_count() >= 1 && inputs.filter.is_none() {
                inputs.filter = Some(dataset.layer(0)?.name());
            }

            if inputs.filter.is_none() {
                return Err(Exception::new(format!(
                    "Found multiple layers! Use the 'filter' option to specifiy one of:\n\t- {}",
                    dataset
                        .layers()
                        .map(|l| l.name())
                        .collect::<Vec<String>>()
                        .join("\n\t- ")
                ))
                .into());
            }

            let layer = dataset.layer_by_name(inputs.filter.as_ref().unwrap())?;

            // Get coordinate reference system
            let spatial_ref_src = match inputs.s_srs {
                Some(epsg) => SpatialRef::from_epsg(epsg)?,
                None => match layer.spatial_ref() {
                    Some(srs) => srs,
                    None => {
                        println!("Unknown spatial reference, falling back to `4326`");
                        SpatialRef::from_epsg(4326)?
                    }
                },
            };

            let storage_crs = Crs::from_srid(spatial_ref_src.auth_code()?);

            // Create collection (overwrite/delete existing)
            Collection {
                id: inputs.collection.clone(),
                crs: Vec::from_iter(HashSet::from([
                    Crs::default(),
                    storage_crs.clone(),
                    Crs::from_epsg(3857),
                ])),
                extent: layer.try_get_extent().unwrap().map(|e| Extent {
                    spatial: Some(SpatialExtent {
                        bbox: vec![Bbox::Bbox2D([e.MinX, e.MinY, e.MaxX, e.MaxY])],
                        crs: storage_crs.to_owned(),
                    }),
                    temporal: None,
                }),
                storage_crs: Some(storage_crs.to_owned()),
                ..Default::default()
            }
        };

        // Setup driver
        let db = Db::setup(&Url::parse(&inputs.database_url)?).await?;

        db.delete_collection(&collection.id).await.unwrap();
        db.create_collection(&collection).await.unwrap();

        // Set concrete geometry type if possible https://github.com/georust/gdal/blob/00adecc94361228a2197224205fc9260d14d7549/gdal-sys/prebuilt-bindings/gdal_3.4.rs#L3454
        if let Some((geometry_type, dimensions)) = {
            let dataset = Dataset::open(&inputs.input)?;
            let layer = dataset
                .layer_by_name(inputs.filter.as_ref().unwrap())
                .unwrap();

            match layer.defn().geom_fields().next().unwrap().field_type() {
                0 => {
                    panic!("Unknown gemetry type.")
                }
                1 => Some(("POINT", 2)),
                2 => Some(("LINESTRING", 2)),
                3 => Some(("POLYGON", 2)),
                4 => Some(("MULTIPOINT", 2)),
                5 => Some(("MULTILINESTRING", 2)),
                6 => Some(("MULTIPOLYGON", 2)),
                2147483653 => Some(("MULTILINESTRINGZ", 3)),
                2147483654 => Some(("MULTIPOLYGONZ", 3)),
                i => {
                    panic!("Unmaped geometry type `{i}`");
                }
            }
        } {
            sqlx::query("SELECT DropGeometryColumn ('items', $1, 'geom')")
                .bind(&collection.id)
                .execute(&db.pool)
                .await?;

            sqlx::query("SELECT AddGeometryColumn ('items', $1, 'geom', $2, $3, $4)")
                .bind(&collection.id)
                .bind(collection.storage_crs.unwrap().as_srid())
                .bind(geometry_type)
                .bind(dimensions)
                .execute(&db.pool)
                .await?;

            sqlx::query(&format!(
                r#"CREATE INDEX ON items."{}" USING gist (geom)"#,
                &collection.id
            ))
            .execute(&db.pool)
            .await?;
        }

        // Load features
        // let _count = layer.lock().unwrap().feature_count();

        let dataset = Dataset::open(&inputs.input)?;
        let mut layer = dataset.layer_by_name(inputs.filter.as_ref().unwrap())?;

        // Instantiate an `ArrowArrayStream` for OGR to write into
        let mut output_stream = FFI_ArrowArrayStream::empty();

        // Take a pointer to it
        let output_stream_ptr = &mut output_stream as *mut FFI_ArrowArrayStream;

        // GDAL includes its own copy of the ArrowArrayStream struct definition. These are guaranteed
        // to be the same across implementations, but we need to manually cast between the two for Rust
        // to allow it.
        let gdal_pointer: *mut ArrowArrayStream = output_stream_ptr.cast();

        // Read the layer's data into our provisioned pointer
        unsafe { layer.read_arrow_stream(gdal_pointer, &CslStringList::new())? }

        let arrow_stream_reader = ArrowArrayStreamReader::try_new(output_stream)?;
        let schema = arrow_stream_reader.schema();

        // Get the index of the fid and geom column
        let fid_column_index = schema.column_with_name("OGC_FID").unwrap().0;
        let mut geom_column_index = schema.column_with_name("wkb_geometry").unwrap().0;

        // adjust for later column removal
        if fid_column_index < geom_column_index {
            geom_column_index -= 1;
        }

        let id = &collection.id;
        let pool = &db.pool;

        for result in arrow_stream_reader {
            let mut batch = result?;
            println!("Got some batch with {} features", batch.num_rows());

            // Get the id column
            let fid_column = batch.remove_column(fid_column_index);
            let fid_column = cast(&fid_column, &DataType::Utf8)?;
            let fid_array = fid_column.as_any().downcast_ref::<StringArray>().unwrap();
            let mut fid_vec = Vec::with_capacity(fid_array.len());
            (0..fid_array.len()).for_each(|i| fid_vec.push(fid_array.value(i).to_owned()));

            // Get the geometry column
            let geom_column = batch.remove_column(geom_column_index);
            let geom_array = geom_column.as_any().downcast_ref::<BinaryArray>().unwrap();
            let mut geom_vec = Vec::with_capacity(geom_array.len());
            (0..geom_array.len()).for_each(|i| geom_vec.push(geom_array.value(i).to_owned()));

            // Get the properties
            let buf = Vec::new();
            let mut writer = ArrayWriter::new(buf);
            writer.write_batches(&[&batch])?;
            writer.finish()?;

            let properties = String::from_utf8(writer.into_inner())?;

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

        Ok(ProcessResponseBody::Requested(
            inputs.collection.as_bytes().to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use ogcapi_types::processes::Execute;

    use crate::{
        Processor,
        gdal_loader::{GdalLoader, GdalLoaderInputs, GdalLoaderOutputs},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn test_loader() {
        let loader = GdalLoader;
        assert_eq!(loader.id(), "gdal-loader");

        println!(
            "Process:\n{}",
            serde_json::to_string_pretty(&loader.process().unwrap()).unwrap()
        );

        let input = GdalLoaderInputs {
            input: "../data/ne_10m_railroads_north_america.geojson".to_owned(),
            collection: "streets-gdal".to_string(),
            filter: None,
            s_srs: None,
            database_url: "postgresql://postgres:password@localhost:5433/ogcapi".to_string(),
        };

        let execute = Execute {
            inputs: input.execute_input(),
            ..Default::default()
        };

        let output: GdalLoaderOutputs = loader.execute(execute).await.unwrap().try_into().unwrap();
        assert_eq!(output.collection, "streets-gdal");
    }
}
