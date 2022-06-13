mod setup;

use axum::http::Request;
use hyper::Body;
use serde_json::json;

use ogcapi_types::{
    common::{media_type::JSON, Collection, Crs},
    features::Feature,
};

#[tokio::test]
async fn minimal_feature_crud() -> anyhow::Result<()> {
    // setup app
    let (addr, _) = setup::spawn_app().await?;
    let client = hyper::Client::new();

    let collection = Collection {
        id: "test.me-_".to_string(),
        links: vec![],
        crs: vec![Crs::default()],
        ..Default::default()
    };

    // create collection
    let res = client
        .request(
            Request::builder()
                .method(axum::http::Method::POST)
                .uri(format!("http://{}/collections", addr))
                .header("Content-Type", JSON)
                .body(Body::from(serde_json::to_string(&collection)?))?,
        )
        .await?;

    let (parts, _body) = res.into_parts();

    assert_eq!(201, parts.status);
    println!("{:#?}", parts.headers.get("Location"));

    let feature: Feature = serde_json::from_value(json!({
        "collection": collection.id,
        "type": "Feature",
        "geometry": {
            "type": "Point",
            "coordinates": [7.428959, 1.513394]
        },
        "links": [{
            "href": "https://localhost:8080/collections/test/items/{id}",
            "rel": "self"
        }]
    }))?;

    // create feature
    let res = client
        .request(
            Request::builder()
                .method(axum::http::Method::POST)
                .uri(format!(
                    "http://{}/collections/{}/items",
                    addr, collection.id
                ))
                .header("Content-Type", JSON.to_string())
                .body(Body::from(serde_json::to_string(&feature)?))?,
        )
        .await?;

    assert_eq!(201, res.status());

    let location = res.headers().get("Location").unwrap().to_str()?;
    println!("{}", location);

    let id = location.split('/').last().unwrap();

    // read feauture
    let res = client
        .request(
            Request::builder()
                .method(axum::http::Method::GET)
                .uri(
                    format!(
                        "http://{}/collections/{}/items/{}",
                        addr, collection.id, &id
                    )
                    .as_str(),
                )
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(200, res.status());
    let body = hyper::body::to_bytes(res.into_body()).await?;
    let _feature: Feature = serde_json::from_slice(&body)?;
    // println!("{:#?}", feature);

    // delete feature
    let res = client
        .request(
            Request::builder()
                .method(axum::http::Method::DELETE)
                .uri(
                    format!(
                        "http://{}/collections/{}/items/{}",
                        addr, collection.id, &id
                    )
                    .as_str(),
                )
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(204, res.status());

    // delete collection
    let res = client
        .request(
            Request::builder()
                .method(axum::http::Method::DELETE)
                .uri(format!("http://{}/collections/{}", addr, &collection.id).as_str())
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(204, res.status());

    Ok(())
}
