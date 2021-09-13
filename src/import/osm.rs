use std::{collections::BTreeMap, fs::File, path::PathBuf};

use geo::{Coordinate, Geometry, LineString, MultiLineString, Point, Polygon};
use osmpbfreader::{NodeId, OsmId, OsmObj, OsmPbfReader};

use serde_json::{Map, Value};

use crate::{
    common::{collections::Collection, Crs},
    db::Db,
    import::boundaries,
};

/// Import osm data from pbf file
pub async fn osm_import(
    input: PathBuf,
    _filter: &Option<String>,
    collection: &Option<String>,
    db: &Db,
) -> Result<(), anyhow::Error> {
    // Create collection
    let title = collection.to_owned().unwrap_or_else(|| "OSM".to_string());

    let collection = Collection {
        id: title.to_lowercase().replace(" ", "_"),
        title: Some(title),
        links: serde_json::from_str("[]")?,
        crs: Some(vec![Crs::default()]),
        ..Default::default()
    };
    db.delete_collection(&collection.id).await?;
    db.insert_collection(&collection).await?;

    // Open file
    let file = File::open(input)?;
    let mut pbf = OsmPbfReader::new(file);

    let blob_count = pbf.blobs().count();
    log::info!("Found {} blobs!", blob_count);
    pbf.rewind()?;

    // let mut pb = pbr::ProgressBar::new((blobs * 1000).try_into()?);

    // Cache
    let objs = pbf.get_objs_and_deps(|_| true)?;
    log::info!("Found {} obj and dependencies!", objs.len());
    pbf.rewind()?;

    log::info!("Done caching!");

    let mut tx = db.pool.begin().await?;

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

        let keys = obj.tags().keys();
        let values = obj.tags().values();

        for (k, v) in keys.zip(values) {
            properties.insert(k.to_string(), Value::from(v.as_str()));
        }

        // build geometry
        if let Some(geometry) =
            geometry_from_obj(&obj, &objs).and_then(|g| wkb::geom_to_wkb(&g).ok())
        {
            sqlx::query(&format!(
                r#"INSERT INTO {} (
                    id,
                    feature_type,
                    properties,
                    geom
                ) VALUES ($1, '"Feature"', $2, ST_GeomFromWKB($3, 4326))"#,
                collection.id
            ))
            .bind(id as i32)
            .bind(Value::from(properties) as Value)
            .bind(geometry as Vec<u8>)
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
                        .map(|_id| {
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

fn to_linestring(nodes: &[NodeId], objs: &BTreeMap<OsmId, OsmObj>) -> Option<LineString<f64>> {
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
    fn new(refs: &'a [osmpbfreader::Ref], cache: &'a BTreeMap<OsmId, OsmObj>) -> Self {
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
