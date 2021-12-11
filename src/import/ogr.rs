use std::path::PathBuf;

use gdal::{
    spatial_ref::{CoordTransform, SpatialRef},
    vector::{Feature, FieldValue},
};
use serde_json::{Map, Value};
use url::Url;

use crate::common::{
    collections::{Collection, Extent, SpatialExtent},
    core::Bbox,
    crs::Crs,
};
use crate::db::Db;

use super::Args;

pub async fn load(mut args: Args, database_url: &Url) -> Result<(), anyhow::Error> {
    // GDAL Configuration Options http://trac.osgeo.org/gdal/wiki/ConfigOptions
    gdal::config::set_config_option("PG_USE_COPY", "YES")?;
    gdal::config::set_config_option("OGR_PG_RETRIEVE_FID", "FALSE")?;
    gdal::config::set_config_option("PGSQL_OGR_FID", "id")?;

    // Get target dataset layer
    let drv = gdal::Driver::get("PostgreSQL")?;
    let ds = drv.create_vector_only(&database_url.to_string())?;

    // Setup a db connection pool
    let db = Db::connect(database_url.as_str()).await?;

    // Open input dataset
    if args.input.starts_with("http") {
        args.input = PathBuf::from("/vsicurl").join(args.input.as_path())
    };
    if args.input.ends_with("zip") {
        args.input = PathBuf::from("/vsizip").join(args.input.as_path())
    };
    let dataset = gdal::Dataset::open(&args.input)?;

    for mut layer in dataset.layers() {
        // only load specified layers
        if args.filter.is_some() && Some(layer.name()) != args.filter {
            continue;
        }

        // Prepare the origin and destination spatial references objects and coordinate transformation
        let spatial_ref_src = match args.s_srs {
            Some(epsg) => SpatialRef::from_epsg(epsg)?,
            None => layer.spatial_ref()?,
        };

        let spatial_ref_dst = match args.t_srs {
            Some(epsg) => SpatialRef::from_epsg(epsg)?,
            None => layer.spatial_ref()?,
        };

        spatial_ref_src.set_axis_mapping_strategy(0);
        spatial_ref_dst.set_axis_mapping_strategy(0);

        let transform = CoordTransform::new(&spatial_ref_src, &spatial_ref_dst)?;

        // Create collection
        let title = args.collection.to_owned().unwrap_or_else(|| layer.name());
        let storage_crs = Crs::from(spatial_ref_dst.auth_code()?);

        let collection = Collection {
            id: title.to_lowercase().replace(" ", "_"),
            title: Some(title),
            links: serde_json::from_str("[]")?,
            crs: Some(vec![Crs::default(), storage_crs.clone()]),
            extent: layer.try_get_extent()?.map(|e| {
                let mut x = [e.MinX, e.MaxX];
                let mut y = [e.MinY, e.MaxY];
                // let mut z = [e.MinZ, e.MaxZ];
                transform
                    .transform_coords(&mut x, &mut y, &mut [])
                    .expect("Transform extent coords");
                Extent {
                    spatial: Some(SpatialExtent {
                        bbox: Some(vec![Bbox::Bbox2D(x[0], y[0], x[1], y[1])]),
                        crs: spatial_ref_dst.auth_code().map(|c| c.into()).ok(),
                    }),
                    temporal: None,
                }
            }),
            storage_crs: Some(storage_crs),
            ..Default::default()
        };

        // db.delete_collection(&collection.id).await?;
        db.insert_collection(&collection).await?;

        // Load features
        log::info!("Importing layer: `{}`", &collection.title.unwrap());

        let field_names: Vec<String> = layer.defn().fields().map(|f| f.name()).collect();

        let mut pb = pbr::ProgressBar::new(layer.feature_count());

        let lyr = ds.layer_by_name(&format!("items.{}", collection.id))?;

        for feature in layer.features() {
            // Extract & transform geometry
            let geom = feature.geometry();
            let geom = geom.transform(&transform)?;

            // Extract properties
            let mut properties = Map::new();

            for field_name in &field_names {
                let value = if let Some(value) = feature.field(field_name)? {
                    match value {
                        FieldValue::IntegerValue(v) => Value::from(v),
                        FieldValue::IntegerListValue(v) => Value::from(v),
                        FieldValue::Integer64Value(v) => Value::from(v),
                        FieldValue::Integer64ListValue(v) => Value::from(v),
                        FieldValue::StringValue(v) => Value::from(v),
                        FieldValue::StringListValue(v) => Value::from(v),
                        FieldValue::RealValue(v) => Value::from(v),
                        FieldValue::RealListValue(v) => Value::from(v),
                        FieldValue::DateValue(v) => Value::from(v.naive_utc().to_string()),
                        FieldValue::DateTimeValue(v) => Value::from(v.to_rfc3339()),
                    }
                } else {
                    Value::Null;
                };
                properties.insert(field_name.to_owned(), value);
            }

            // Create a new feature
            let mut ft = Feature::new(lyr.defn())?;
            ft.set_geometry(geom)?;
            ft.set_field(
                "properties",
                &FieldValue::StringValue(serde_json::to_string(&Value::from(properties))?),
            )?;

            // Add the feature to the layer
            ft.create(&lyr)?;

            pb.inc();
        }
        pb.finish_println("");
    }

    Ok(())
}
