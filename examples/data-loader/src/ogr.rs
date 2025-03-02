use std::{collections::HashSet, ffi::OsStr, path::PathBuf, time::Instant};

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

use ogcapi::{
    drivers::{CollectionTransactions, postgres::Db},
    types::common::{Bbox, Collection, Crs, Extent, SpatialExtent},
};

use super::Args;

pub async fn load(mut args: Args) -> Result<(), anyhow::Error> {
    let now = Instant::now();

    // Setup a db connection pool
    let db = Db::setup(&args.database_url).await?;

    // Handle http & zip
    if args.input.starts_with("http") && args.input.extension() == Some(OsStr::new("zip")) {
        args.input = PathBuf::from(format!(
            "/vsizip//vsicurl/{}",
            args.input.as_path().to_str().unwrap()
        ))
    } else if args.input.display().to_string().starts_with("http") {
        args.input = PathBuf::from("/vsicurl/").join(args.input.as_path())
    } else if args.input.extension() == Some(OsStr::new("zip")) {
        args.input = PathBuf::from("/vsizip/").join(args.input.as_path())
    }

    // Get vector layer
    let dataset = Dataset::open(&args.input)?;

    let mut layer = if let Some(filter) = args.filter {
        dataset.layer_by_name(&filter)?
    } else if dataset.layer_count() > 1 {
        tracing::warn!(
            "Found multiple layers! Use the '--filter' option to specifiy one of:\n\t- {}",
            dataset
                .layers()
                .map(|l| l.name())
                .collect::<Vec<String>>()
                .join("\n\t- ")
        );
        return Ok(());
    } else {
        dataset.layer(0).unwrap()
    };

    // Get coordinate reference system
    let spatial_ref_src = match args.s_srs {
        Some(epsg) => SpatialRef::from_epsg(epsg)?,
        None => match layer.spatial_ref() {
            Some(srs) => srs,
            None => {
                tracing::warn!("Unknown spatial reference, falling back to `4326`");
                SpatialRef::from_epsg(4326)?
            }
        },
    };

    let storage_crs = Crs::from_srid(spatial_ref_src.auth_code()?);

    // Create collection (overwrite/delete existing)
    let collection = Collection {
        id: args.collection.to_owned(),
        crs: Vec::from_iter(HashSet::from([
            Crs::default(),
            storage_crs.clone(),
            Crs::from_epsg(3857),
        ])),
        extent: layer.try_get_extent()?.map(|e| Extent {
            spatial: Some(SpatialExtent {
                bbox: vec![Bbox::Bbox2D([e.MinX, e.MinY, e.MaxX, e.MaxY])],
                crs: storage_crs.to_owned(),
            }),
            temporal: None,
        }),
        storage_crs: Some(storage_crs.to_owned()),
        #[cfg(feature = "stac")]
        assets: crate::asset::load_asset_from_path(&args.input).await?,
        ..Default::default()
    };

    db.delete_collection(&collection.id).await?;
    db.create_collection(&collection).await?;

    // Set concrete geometry type if possible https://github.com/georust/gdal/blob/00adecc94361228a2197224205fc9260d14d7549/gdal-sys/prebuilt-bindings/gdal_3.4.rs#L3454
    if let Some((geometry_type, dimensions)) =
        match layer.defn().geom_fields().next().unwrap().field_type() {
            0 => {
                tracing::debug!("Unknown gemetry type.");
                None
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
                tracing::warn!("Unmaped geometry type `{i}`");
                None
            }
        }
    {
        sqlx::query("SELECT DropGeometryColumn ('items', $1, 'geom')")
            .bind(&collection.id)
            .execute(&db.pool)
            .await?;
        sqlx::query("SELECT AddGeometryColumn ('items', $1, 'geom', $2, $3, $4)")
            .bind(&collection.id)
            .bind(storage_crs.as_srid())
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
    let count = layer.feature_count();

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

    for result in arrow_stream_reader {
        let mut batch = result?;

        // Get the id column
        let fid_column = batch.remove_column(fid_column_index);
        let fid_column = cast(&fid_column, &DataType::Utf8)?;
        let fid_array = fid_column.as_any().downcast_ref::<StringArray>().unwrap();
        let mut fid_vec = Vec::with_capacity(fid_array.len());
        (0..fid_array.len()).for_each(|i| fid_vec.push(fid_array.value(i)));

        // Get the geometry column
        let geom_column = batch.remove_column(geom_column_index);
        let geom_array = geom_column.as_any().downcast_ref::<BinaryArray>().unwrap();
        let mut geom_vec = Vec::with_capacity(geom_array.len());
        (0..geom_array.len()).for_each(|i| geom_vec.push(geom_array.value(i)));

        // Get the properties
        let buf = Vec::new();
        let mut writer = ArrayWriter::new(buf);
        writer.write_batches(&[&batch])?;
        writer.finish()?;

        let properties = String::from_utf8(writer.into_inner())?;

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
            collection.id
        ))
        .bind(fid_vec)
        .bind(properties)
        .bind(geom_vec)
        .execute(&db.pool)
        .await?;
    }

    // stats
    let elapsed = now.elapsed().as_millis() as f64 / 1000.0;
    tracing::info!(
        "Loaded {count} features in {elapsed} seconds ({:.2}/s)",
        count as f64 / elapsed
    );

    Ok(())
}
