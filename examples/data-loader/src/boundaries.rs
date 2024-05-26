use std::{borrow::Borrow, collections::BTreeMap};

use geo::{Coord, LineString, MultiPolygon, Point, Polygon};

// from https://github.com/Qwant/osm_boundaries_utils_rs

const WARN_UNCLOSED_RING_MAX_DISTANCE: f64 = 10.;

// Define BoundaryPart in a mod to make its fields private
mod boundary_part {
    /// Wrapper arround a Vec<osmpbfreader::Node> that has length at least 2.
    pub struct BoundaryPart {
        nodes: Vec<osmpbfreader::Node>,
    }

    impl BoundaryPart {
        pub fn new(nodes: Vec<osmpbfreader::Node>) -> Option<Self> {
            if nodes.len() >= 2 {
                Some(Self { nodes })
            } else {
                None
            }
        }

        pub fn first(&self) -> osmpbfreader::NodeId {
            self.nodes.first().unwrap().id
        }

        pub fn last(&self) -> osmpbfreader::NodeId {
            self.nodes.last().unwrap().id
        }

        pub fn reverse(&mut self) {
            self.nodes.reverse();
        }

        pub fn into_vec(self) -> Vec<osmpbfreader::Node> {
            self.nodes
        }
    }
}

use self::boundary_part::BoundaryPart;

fn get_nodes<T: Borrow<osmpbfreader::OsmObj>>(
    way: &osmpbfreader::Way,
    objects: &BTreeMap<osmpbfreader::OsmId, T>,
) -> Option<Vec<osmpbfreader::Node>> {
    way.nodes
        .iter()
        .map(|node_id| objects.get(&osmpbfreader::OsmId::Node(*node_id)))
        .map(|node_obj| {
            if let Some(n) = node_obj {
                if let osmpbfreader::OsmObj::Node(ref node) = *n.borrow() {
                    Some(node.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

pub fn build_boundary<T: Borrow<osmpbfreader::OsmObj>>(
    relation: &osmpbfreader::Relation,
    objects: &BTreeMap<osmpbfreader::OsmId, T>,
) -> Option<MultiPolygon<f64>> {
    use geo::prelude::Intersects;

    let mut outer_polys = build_boundary_parts(relation, objects, vec!["outer", "enclave", ""]);
    let inner_polys = build_boundary_parts(relation, objects, vec!["inner"]);

    if let Some(ref mut outers) = outer_polys {
        if let Some(inners) = inner_polys {
            inners.into_iter().for_each(|inner| {
                /*
                    It's assumed here that the 'inner' ring is contained into
                    exactly ONE outer ring. To find it among all 'outers', all
                    we need is to find a candidate 'outer' area that shares a point
                    point with (i.e 'intersects') all 'inner' segments.
                    Using 'contains' is not suitable here, as 'inner' may touch its outer
                    ring at a single point.

                    NB: this algorithm cannot handle "donut inside donut" boundaries
                    (where 'inner' would be contained into multiple concentric outer rings).
                */
                let (exterior, _) = inner.into_inner();
                for ref mut outer in outers.0.iter_mut() {
                    if exterior.lines().all(|line| outer.intersects(&line)) {
                        outer.interiors_push(exterior);
                        break;
                    }
                }
            })
        }
    }
    outer_polys
}

pub fn build_boundary_parts<T: Borrow<osmpbfreader::OsmObj>>(
    relation: &osmpbfreader::Relation,
    objects: &BTreeMap<osmpbfreader::OsmId, T>,
    roles_to_extact: Vec<&str>,
) -> Option<MultiPolygon<f64>> {
    let roles = roles_to_extact;

    let parts = relation
        .refs
        .iter()
        .filter(|r| roles.contains(&r.role.as_str()))
        .map(|r| objects.get(&r.member))
        .collect::<Option<Vec<&T>>>()?;

    let parts = parts
        .iter()
        .cloned()
        .filter_map(|way_obj| way_obj.borrow().way())
        .map(|way| get_nodes(way, objects))
        .collect::<Option<Vec<Vec<osmpbfreader::Node>>>>()?;

    let mut boundary_parts: Vec<BoundaryPart> = parts
        .iter()
        .cloned()
        .filter_map(BoundaryPart::new)
        .collect();

    let mut multipoly = MultiPolygon(vec![]);

    let mut append_ring = |nodes: &[osmpbfreader::Node]| {
        let poly_geom = nodes
            .iter()
            .map(|n| Coord {
                x: n.lon(),
                y: n.lat(),
            })
            .collect();
        multipoly
            .0
            .push(Polygon::new(LineString(poly_geom), vec![]));
    };

    while !boundary_parts.is_empty() {
        let first_part = boundary_parts.remove(0);
        let mut added_nodes: Vec<osmpbfreader::Node> = vec![];
        let mut node_to_idx: BTreeMap<osmpbfreader::NodeId, usize> = BTreeMap::new();

        let mut add_part = |part: BoundaryPart| {
            let mut part = part.into_vec();

            let nodes = if added_nodes.is_empty() {
                part.drain(..)
            } else {
                part.drain(1..)
            };

            for n in nodes {
                if let Some(start_idx) = node_to_idx.get(&n.id) {
                    let ring = added_nodes.split_off(*start_idx);
                    node_to_idx = added_nodes
                        .iter()
                        .enumerate()
                        .map(|(i, n)| (n.id, i))
                        .collect();
                    if ring.len() >= 3 {
                        append_ring(&ring);
                    } else {
                        // debug!(
                        //     "Ignored ring with less than 3 nodes in relation:{} at node:{}",
                        //     relation.id.0, n.id.0
                        // );
                    }
                }
                node_to_idx.insert(n.id, added_nodes.len());
                added_nodes.push(n);
            }
        };

        let mut current = first_part.last();
        add_part(first_part);

        loop {
            let mut added_part = false;
            let mut i = 0;
            while i < boundary_parts.len() {
                if current == boundary_parts[i].first() {
                    // the start of current way touches the polygon,
                    // we add it and remove it from the pool
                    current = boundary_parts[i].last();
                    add_part(boundary_parts.remove(i));
                    added_part = true;
                } else if current == boundary_parts[i].last() {
                    // the end of the current way touches the polygon, we reverse the way and add it
                    current = boundary_parts[i].first();
                    boundary_parts[i].reverse();
                    add_part(boundary_parts.remove(i));
                    added_part = true;
                } else {
                    i += 1;
                    // didn't do anything, we want to explore the next way, if we had do something we
                    // will have removed the current way and there will be no need to increment
                }
            }
            if !added_part {
                use geo::haversine_distance::HaversineDistance;
                let p = |n: &osmpbfreader::Node| {
                    Point(Coord {
                        x: n.lon(),
                        y: n.lat(),
                    })
                };

                if added_nodes.len() > 1 {
                    let distance = p(added_nodes.first().unwrap())
                        .haversine_distance(&p(added_nodes.last().unwrap()));
                    if distance < WARN_UNCLOSED_RING_MAX_DISTANCE {
                        // warn!(
                        //     "boundary: relation/{} ({}): unclosed polygon, dist({:?}, {:?}) = {}",
                        //     relation.id.0,
                        //     relation.tags.get("name").map_or("", |s| &s),
                        //     added_nodes.first().unwrap().id,
                        //     added_nodes.last().unwrap().id,
                        //     distance
                        // );
                    }
                }
                break;
            }
        }
    }
    if multipoly.0.is_empty() {
        None
    } else {
        Some(multipoly)
    }
}
