use std::net::{SocketAddr, TcpListener};

use axum::{
    headers::{authorization::Credentials, Authorization},
    http::Request,
};
use hyper::Body;
use serde_json::json;
use url::Url;
use uuid::Uuid;

use ogcapi_drivers::postgres::Db;
use ogcapi_types::{
    common::{link_rel::SELF, media_type::JSON, Collection, Crs, Link},
    features::Feature,
};

async fn spawn_app() -> anyhow::Result<SocketAddr> {
    dotenv::dotenv().ok();

    // tracing_subscriber::fmt::init();

    let database_url = Url::parse(&std::env::var("DATABASE_URL")?)?;

    let db = Db::setup_with(&database_url, &Uuid::new_v4().to_string(), true)
        .await
        .expect("Setup database");

    let app = ogcapi_services::app(db).await;

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>()?)?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .expect("")
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(addr)
}

#[tokio::test]
async fn minimal_feature_crud() -> anyhow::Result<()> {
    // setup app
    let addr = spawn_app().await?;
    let client = hyper::Client::new();
    let credentials = Authorization::basic("user", "password").0.encode();

    let collection = Collection {
        id: "test".to_string(),
        links: vec![Link::new("http://localhost:8080/collections/test", SELF)],
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
                .header("Authorization", &credentials)
                .body(Body::from(serde_json::to_string(&collection)?))?,
        )
        .await?;

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
    }))?;

    // create feature
    let res = client
        .request(
            Request::builder()
                .method(axum::http::Method::POST)
                .uri(format!("http://{}/collections/test/items", addr))
                .header("Content-Type", JSON.to_string())
                .header("Authorization", &credentials)
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
                .uri(format!("http://{}/collections/test/items/{}", addr, &id).as_str())
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
                .uri(format!("http://{}/collections/test/items/{}", addr, &id).as_str())
                .header("Authorization", &credentials)
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
                .header("Authorization", &credentials)
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(204, res.status());

    Ok(())
}
