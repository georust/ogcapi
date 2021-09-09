use std::path::PathBuf;

use gdal::{
    spatial_ref::{CoordTransform, SpatialRef},
    vector::{Feature, FieldValue, Layer},
};

use serde_json::{json, Map, Value};

use crate::{common::collections::Collection, db::Db};

pub async fn gdal_import(
    input: PathBuf,
    filter: &Option<String>,
    collection: &Option<String>,
    db: &Db,
) -> Result<(), anyhow::Error> {
    // GDAL Configuration Options http://trac.osgeo.org/gdal/wiki/ConfigOptions
    gdal::config::set_config_option("PG_USE_COPY", "YES")?;
    gdal::config::set_config_option("OGR_PG_RETRIEVE_FID", "FALSE")?;
    gdal::config::set_config_option("PGSQL_OGR_FID", "id")?;

    // Get target dataset layer
    let drv = gdal::Driver::get("PostgreSQL")?;
    // let db_url = format!("PG:{}", std::env::var("DATABASE_URL")?);
    let db_url = "PG:host=localhost user=postgres dbname=ogcapi password=postgres"; // workaround gdal issue

    let ds = drv.create_vector_only(&db_url)?;

    // Open input dataset
    let input = if input.to_str().map(|s| s.starts_with("http")).unwrap() {
        PathBuf::from("/vsicurl").join(input.to_owned())
    } else {
        input.to_owned()
    };

    let dataset = gdal::Dataset::open(&input)?;

    for mut layer in dataset.layers() {
        // only load specified layers
        if filter.is_some() && Some(layer.name()) != *filter {
            continue;
        }

        // Create collection
        let collection = collection_from_layer(&layer, collection)?;
        db.delete_collection(&collection.id).await?;
        db.insert_collection(&collection).await?;

        log::info!("Importing layer: `{}`", &collection.title.unwrap());

        let fields: Vec<(String, u32, i32)> = layer
            .defn()
            .fields()
            .map(|field| (field.name(), field.field_type(), field.width()))
            .collect();

        log::debug!("fields_def: {:?}", fields);

        // Prepare the origin and destination spatial references objects:
        let spatial_ref_src = layer.spatial_ref()?;
        let spatial_ref_dst = SpatialRef::from_epsg(4326)?;

        spatial_ref_src.set_axis_mapping_strategy(0);
        spatial_ref_dst.set_axis_mapping_strategy(0);

        // And the feature used to actually transform the geometries:
        let transform = CoordTransform::new(&spatial_ref_src, &spatial_ref_dst)?;

        // Load features
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

/// Create new collection metadata from gdal layer
fn collection_from_layer(
    layer: &Layer,
    collection: &Option<String>,
) -> Result<Collection, anyhow::Error> {
    let title = collection.to_owned().unwrap_or_else(|| layer.name());

    let extent = layer.try_get_extent()?.and_then(|e| {
        serde_json::from_value(json!({
            "spatial": {
                "bbox": [e.MinX, e.MinY, e.MaxX, e.MinY],
                "crs": "http://www.opengis.net/def/crs/OGC/1.3/CRS84",
            }
        }))
        .ok()
    });

    let collection = Collection {
        id: title.to_lowercase().replace(" ", "_"),
        title: Some(title),
        links: serde_json::from_str("[]")?,
        crs: Some(vec![
            "http://www.opengis.net/def/crs/OGC/1.3/CRS84".to_string()
        ]),
        extent,
        ..Default::default()
    };

    Ok(collection)
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
                    unimplemented!("Can not parse field type {} `{:#?}` yet!", field.1, value);
                }
            };
            properties.insert(field.0.to_owned(), value);
        }
    }

    Ok(Value::from(properties))
}
