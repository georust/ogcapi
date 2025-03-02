use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Mutex,
};

use anyhow::Result;
use arrow::{
    array::{Array, BinaryArray, RecordBatchReader, StringArray},
    compute::cast,
    datatypes::DataType,
    ffi_stream::{ArrowArrayStreamReader, FFI_ArrowArrayStream},
    json::ArrayWriter,
};
use gdal::{
    cpl::CslStringList, spatial_ref::SpatialRef, vector::LayerAccess, ArrowArrayStream, Dataset,
};
use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use url::Url;

use ogcapi_drivers::{postgres::Db, CollectionTransactions};
use ogcapi_types::{
    common::{Bbox, Collection, Crs, Exception, Extent, SpatialExtent},
    processes::{Execute, InlineOrRefData, Input, InputValueNoObject, Process},
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
        let value = serde_json::to_value(execute.inputs)?;
        let mut inputs: GdalLoaderInputs = serde_json::from_value(value)?;

        // Setup driver
        let db = Db::setup(&Url::parse(&inputs.database_url)?).await?;

        // Handle http & zip
        if inputs.input.to_lowercase().starts_with("http") {
            inputs.input = format!("/vsicurl/{}", inputs.input);
        }
        if inputs.input.to_lowercase().ends_with("zip") {
            inputs.input = format!("/vsizip/{}", inputs.input);
        }

        // Get vector layer
        let dataset = Dataset::open(&inputs.input)?;

        let layer = if let Some(filter) = inputs.filter {
            Rc::new(Mutex::new(dataset.layer_by_name(&filter)?))
        } else if dataset.layer_count() > 1 {
            return Err(Exception::new(format!(
                "Found multiple layers! Use the '--filter' option to specifiy one of:\n\t- {}",
                dataset
                    .layers()
                    .map(|l| l.name())
                    .collect::<Vec<String>>()
                    .join("\n\t- ")
            ))
            .into());
        } else {
            Rc::new(Mutex::new(dataset.layer(0)?))
        };

        // Get coordinate reference system
        let spatial_ref_src = match inputs.s_srs {
            Some(epsg) => Rc::new(Mutex::new(SpatialRef::from_epsg(epsg)?)),
            None => match layer.lock().unwrap().spatial_ref() {
                Some(srs) => Rc::new(Mutex::new(srs)),
                None => {
                    println!("Unknown spatial reference, falling back to `4326`");
                    Rc::new(Mutex::new(SpatialRef::from_epsg(4326)?))
                }
            },
        };

        let storage_crs = Crs::from_srid(spatial_ref_src.lock().unwrap().auth_code()?);

        // Create collection (overwrite/delete existing)
        let collection = Collection {
            id: inputs.collection.to_owned(),
            crs: Vec::from_iter(HashSet::from([
                Crs::default(),
                storage_crs.clone(),
                Crs::from_epsg(3857),
            ])),
            extent: layer.lock().unwrap().try_get_extent()?.map(|e| Extent {
                spatial: Some(SpatialExtent {
                    bbox: vec![Bbox::Bbox2D([e.MinX, e.MinY, e.MaxX, e.MaxY])],
                    crs: storage_crs.to_owned(),
                }),
                temporal: None,
            }),
            storage_crs: Some(storage_crs.to_owned()),
            ..Default::default()
        };

        let handle = tokio::runtime::Handle::current();

        let db2 = db.clone();
        let id = collection.id.clone();
        handle.spawn_blocking(async move || {
            db2.delete_collection(&id).await.unwrap();
        });
        let db2 = db.clone();
        let collection2 = collection.clone();
        handle.spawn_blocking(async move || {
            db2.create_collection(&collection2).await.unwrap();
        });

        // Set concrete geometry type if possible https://github.com/georust/gdal/blob/00adecc94361228a2197224205fc9260d14d7549/gdal-sys/prebuilt-bindings/gdal_3.4.rs#L3454
        if let Some((geometry_type, dimensions)) = match layer
            .lock()
            .unwrap()
            .defn()
            .geom_fields()
            .next()
            .unwrap()
            .field_type()
        {
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
        } {
            let pool = db.pool.clone();
            let collection2 = collection.clone();

            handle.spawn_blocking(async move || {
                sqlx::query("SELECT DropGeometryColumn ('items', $1, 'geom')")
                    .bind(&collection2.id)
                    .execute(&pool)
                    .await
                    .unwrap();

                sqlx::query("SELECT AddGeometryColumn ('items', $1, 'geom', $2, $3, $4)")
                    .bind(&collection2.id)
                    .bind(storage_crs.as_srid())
                    .bind(geometry_type)
                    .bind(dimensions)
                    .execute(&pool)
                    .await
                    .unwrap();

                sqlx::query(&format!(
                    r#"CREATE INDEX ON items."{}" USING gist (geom)"#,
                    collection2.id
                ))
                .execute(&pool)
                .await
                .unwrap();
            });
        }

        // Load features
        let _count = layer.lock().unwrap().feature_count();

        // Instantiate an `ArrowArrayStream` for OGR to write into
        let mut output_stream = FFI_ArrowArrayStream::empty();

        // Take a pointer to it
        let output_stream_ptr = &mut output_stream as *mut FFI_ArrowArrayStream;

        // GDAL includes its own copy of the ArrowArrayStream struct definition. These are guaranteed
        // to be the same across implementations, but we need to manually cast between the two for Rust
        // to allow it.
        let gdal_pointer: *mut ArrowArrayStream = output_stream_ptr.cast();

        // Read the layer's data into our provisioned pointer
        unsafe {
            layer
                .lock()
                .unwrap()
                .read_arrow_stream(gdal_pointer, &CslStringList::new())?
        }

        let arrow_stream_reader = ArrowArrayStreamReader::try_new(output_stream)?;
        let schema = arrow_stream_reader.schema();

        // Get the index of the fid and geom column
        let fid_column_index = schema.column_with_name("OGC_FID").unwrap().0;
        let mut geom_column_index = schema.column_with_name("wkb_geometry").unwrap().0;

        // adjust for later column removal
        if fid_column_index < geom_column_index {
            geom_column_index -= 1;
        }

        for result in arrow_stream_reader {
            let mut batch = result?;

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

            let id = collection.id.clone();
            let pool = db.pool.clone();

            handle.spawn_blocking(async move || {
                sqlx::query(&format!(
                    r#"
                INSERT INTO items."{}" (id, properties, geom)
                SELECT * FROM UNNEST(
                    $1::text[], 
                    (SELECT
                        array_agg(properties)
                    FROM (
                        SELECT jsonb_array_elements($2::jsonb) AS properties
                    )),
                    $3::bytea[]
                )
                "#,
                    id
                ))
                .bind(fid_vec)
                .bind(properties)
                .bind(geom_vec)
                .execute(&pool)
                .await
                .unwrap()
            });
        }
        Ok(ProcessResponseBody::Requested(
            // inputs.collection.as_bytes().to_owned(),
            vec![1_u8],
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Instant};

    use ogcapi_types::processes::Execute;

    use crate::{
        gdal_loader::{GdalLoader, GdalLoaderInputs, GdalLoaderOutputs},
        Processor,
    };

    #[tokio::test]
    async fn test_loader() {
        let loader = GdalLoader;
        assert_eq!(loader.id(), "gdal-loader");

        println!(
            "Process:\n{}",
            serde_json::to_string_pretty(&loader.process().unwrap()).unwrap()
        );

        let input = GdalLoaderInputs {
            input: "../data/ne_10m_railroads_north_america.geojson".to_owned(),
            collection: "streets".to_string(),
            filter: None,
            s_srs: None,
            database_url: "postgresql://postgres:password@localhost:5433/ogcapi".to_string(),
        };

        let execute = Execute {
            inputs: input.execute_input(),
            outputs: HashMap::new(),
            subscriber: None,
        };

        let now = Instant::now();

        let output: GdalLoaderOutputs = loader.execute(execute).await.unwrap().try_into().unwrap();
        println!("{}", output.collection);

        // stats
        let count = 121895;
        let elapsed = now.elapsed().as_secs_f64();
        println!(
            "Loaded {count} features in {elapsed:.3} seconds ({:.2}/s)",
            count as f64 / elapsed
        );
    }
}
