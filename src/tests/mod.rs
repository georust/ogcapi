#[allow(unused_imports)]
use geojson::{Bbox, Geometry};
#[allow(unused_imports)]
use serde_json::json;
#[allow(unused_imports)]
use sqlx::postgres::PgPoolOptions;
#[allow(unused_imports)]
use sqlx::types::Json;
#[allow(unused_imports)]
use std::env;

#[allow(unused_imports)]
use crate::collections::{Collection, CollectionType, Extent, Provider, Summaries};
#[allow(unused_imports)]
use crate::common::Link;
#[allow(unused_imports)]
use crate::features::{Assets, Feature, FeatureType};

#[async_std::test]
async fn minimal_feature_crud() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let collection: Collection = serde_json::from_value(json!({
        "id": "test",
        "links": [{
            "href": "collections/test",
            "rel": "self"
        }]
    }))
    .unwrap();

    // create collection
    let inserted_collection = sqlx::query_file_as!(
        Collection,
        "sql/collection_insert.sql",
        collection.id,
        collection.title,
        collection.description,
        collection.links as _,
        collection.extent as _,
        collection.collection_type as _,
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
    .fetch_one(&pool)
    .await
    .unwrap();

    println!(
        "{}",
        serde_json::to_string_pretty(&inserted_collection).unwrap()
    );

    let feature: Feature = serde_json::from_value(json!({
        "id": null,
        "collection": "test",
        "type": "Feature",
        "geometry": {
            "type": "Point",
            "coordinates": [7.428959, 1.513394],
            "crs" : "urn:ogc:def:crs:EPSG::2056"
        },
        "links": [{
            "href": "collections/trials/items/{id}",
            "rel": "self"
        }]
    }))
    .unwrap();

    // create feature
    let inserted_feature = sqlx::query_file_as!(
        Feature,
        "sql/feature_insert.sql",
        &collection.id,
        feature.feature_type as _,
        feature.properties,
        feature.geometry as _,
        feature.links as _,
        feature.stac_version,
        feature.stac_extensions.as_deref(),
        feature.assets as _
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let id = inserted_feature.id.clone().unwrap();

    // read feauture
    let selected_feature =
        sqlx::query_file_as!(Feature, "sql/feature_select.sql", &id, &collection.id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(inserted_feature, selected_feature);

    // update
    let updated_feature = sqlx::query_file_as!(
        Feature,
        "sql/feature_update.sql",
        selected_feature.id,
        selected_feature.collection,
        selected_feature.feature_type as _,
        selected_feature.properties,
        selected_feature.geometry as _,
        selected_feature.links as _,
        selected_feature.stac_version,
        selected_feature.stac_extensions.as_deref(),
        selected_feature.assets as _
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    println!(
        "{}",
        serde_json::to_string_pretty(&updated_feature).unwrap()
    );

    // // delete feature
    // sqlx::query_file_as!(Feature, "sql/feature_delete.sql", &id)
    //     .execute(&pool)
    //     .await
    //     .unwrap();

    // // delete collection
    // sqlx::query_file_as!(Collection, "sql/collection_delete.sql", &collection.id)
    //     .execute(&pool)
    //     .await
    //     .unwrap();

    Ok(())
}
