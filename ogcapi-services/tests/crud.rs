mod setup;

#[cfg(feature = "features")]
#[tokio::test]
async fn minimal_feature_crud() -> anyhow::Result<()> {
    use axum::{
        body::Body,
        http::{Method, Request},
    };
    use http_body_util::BodyExt;
    use hyper_util::{client::legacy::Client, rt::TokioExecutor};
    use serde_json::json;

    use ogcapi_types::{
        common::{Collection, media_type::JSON},
        features::Feature,
    };

    // setup app
    let (addr, _) = setup::spawn_app().await?;
    let client = Client::builder(TokioExecutor::new()).build_http();

    // create collection
    let collection = Collection::new("test.me-_");
    let res = client
        .request(
            Request::builder()
                .method(Method::POST)
                .uri(format!("http://{addr}/collections"))
                .header("Content-Type", JSON)
                .body(Body::from(serde_json::to_string(&collection)?))?,
        )
        .await?;

    let (parts, _body) = res.into_parts();

    assert_eq!(201, parts.status);
    // println!("{:#?}", parts.headers.get("Location"));

    let feature: Feature = serde_json::from_value(json!({
        "collection": collection.id,
        "type": "Feature",
        "geometry": {
            "type": "Point",
            "coordinates": [7.428959, 1.513394]
        },
    }))?;

    // create feature
    let res = client
        .request(
            Request::builder()
                .method(Method::POST)
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
    // println!("{location}");

    // read feauture
    let res = client
        .request(
            Request::builder()
                .method(Method::GET)
                .uri(location)
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(200, res.status());
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let _feature: Feature = serde_json::from_slice(&body)?;
    // println!("{:#?}", feature);

    // delete feature
    let res = client
        .request(
            Request::builder()
                .method(Method::DELETE)
                .uri(location)
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(204, res.status());

    // delete collection
    let res = client
        .request(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("http://{}/collections/{}", addr, &collection.id).as_str())
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(204, res.status());

    Ok(())
}
