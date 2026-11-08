use ogcapi_types::{
    features::{Feature, FeatureCollection},
    stac::SearchParams,
};

use crate::StacSearch;

use super::{Db, datetime_condition, envelopes, items_table};

#[allow(clippy::too_many_lines, reason = "Complex search logic")]
#[async_trait::async_trait]
impl StacSearch for Db {
    async fn search(&self, query: &SearchParams) -> anyhow::Result<FeatureCollection> {
        let mut tx = self.pool.begin().await?;

        // WITH
        let mut collection_ids: Vec<String> = sqlx::query_scalar(
            r"
            SELECT id FROM meta.collections 
            WHERE collection ->> 'type' = 'Collection'
            ",
        )
        .fetch_all(&mut *tx)
        .await?;

        if !query.collections.is_empty() {
            collection_ids.retain(|id| query.collections.contains(id));
        }

        let union_all_items = collection_ids
            .iter()
            .filter_map(|collection_id| items_table(collection_id))
            .map(|table| format!("SELECT * FROM {table}"))
            .collect::<Vec<String>>()
            .join(" UNION ALL ");

        if union_all_items.is_empty() {
            let mut fc = FeatureCollection::new(Vec::new());
            fc.number_matched = Some(0);
            return Ok(fc);
        }

        // WHERE
        let mut where_conditions = vec!["TRUE".to_string()];

        // bbox
        if let Some(bbox) = query.bbox.as_ref() {
            // CRS84, as the search's geometry (STAC API), compared in each item's
            // storage system
            let intersects: Vec<String> = envelopes(bbox, 4326)
                .iter()
                .map(|envelope| {
                    format!("ST_Intersects(geom, ST_Transform({envelope}, ST_SRID(geom)))")
                })
                .collect();
            where_conditions.push(format!("({})", intersects.join(" OR ")));
        }

        // datetime
        if let Some(datetime) = query.datetime.as_ref() {
            where_conditions.push(datetime_condition(datetime));
        }

        // ids and intersects, bound in this order
        let ids = (!query.ids.is_empty()).then_some(&query.ids);
        let intersects = query
            .intersects
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        if ids.is_some() {
            where_conditions.push("id = ANY($1)".to_owned());
        }
        if intersects.is_some() {
            let n = 1 + usize::from(ids.is_some());
            where_conditions.push(format!(
                "ST_Intersects(geom, ST_Transform(ST_SetSRID(ST_GeomFromGeoJSON(${n}), 4326), ST_SRID(geom)))"
            ));
        }

        let conditions = where_conditions.join(" AND ");

        // COUNT
        let sql = format!(
            r"
            WITH items AS ({union_all_items})
            SELECT count(*) FROM items
            WHERE {conditions}
            ",
        );
        let mut count = sqlx::query_as(&sql);
        if let Some(ids) = ids {
            count = count.bind(ids);
        }
        if let Some(intersects) = &intersects {
            count = count.bind(intersects);
        }
        let number_matched: (i64,) = count.fetch_one(&mut *tx).await?;

        // FETCH
        let sql = format!(
            r"
            WITH items AS ({union_all_items})
            SELECT array_to_json(array_agg(row_to_json(t)))
            FROM (
                SELECT
                    id,
                    collection,
                    properties,
                    ST_AsGeoJSON(ST_Transform(geom, 4326))::jsonb as geometry,
                    links,
                    assets,
                    bbox
                FROM items
                WHERE {conditions}
                LIMIT {}
                OFFSET {}
            ) t
            ",
            query
                .limit
                .map_or_else(|| String::from("NULL"), |l| l.to_string()),
            query.offset.unwrap_or(0)
        );
        let mut fetch = sqlx::query_scalar(&sql);
        if let Some(ids) = ids {
            fetch = fetch.bind(ids);
        }
        if let Some(intersects) = &intersects {
            fetch = fetch.bind(intersects);
        }
        let features: Option<sqlx::types::Json<Vec<Feature>>> = fetch.fetch_one(&mut *tx).await?;

        tx.commit().await?;

        let features = features.map(|f| f.0).unwrap_or_default();
        let mut fc = FeatureCollection::new(features);
        fc.number_matched = Some(number_matched.0.cast_unsigned());

        Ok(fc)
    }
}
