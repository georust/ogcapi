extern crate osmpbfreader;

mod boundaries;

use std::{collections::BTreeMap, convert::TryInto, path::PathBuf};

use anyhow;
use gdal::{
    spatial_ref::{CoordTransform, SpatialRef},
    vector::{Feature, Layer},
};
use geo::{Geometry, LineString, MultiLineString, Point, Polygon};
use osmpbfreader::{OsmId, OsmObj, OsmPbfReader, Ref, Relation};
use serde_json::{json, Map, Value};
use sqlx::{types::Json, Pool, Postgres, Transaction};

use crate::{
    collection::{Collection, Extent, ItemType, Provider, Summaries},
    common::Link,
};

pub async fn gdal_import(
    input: &PathBuf,
    layer_name: &Option<String>,
    collection: &Option<String>,
    pool: &Pool<Postgres>,
) -> Result<(), anyhow::Error> {
    // Enable reading from url, TODO: read ZIP
    let input = if input.to_str().map(|s| s.starts_with("http")).unwrap() {
        PathBuf::from("/vsicurl").join(input.to_owned())
    } else {
        input.to_owned()
    };

    let dataset = gdal::Dataset::open(&input)?;

    for mut layer in dataset.layers() {
        // only load specified layers
        if layer_name.is_some() && Some(layer.name()) != *layer_name {
            continue;
        }

        // Create collection
        let collection = collection_from_layer(&layer, collection)?;
        delete_collection(&collection.id, &pool).await?;
        insert_collection(&collection, &pool).await?;

        println!("Importing layer: `{}`", &collection.title.unwrap());

        let fields: Vec<(String, u32, i32)> = layer
            .defn()
            .fields()
            .map(|field| (field.name(), field.field_type(), field.width()))
            .collect();
        // println!("fileds_def:\n{:#?}", fields);

        // Prepare the origin and destination spatial references objects:
        let spatial_ref_src = layer.spatial_ref()?;
        let spatial_ref_dst = SpatialRef::from_epsg(4326)?;
        spatial_ref_src.set_axis_mapping_strategy(0);
        spatial_ref_dst.set_axis_mapping_strategy(0);

        // And the feature used to actually transform the geometries:
        let transform = CoordTransform::new(&spatial_ref_src, &spatial_ref_dst)?;

        // Load features
        let pb = indicatif::ProgressBar::new(layer.feature_count());
        let mut tx = pool.begin().await?;
        for feature in layer.features() {
            create_feature(&feature, &collection.id, &transform, &fields, &mut tx).await?;

            pb.inc(1)
        }
        tx.commit().await?;
        pb.finish_with_message("done");
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
async fn create_feature(
    feature: &Feature<'_>,
    collection: &str,
    transform: &CoordTransform,
    fields: &Vec<(String, u32, i32)>,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), anyhow::Error> {
    let id = feature.fid().expect("feature identifier") as i32;
    let geometry = feature.geometry().transform(transform)?.wkb()?;
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
                    unimplemented!("Can not parse field type {} `{:#?}` yet!", field.1, value)
                }
            };
            properties.insert(field.0.to_owned(), value);
        }
    }

    sqlx::query_file!(
        "sql/feature_import.sql",
        id,
        collection,
        Value::from(properties) as _,
        geometry as _,
    )
    .execute(tx)
    .await?;

    Ok(())
}

/// Import osm data from pbf file
pub async fn osm_import(
    input: &PathBuf,
    _filter: &Option<String>,
    collection: &Option<String>,
    pool: &Pool<Postgres>,
) -> Result<(), anyhow::Error> {
    let r = std::fs::File::open(input)?;
    let mut pbf = OsmPbfReader::new(r);

    let objs = pbf.get_objs_and_deps(|_o| true)?;
    // println!("found {} objects and dependencies", objs.len());

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

    pbf.rewind()?;

    // Load features
    let pb = indicatif::ProgressBar::new(objs.len().try_into()?);
    let mut tx = pool.begin().await?;
    for obj in pbf.par_iter().map(Result::unwrap) {
        pb.inc(1);

        // skip objects without tags
        if obj.tags().is_empty() {
            continue;
        }

        // get id
        let id = obj.id().inner_id();

        // extract tags
        let keys = obj.tags().keys().into_iter().map(|k| k.to_string());
        let values = obj.tags().values().into_iter().map(|v| v.to_string());
        let mut properties = Map::new();
        for (k, v) in keys.zip(values) {
            properties.insert(k, Value::from(v));
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
        } else {
            continue;
        }
    }
    tx.commit().await?;
    pb.finish_with_message("done");

    Ok(())
}

fn geometry_from_obj(obj: &OsmObj, objs: &BTreeMap<OsmId, OsmObj>) -> Option<Geometry<f64>> {
    match obj {
        OsmObj::Node(node) => Some(Point::new(node.lon(), node.lat()).into()),
        OsmObj::Way(way) => {
            let geom = LineString::from(
                way.nodes
                    .iter()
                    .filter_map(|id| {
                        objs.get(&OsmId::Node(*id))
                            .and_then(|o| o.node())
                            .map(|n| (n.lon(), n.lat()))
                    })
                    .collect::<Vec<(f64, f64)>>(),
            );

            assert!(geom.lines().count() > 0);

            if geom.is_closed() {
                Some(Polygon::new(geom, vec![]).into())
            } else {
                Some(geom.into())
            }
        }
        OsmObj::Relation(rel) => {
            // match type of relation https://wiki.openstreetmap.org/wiki/Types_of_relation
            if let Some(rel_type) = rel.tags.get("type").map(|s| s.as_str()) {
                if vec!["multipolygon", "boundary"].contains(&rel_type) {
                    boundaries::build_boundary(&rel, &objs).map(|p| p.into())
                } else if vec![
                    "multilinestring",
                    "route",
                    "route_master",
                    "waterway",
                    "network",
                ]
                .contains(&rel_type)
                {
                    let refs = RefIter::new(rel, &objs);
                    let geom = MultiLineString(
                        refs.filter_map(|r| {
                            objs.get(&r.member).and_then(|o| o.way()).map(|w| {
                                LineString::from(
                                    w.nodes
                                        .iter()
                                        .filter_map(|n| {
                                            objs.get(&OsmId::Node(*n))
                                                .and_then(|n| n.node())
                                                .map(|n| (n.lon(), n.lat()))
                                        })
                                        .collect::<Vec<(f64, f64)>>(),
                                )
                            })
                        })
                        .collect::<Vec<LineString<f64>>>(),
                    );
                    Some(geom.into())
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

struct RefIter<'a> {
    refs: Vec<Ref>,
    objs: &'a BTreeMap<OsmId, OsmObj>,
}

impl<'a> RefIter<'a> {
    fn new(relation: &'a Relation, objs: &'a BTreeMap<OsmId, OsmObj>) -> Self {
        RefIter {
            refs: relation.refs.to_owned(),
            objs,
        }
    }
}

impl<'a> Iterator for RefIter<'a> {
    type Item = Ref;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(r) = self.refs.pop() {
            if let Some(mut relation) = self.objs.get(&r.member).and_then(|o| o.relation().cloned())
            {
                self.refs.append(&mut relation.refs);
            }
            Some(r)
        } else {
            None
        }
    }
}
