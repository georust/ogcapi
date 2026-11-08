use anyhow::Context;
use sqlx::types::Json;

use ogcapi_types::{
    common::{Crs, Exception},
    edr::{Query, QueryType},
    features::{Feature, FeatureCollection},
};

use crate::{CollectionTransactions, EdrQuerier};

use super::{Db, items_table};

#[allow(clippy::too_many_lines, reason = "Complex query logic")]
#[async_trait::async_trait]
impl EdrQuerier for Db {
    async fn query(
        &self,
        collection_id: &str,
        query_type: &QueryType,
        query: &Query,
    ) -> anyhow::Result<(FeatureCollection, Crs)> {
        let table = items_table(collection_id).ok_or_else(|| Exception::new_from_status(404))?;

        let collection = self.read_collection(collection_id).await?;
        let storage_srid: i32 = match collection {
            Some(collection) => match collection.storage_crs.map(|crs| crs.as_srid()) {
                Some(srid) => srid.context("cannot determine SRID")?,
                None => {
                    sqlx::query_scalar("SELECT Find_SRID('items', $1, 'geom')")
                        .bind(collection_id)
                        .fetch_one(&self.pool)
                        .await?
                }
            },
            None => return Err(Exception::new_from_status(404).into()),
        };

        let mut geometry_type = query
            .coords
            .split('(')
            .next()
            .context("cannot determine geometry type")?
            .to_uppercase();
        geometry_type.retain(|c| !c.is_whitespace());
        let has_zm = geometry_type.ends_with('Z') || geometry_type.ends_with('M');

        let srid: i32 = query
            .crs
            .as_ref()
            .map_or_else(
                || {
                    if has_zm
                        || (matches!(query_type, QueryType::Cube)
                            && query.coords.split(',').count() == 6)
                    {
                        Crs::default3d().as_srid()
                    } else {
                        Crs::default2d().as_srid()
                    }
                },
                Crs::as_srid,
            )
            .context("cannot determine SRID")?;

        // `$1` is the response's SRID, `$2` the coordinates as WKT where a
        // query has them, then the parameters' names
        let wkt = !matches!(query_type, QueryType::Cube);
        let coords = format!("ST_Transform(ST_GeomFromText($2, {srid}), {storage_srid})");
        let spatial_predicate = match &query_type {
            QueryType::Position | QueryType::Area | QueryType::Trajectory => {
                if has_zm {
                    format!("ST_3DIntersects(geom, {coords})")
                } else {
                    format!("ST_Intersects(geom, {coords})")
                }
            }
            QueryType::Radius => {
                let within = query.within.as_deref().unwrap_or("0").trim();
                let units = query.within_units.as_deref().unwrap_or("m").trim();
                let refused = || {
                    Exception::new_from_status(400)
                        .detail(format!("cannot take `{within} {units}` as a distance"))
                };
                // only a number and a unit's name reach rink, no other
                // expression of its language
                let number = within
                    .parse::<f64>()
                    .ok()
                    .filter(|d| d.is_finite() && *d >= 0.0)
                    .ok_or_else(refused)?;
                if units.is_empty()
                    || !units
                        .chars()
                        .all(|c| c.is_alphabetic() || c == ' ' || c == '_')
                {
                    return Err(refused().into());
                }
                let mut ctx =
                    rink_core::simple_context().map_err(|error| anyhow::anyhow!("{error}"))?;
                // "1609.344 meter (length)", or "approx. 0.333 meter (length)"
                let distance = rink_core::one_line(&mut ctx, &format!("{number} {units} -> m"))
                    .ok()
                    .and_then(|answer| {
                        answer
                            .split_whitespace()
                            .find_map(|word| word.parse::<f64>().ok())
                    })
                    .ok_or_else(refused)?;

                if has_zm {
                    // in the storage system's units, as PostGIS measures in 3D
                    format!("ST_3DDWithin(geom, {coords}, {distance})")
                } else {
                    format!(
                        "ST_DWithin(ST_Transform(geom, 4326)::geography, \
                        ST_Transform(ST_GeomFromText($2, {srid}), 4326)::geography, {distance}, false)"
                    )
                }
            }
            QueryType::Cube => {
                let bbox = query
                    .coords
                    .split(',')
                    .map(|n| n.trim().parse::<f64>().ok().filter(|n| n.is_finite()))
                    .collect::<Option<Vec<f64>>>()
                    .unwrap_or_default();
                match bbox[..] {
                    [xmin, ymin, xmax, ymax] => format!(
                        "ST_Intersects(geom, ST_Transform(\
                        ST_MakeEnvelope({xmin}, {ymin}, {xmax}, {ymax}, {srid}), {storage_srid}))"
                    ),
                    [xmin, ymin, zmin, xmax, ymax, zmax] => format!(
                        "ST_3DIntersects(geom, ST_Transform(ST_SetSRID(ST_3DMakeBox(\
                        ST_MakePoint({xmin}, {ymin}, {zmin}), ST_MakePoint({xmax}, {ymax}, {zmax})), \
                        {srid}), {storage_srid}))"
                    ),
                    _ => {
                        return Err(Exception::new_from_status(400)
                            .detail("a cube's `bbox` holds four or six numbers")
                            .into());
                    }
                }
            }
            _ => return Err(Exception::new_from_status(404).into()),
        };

        let names: Vec<&str> = query
            .parameter_name
            .as_deref()
            .map(|names| names.split(',').collect())
            .unwrap_or_default();
        let properties = if names.is_empty() {
            "properties".to_string()
        } else {
            let first = 2 + usize::from(wkt);
            let selected = (first..first + names.len())
                .map(|i| format!("jsonb_build_object(${i}, properties -> ${i})"))
                .collect::<Vec<String>>()
                .join(" || ");
            format!("{selected} AS properties")
        };

        let sql = format!(
            r"
            SELECT
                id,
                {properties},
                ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb as geometry,
                links,
                collection,
                assets
            FROM {table}
            WHERE {spatial_predicate}
            "
        );

        let mut count = sqlx::query(&sql).bind(srid);
        if wkt {
            count = count.bind(&query.coords);
        }
        for name in &names {
            count = count.bind(*name);
        }
        let number_matched = count.execute(&self.pool).await?.rows_affected();

        let sql = format!("SELECT array_to_json(array_agg(row_to_json(t))) FROM ( {sql} ) t");
        let mut fetch = sqlx::query_scalar(&sql).bind(srid);
        if wkt {
            fetch = fetch.bind(&query.coords);
        }
        for name in &names {
            fetch = fetch.bind(*name);
        }
        let features: Option<Json<Vec<Feature>>> = fetch.fetch_one(&self.pool).await?;

        let features = features.map(|f| f.0).unwrap_or_default();
        let mut fc = FeatureCollection::new(features);
        fc.number_matched = Some(number_matched);

        Ok((fc, Crs::from_srid(srid)))
    }
}
