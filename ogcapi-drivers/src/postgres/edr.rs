use sqlx::types::Json;

use ogcapi_types::{
    common::{Crs, Exception},
    edr::{Query, QueryType},
    features::{Feature, FeatureCollection},
};

use crate::{CollectionTransactions, EdrQuerier};

use super::Db;

#[async_trait::async_trait]
impl EdrQuerier for Db {
    async fn query(
        &self,
        collection_id: &str,
        query_type: &QueryType,
        query: &Query,
    ) -> anyhow::Result<(FeatureCollection, Crs)> {
        let collection = self.read_collection(collection_id).await?;
        let storage_srid = match collection {
            Some(collection) => match collection.storage_crs.map(|crs| crs.as_srid()) {
                Some(srid) => srid,
                None => {
                    sqlx::query_scalar(&format!(
                        "SELECT Find_SRID('items', '{collection_id}', 'geom')"
                    ))
                    .fetch_one(&self.pool)
                    .await?
                }
            },
            None => return Err(Exception::new_from_status(404).into()),
        };

        let mut geometry_type = query.coords.split('(').next().unwrap().to_uppercase();
        geometry_type.retain(|c| !c.is_whitespace());

        let (spatial_predicate, srid) = match &query_type {
            QueryType::Position | QueryType::Area | QueryType::Trajectory => {
                if geometry_type.ends_with('Z') || geometry_type.ends_with('M') {
                    let srid: i32 = query
                        .crs
                        .as_ref()
                        .map(|crs| crs.as_srid())
                        .unwrap_or_else(|| Crs::default3d().as_srid());
                    let predicate = format!(
                        "ST_3DIntersects(geom, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), {}))",
                        srid, query.coords, storage_srid
                    );
                    (predicate, srid)
                } else {
                    let srid: i32 = query
                        .crs
                        .as_ref()
                        .map(|crs| crs.as_srid())
                        .unwrap_or_else(|| Crs::default2d().as_srid());
                    let predicate = format!(
                        "ST_Intersects(geom, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), {}))",
                        srid, query.coords, storage_srid
                    );
                    (predicate, srid)
                }
            }
            QueryType::Radius => {
                let mut ctx = rink_core::simple_context().unwrap();
                let line = format!(
                    "{} {} -> m",
                    &query.within.to_owned().unwrap_or_else(|| "0".to_string()),
                    &query
                        .within_units
                        .to_owned()
                        .unwrap_or_else(|| "m".to_string())
                );

                let distance = rink_core::one_line(&mut ctx, &line)
                    .ok()
                    .and_then(|s| s.split(' ').next().and_then(|s| s.parse::<f64>().ok()))
                    .expect("Failed to parse & convert distance");

                if geometry_type.ends_with('Z') || geometry_type.ends_with('M') {
                    let srid: i32 = query
                        .crs
                        .as_ref()
                        .map(|crs| crs.as_srid())
                        .unwrap_or_else(|| Crs::default3d().as_srid());
                    let predicate = format!(
                        "ST_3DDWithin(geom, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), {}))",
                        srid, query.coords, storage_srid
                    );
                    (predicate, srid)
                } else {
                    let srid: i32 = query
                        .crs
                        .as_ref()
                        .map(|crs| crs.as_srid())
                        .unwrap_or_else(|| Crs::default2d().as_srid());
                    let predicate = format!(
                        "ST_DWithin(ST_Transform(geom, 4326)::geography, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), 4326)::geography, {}, false)",
                        srid, query.coords, distance
                    );
                    (predicate, srid)
                }
            }
            QueryType::Cube => {
                let bbox: Vec<&str> = query.coords.split(',').collect();
                if bbox.len() == 4 {
                    let srid: i32 = query
                        .crs
                        .as_ref()
                        .map(|crs| crs.as_srid())
                        .unwrap_or_else(|| Crs::default2d().as_srid());
                    let predicate = format!(
                        "ST_Intersects(geom, ST_Transform(ST_MakeEnvelope({}, {}), {})",
                        query.coords, srid, storage_srid
                    );
                    (predicate, srid)
                } else {
                    let srid: i32 = query
                        .crs
                        .as_ref()
                        .map(|crs| crs.as_srid())
                        .unwrap_or_else(|| Crs::default3d().as_srid());
                    let predicate = format!(
                        "ST_3DIntersects(
                            geom,
                            ST_Transform(
                                ST_SetSRID(
                                    ST_3DMakeBox(ST_MakePoint({}, {}, {}), ST_MakePoint({} , {}, {})),
                                    {}
                                ),
                                {}
                            )
                        )",
                        bbox[0], bbox[1], bbox[2], bbox[3], bbox[4], bbox[5], srid, storage_srid
                    );
                    (predicate, srid)
                }
            }
            qt => unimplemented!("{qt:?}"),
        };

        let properties = if let Some(parameters) = &query.parameter_name {
            format!(
                "{0} as properties",
                parameters
                    .split(',')
                    .map(|s| format!(
                        "('{{\"{s}\":' || (properties -> '{s}')::text || '}}')::jsonb"
                    ))
                    .collect::<Vec<String>>()
                    .join("||")
            )
        } else {
            "properties".to_string()
        };

        let sql = format!(
            r#"
            SELECT
                id,
                {properties},
                ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb as geometry,
                links,
                '{collection_id}' as collection,
                assets
            FROM items."{collection_id}"
            WHERE {spatial_predicate}
            "#
        );

        let number_matched = sqlx::query(&sql)
            .bind(srid)
            .execute(&self.pool)
            .await?
            .rows_affected();

        let features: Option<Json<Vec<Feature>>> = sqlx::query_scalar(&format!(
            r#"
            SELECT array_to_json(array_agg(row_to_json(t)))
            FROM ( {sql} ) t
            "#
        ))
        .bind(srid)
        .fetch_one(&self.pool)
        .await?;

        let features = features.map(|f| f.0).unwrap_or_default();
        let mut fc = FeatureCollection::new(features);
        fc.number_matched = Some(number_matched);

        Ok((fc, Crs::from_srid(srid)))
    }
}
