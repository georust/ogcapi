extern crate osmpbfreader;

mod boundaries;

use std::{collections::BTreeMap, fs::File, path::PathBuf};

use gdal::{
    spatial_ref::{CoordTransform, SpatialRef},
    vector::{Feature, FieldValue, Layer},
};
use geo::{Coordinate, Geometry, LineString, MultiLineString, Point, Polygon};
use osmpbfreader::{NodeId, OsmId, OsmObj, OsmPbfReader};

use serde_json::{json, Map, Value};
use sqlx::{postgres::PgPoolOptions, types::Json, Pool, Postgres};

use crate::{
    collections::{Collection, Extent, ItemType, Provider, Summaries},
    common::Link,
};

pub async fn import(
    input: PathBuf,
    filter: &Option<String>,
    collection: &Option<String>,
) -> Result<(), anyhow::Error> {
    // Create a connection pool
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Import data
    if input.extension() == Some(std::ffi::OsStr::new("pbf")) {
        osm_import(input, &filter, &collection, &pool).await
    } else {
        gdal_import(input, &filter, &collection, &pool).await
    }
}

pub async fn gdal_import(
    input: PathBuf,
    filter: &Option<String>,
    collection: &Option<String>,
    pool: &Pool<Postgres>,
) -> Result<(), anyhow::Error> {
    // GDAL Configuration Options http://trac.osgeo.org/gdal/wiki/ConfigOptions
    gdal::config::set_config_option("PG_USE_COPY", "YES")?;

    // Get target dataset layer
    let drv = Driver::get("PostgreSQL")?;
    let ds = drv.create_vector_only(&format!("PG:{}", std::env::var("DATABASE_URL")?))?;
    let lyr = ds.layer_by_name("features")?;

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
        delete_collection(&collection.id, &pool).await?;
        insert_collection(&collection, &pool).await?;

        log::info!("Importing layer: `{}`", &collection.title.unwrap());

        let fields: Vec<(String, u32, i32)> = layer
            .defn()
            .fields()
            .map(|field| (field.name(), field.field_type(), field.width()))
            .collect();
        log::debug!("fileds_def:\n{:#?}", fields);

        // Prepare the origin and destination spatial references objects:
        let spatial_ref_src = layer.spatial_ref()?;
        let spatial_ref_dst = SpatialRef::from_epsg(4326)?;
        spatial_ref_src.set_axis_mapping_strategy(0);
        spatial_ref_dst.set_axis_mapping_strategy(0);

        // And the feature used to actually transform the geometries:
        let transform = CoordTransform::new(&spatial_ref_src, &spatial_ref_dst)?;

        // Load features
        let mut pb = pbr::ProgressBar::new(layer.feature_count());

        for feature in layer.features() {
            // Get the original geometry:
            let geom = feature.geometry();
            // Get a new transformed geometry:
            let new_geom = geom.transform(&transform)?;
            // Create the new feature, set its geometry:
            let mut ft = Feature::new(lyr.defn())?;
            ft.set_geometry(new_geom)?;

            // Map fields
            let id = feature.fid().expect("feature identifier") as i64;
            ft.set_field("id", &FieldValue::Integer64Value(id))?;

            ft.set_field(
                "collection",
                &FieldValue::StringValue(collection.id.to_owned()),
            )?;

            let properties = extract_properties(&feature, &fields).await?;
            ft.set_field(
                "properties",
                &FieldValue::StringValue(serde_json::to_string(&properties)?),
            )?;

            // Add the feature to the layer:
            ft.create(&lyr)?;

            pb.inc();
        }
        pb.finish();
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

/// Insert collection metadata
async fn insert_collection(
    collection: &Collection,
    pool: &Pool<Postgres>,
) -> Result<(), anyhow::Error> {
    let _ = sqlx::query_file!(
        "sql/collection_insert.sql",
        collection.id,
        collection.title,
        collection.description,
        collection.links as _,
        collection.extent as _,
        collection.item_type as _,
        collection.crs.as_deref(),
        collection.storage_crs,
        collection.storage_crs_coordinate_epoch,
        collection.stac_version,
        collection.stac_extensions.as_deref(),
        collection.keywords.as_deref(),
        collection.licence,
        collection.providers as _,
        collection.summaries as _
    )
    .fetch_one(pool)
    .await?;

    Ok(())
}

/// Delete collection metadata
async fn delete_collection(id: &str, pool: &Pool<Postgres>) -> Result<(), anyhow::Error> {
    let _ = sqlx::query_file!("sql/collection_delete.sql", id)
        .fetch_optional(pool)
        .await?;
    Ok(())
}

/// Create new feature
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

/// Import osm data from pbf file
pub async fn osm_import(
    input: PathBuf,
    _filter: &Option<String>,
    collection: &Option<String>,
    pool: &Pool<Postgres>,
) -> Result<(), anyhow::Error> {
    // Create collection
    let title = collection.to_owned().unwrap_or_else(|| "OSM".to_string());

    let collection = Collection {
        id: title.to_lowercase().replace(" ", "_"),
        title: Some(title),
        links: serde_json::from_str("[]")?,
        crs: Some(vec![
            "http://www.opengis.net/def/crs/OGC/1.3/CRS84".to_string()
        ]),
        ..Default::default()
    };
    delete_collection(&collection.id, pool).await?;
    insert_collection(&collection, pool).await?;

    // Open file
    let file = File::open(input)?;
    let mut pbf = OsmPbfReader::new(file);

    let blob_count = pbf.blobs().count();
    log::info!("Found {} blobs!", blob_count);
    pbf.rewind()?;

    let block_count = pbf.primitive_blocks().count();
    log::info!("Found {} bloks!", block_count);
    pbf.rewind()?;

    // let mut pb = pbr::ProgressBar::new((blobs * 1000).try_into()?);

    // Cache
    let objs = pbf.get_objs_and_deps(|_| true)?;
    log::info!("Found {} obj and dependencies!", objs.len());
    pbf.rewind()?;

    // pbf.par_iter().for_each(|obj| {
    //     let obj = obj.unwrap();

    //     match obj {
    //         OsmObj::Node(n) => {
    //             coords_cache.insert(
    //                 n.id,
    //                 Coordinate {
    //                     x: n.lon(),
    //                     y: n.lat(),
    //                 },
    //             );
    //         }
    //         OsmObj::Way(w) => {
    //             way_cache.insert(w.id, w.nodes);
    //         }
    //         OsmObj::Relation(r) => {
    //             ref_cache.insert(r.id, r.refs.iter().map(|r| r.member).collect());
    //         }
    //     }
    // });

    log::info!("Done caching!");

    let mut tx = pool.begin().await?;

    pbf.rewind()?;

    for obj in pbf.par_iter() {
        let obj = obj.unwrap();

        // skip objects without tags
        if obj.tags().is_empty() {
            continue;
        }

        // get id
        let id = obj.id().inner_id();

        // extract tags
        let mut properties = Map::new();

        let keys = obj.tags().keys().into_iter();
        let values = obj.tags().values().into_iter();

        for (k, v) in keys.zip(values) {
            properties.insert(k.to_string(), Value::from(v.as_str()));
        }

        // build geometry
        if let Some(geometry) = geometry_from_obj(&obj, &objs) {
            sqlx::query_file!(
                "sql/feature_import.sql",
                id as i32,
                collection.id,
                Value::from(properties) as _,
                wkb::geom_to_wkb(&geometry).expect("convert geom to wkb") as _,
            )
            .execute(&mut tx)
            .await?;
        }
    }

    tx.commit().await?;
    // pb.inc();

    Ok(())
}

fn geometry_from_obj(obj: &OsmObj, objs: &BTreeMap<OsmId, OsmObj>) -> Option<Geometry<f64>> {
    match obj {
        OsmObj::Node(node) => Some(Point::new(node.lon(), node.lat()).into()),
        OsmObj::Way(way) => to_linestring(&way.nodes, objs).map(|l| {
            if l.is_closed() {
                Polygon::new(l, vec![]).into()
            } else {
                l.into()
            }
        }),
        OsmObj::Relation(rel) => {
            // match type of relation https://wiki.openstreetmap.org/wiki/Types_of_relation
            if let Some(rel_type) = rel.tags.get("type").map(|s| s.as_str()) {
                if vec!["multipolygon", "boundary"].contains(&rel_type) {
                    boundaries::build_boundary(&rel, objs).map(|p| p.into())
                } else if vec![
                    "multilinestring",
                    "route",
                    "route_master",
                    "waterway",
                    "network",
                ]
                .contains(&rel_type)
                {
                    RefIter::new(&rel.refs, objs)
                        .filter_map(|id| id.way())
                        .map(|id| {
                            objs.get(&obj.id())
                                .and_then(|obj| obj.way())
                                .and_then(|way| to_linestring(&way.nodes, objs))
                        })
                        .collect::<Option<Vec<LineString<f64>>>>()
                        .map(|l| MultiLineString(l).into())
                } else if vec![
                    "collection",
                    "public_transport",
                    "site",
                    "restriction",
                    "street",
                    "bridge",
                    "tunnel",
                ]
                .contains(&rel_type)
                {
                    // TODO: geometry collection
                    // Some(GeometryCollection::<f64>(vec![]));
                    None
                } else {
                    None
                }
            } else {
                // println!("found relation without type tag:\n{:#?}", &obj);
                None
            }
        }
    }
}

fn to_linestring(nodes: &Vec<NodeId>, objs: &BTreeMap<OsmId, OsmObj>) -> Option<LineString<f64>> {
    if nodes.len() < 2 {
        return None;
    }

    nodes
        .iter()
        .map(|id| objs.get(&OsmId::Node(*id)).and_then(to_coordinate))
        .collect::<Option<Vec<Coordinate<f64>>>>()
        .map(LineString::from)
}

fn to_coordinate(obj: &OsmObj) -> Option<Coordinate<f64>> {
    obj.node().map(|n| Coordinate {
        x: n.lon(),
        y: n.lat(),
    })
}

struct RefIter<'a> {
    refs: Vec<osmpbfreader::Ref>,
    cache: &'a BTreeMap<OsmId, OsmObj>,
}

impl<'a> RefIter<'a> {
    fn new(refs: &'a Vec<osmpbfreader::Ref>, cache: &'a BTreeMap<OsmId, OsmObj>) -> Self {
        RefIter {
            refs: refs.to_owned(),
            cache,
        }
    }
}

impl<'a> Iterator for RefIter<'a> {
    type Item = OsmId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(r) = self.refs.pop() {
            if let Some(mut relation) = self
                .cache
                .get(&r.member)
                .and_then(|obj| obj.relation())
                .cloned()
            {
                self.refs.append(&mut relation.refs);
            } else {
                return Some(r.member);
            }
        }
        None
    }
}
