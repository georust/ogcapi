use ogcapi_types::{
    edr::{Query, QueryType},
    features::{Feature, FeatureCollection},
};
use sqlx::types::Json;

use crate::{CollectionTransactions, EdrQuerier};

use super::Db;

#[async_trait::async_trait]
impl EdrQuerier for Db {
    async fn query(
        &self,
        collection_id: &str,
        query_type: &QueryType,
        query: &Query,
    ) -> anyhow::Result<FeatureCollection> {
        let srid: i32 = query.crs.as_srid();

        let c = self.read_collection(collection_id).await?;
        let storage_srid = c.unwrap().storage_crs.unwrap_or_default().as_srid();

        let mut geometry_type = query.coords.split('(').next().unwrap().to_uppercase();
        geometry_type.retain(|c| !c.is_whitespace());

        let spatial_predicate = match &query_type {
            QueryType::Position | QueryType::Area | QueryType::Trajectory => {
                if geometry_type.ends_with('Z') || geometry_type.ends_with('M') {
                    format!(
                        "ST_3DIntersects(geom, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), {}))",
                        srid, query.coords, storage_srid
                    )
                } else {
                    format!(
                        "ST_Intersects(geom, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), {}))",
                        srid, query.coords, storage_srid
                    )
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
                    format!(
                        "ST_3DDWithin(geom, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), {}))",
                        srid, query.coords, storage_srid
                    )
                } else {
                    format!(
                    "ST_DWithin(ST_Transform(geom, 4326)::geography, ST_Transform(ST_GeomFromEWKT('SRID={};{}'), 4326)::geography, {}, false)",
                    srid, query.coords, distance
                )
                }
            }
            QueryType::Cube => {
                let bbox: Vec<&str> = query.coords.split(',').collect();
                if bbox.len() == 4 {
                    format!(
                        "ST_Intersects(geom, ST_Transform(ST_MakeEnvelope({}, {}), {})",
                        query.coords, srid, storage_srid
                    )
                } else {
                    format!(
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
                    )
                }
            }
            QueryType::Corridor | QueryType::Locations => unimplemented!(),
        };

        let properties = if let Some(parameters) = &query.parameter_name {
            format!(
                "{0} as properties",
                parameters
                    .split(',')
                    .map(|s| format!(
                        "('{{\"{0}\":' || (properties -> '{0}')::text || '}}')::jsonb",
                        s
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
                {1},
                ST_AsGeoJSON(ST_Transform(geom, $1))::jsonb as geometry,
                links,
                '{0}' as collection,
                assets
            FROM items."{0}"
            WHERE {2}
            "#,
            collection_id, properties, spatial_predicate
        );

        let number_matched = sqlx::query(&sql)
            .bind(srid)
            .execute(&self.pool)
            .await?
            .rows_affected();

        let features: Option<Json<Vec<Feature>>> = sqlx::query_scalar(&format!(
            r#"
            SELECT array_to_json(array_agg(row_to_json(t)))
            FROM ( {} ) t
            "#,
            sql
        ))
        .bind(srid)
        .fetch_one(&self.pool)
        .await?;

        let features = features.map(|f| f.0).unwrap_or_default();
        let mut fc = FeatureCollection::new(features);
        fc.number_matched = Some(number_matched);

        Ok(fc)
    }
}
