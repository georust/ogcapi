use std::net::{SocketAddr, TcpListener};

use http::Request;
use hyper::Body;
use serde_json::json;
use url::Url;
use uuid::Uuid;

use ogcapi::common::collections::Collection;
use ogcapi::common::core::{Link, MediaType};
use ogcapi::common::crs::Crs;
use ogcapi::db::Db;
use ogcapi::features::Feature;

async fn spawn_app() -> SocketAddr {
    dotenv::dotenv().ok();

    // tracing_subscriber::fmt::init();

    let database_url = Url::parse(&std::env::var("DATABASE_URL").unwrap()).unwrap();

    let db = Db::setup_with(&database_url, &Uuid::new_v4().to_string(), true)
        .await
        .expect("Setup database");

    let app = ogcapi::server::server(db).await;

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .expect("")
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    addr
}

#[tokio::test]
async fn minimal_feature_crud() -> anyhow::Result<()> {
    // setup app
    let addr = spawn_app().await;
    let client = hyper::Client::new();

    let collection = Collection {
        id: "test".to_string(),
        links: vec![Link::new("http://localhost:8080/collections/test")],
        crs: Some(vec![Crs::default()]),
        ..Default::default()
    };

    // create collection
    let res = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://{}/collections", addr))
                .header("Content-Type", MediaType::JSON.to_string())
                .body(Body::from(serde_json::to_string(&collection)?))
                .unwrap(),
        )
        .await
        .unwrap();

    let (parts, _body) = res.into_parts();

    assert_eq!(201, parts.status);
    println!("{:#?}", parts.headers.get("Location"));

    let feature: Feature = serde_json::from_value(json!({
        "collection": "test",
        "type": "Feature",
        "geometry": {
            "type": "Point",
            "coordinates": [7.428959, 1.513394]
        },
        "links": [{
            "href": "https://localhost:8080/collections/test/items/{id}",
            "rel": "self"
        }]
    }))
    .unwrap();

    // create feature
    let res = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://{}/collections/test/items", addr))
                .header("Content-Type", MediaType::JSON.to_string())
                .body(Body::from(serde_json::to_string(&feature)?))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(201, res.status());

    let location = res.headers().get("Location").unwrap().to_str()?;
    println!("{}", location);

    let id = location.split('/').last().unwrap();

    // read feauture
    let res = client
        .request(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("http://{}/collections/test/items/{}", addr, &id).as_str())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(200, res.status());
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let _feature: Feature = serde_json::from_slice(&body)?;
    // println!("{:#?}", feature);

    // update
    // db.update_feature(&feature).await?;

    // delete feature
    let res = client
        .request(
            Request::builder()
                .method(http::Method::DELETE)
                .uri(format!("http://{}/collections/test/items/{}", addr, &id).as_str())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(204, res.status());

    // delete collection
    let res = client
        .request(
            Request::builder()
                .method(http::Method::DELETE)
                .uri(format!("http://{}/collections/{}", addr, &collection.id).as_str())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(204, res.status());

    Ok(())
}
