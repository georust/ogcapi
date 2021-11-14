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
    // TODO: pass database url directly once GDAL 8.4 is out
    let mut db_params = Vec::new();
    if let Some(host) = database_url.host_str() {
        db_params.push(format!("host={}", host));
    }
    if let Some(port) = database_url.port() {
        db_params.push(format!("port={}", port));
    }
    if !database_url.username().is_empty() {
        db_params.push(format!("user={}", database_url.username()));
    }
    if let Some(password) = database_url.password() {
        db_params.push(format!("password={}", password));
    }
    if let Some(mut path_segments) = database_url.path_segments() {
        db_params.push(format!(
            "dbname={}",
            path_segments.next().expect("Some path segment")
        ));
    }
    let db_url = format!("PG:{}", db_params.join(" "));

    let drv = gdal::Driver::get("PostgreSQL")?;
    let ds = drv.create_vector_only(&db_url)?;

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

        let fields: Vec<(String, u32, i32)> = layer
            .defn()
            .fields()
            .map(|field| (field.name(), field.field_type(), field.width()))
            .collect();

        let mut pb = pbr::ProgressBar::new(layer.feature_count());

        let lyr = ds.layer_by_name(&format!("items.{}", collection.id))?;

        for feature in layer.features() {
            // Get the original geometry:
            let geom = feature.geometry();
            // Get a new transformed geometry:
            let new_geom = geom.transform(&transform)?;
            // Create the new feature, set its geometry:
            let mut ft = Feature::new(lyr.defn())?;
            ft.set_geometry(new_geom)?;

            feature.geometry().geometry_type();

            // Map fields
            // if let Some(id) = feature.fid() {
            //     ft.set_field("id", &FieldValue::Integer64Value(id as i64))?;
            // }

            let properties = extract_properties(&feature, &fields).await?;
            ft.set_field(
                "properties",
                &FieldValue::StringValue(serde_json::to_string(&properties)?),
            )?;

            // Add the feature to the layer:
            ft.create(&lyr)?;

            pb.inc();
        }
        pb.finish_println("");
    }

    Ok(())
}

/// Extract properties from feature
async fn extract_properties(
    feature: &Feature<'_>,
    fields: &Vec<(String, u32, i32)>,
) -> Result<serde_json::Value, anyhow::Error> {
    let mut properties = Map::new();

    for field in fields {
        if let Some(value) = feature.field(&field.0)? {
            // Match field types https://gdal.org/doxygen/ogr__core_8h.html#a787194bea637faf12d61643124a7c9fc
            let value = match field.1 {
                0 => {
                    let i = value.into_int().unwrap();
                    Value::from(i)
                }
                2 => {
                    let f = value.into_real().unwrap();
                    Value::from(f)
                }
                4 => {
                    let s = value.into_string().unwrap();
                    Value::from(s)
                }
                11 => {
                    let d = value.into_datetime().unwrap();
                    Value::from(d.to_rfc3339())
                }
                12 => {
                    let i = value.into_int64().unwrap();
                    Value::from(i)
                }
                _ => {
                    unimplemented!("Can not parse field {:?} `{:#?}` yet!", field, value);
                }
            };
            properties.insert(field.0.to_owned(), value);
        }
    }

    Ok(Value::from(properties))
}
