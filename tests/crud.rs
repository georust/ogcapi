#[async_std::test]
async fn minimal_feature_crud() -> tide::Result<()> {
    use std::env;

    use serde_json::json;
    use tide::http::{Method, Request, Response, Url};

    use ogcapi::common::{collections::Collection, core::Link, crs::Crs};
    use ogcapi::features::Feature;

    // setup app
    dotenv::dotenv().ok();

    let host = env::var("OGCAPI_HOST")?;
    let port = env::var("OGCAPI_PORT")?;

    let app = ogcapi::server::server(&Url::parse(&env::var("DATABASE_URL")?)?).await;

    let collection = Collection {
        id: "test".to_string(),
        links: vec![Link::new(
            Url::parse("http://localhost:8080/collections/test").unwrap(),
        )],
        crs: Some(vec![Crs::default()]),
        ..Default::default()
    };

    // create collection
    let mut req = Request::new(
        Method::Post,
        format!("http://{}:{}/collections", &host, &port).as_str(),
    );
    req.set_body(serde_json::to_string(&collection)?);
    let res: Response = app.respond(req).await?;
    assert_eq!(201, res.status());
    println!("{:#?}", res.header("Location"));

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
    let mut req = Request::new(
        Method::Post,
        format!("http://{}:{}/collections/test/items", &host, &port).as_str(),
    );
    req.set_body(serde_json::to_string(&feature)?);
    let res: Response = app.respond(req).await?;
    assert_eq!(201, res.status());

    let location = res.header("Location").unwrap()[0].to_string();
    println!("{}", location);

    let id = location.split("/").last().unwrap();

    // read feauture
    let req = Request::new(
        Method::Get,
        format!("http://{}:{}/collections/test/items/{}", &host, &port, &id).as_str(),
    );
    let mut res: Response = app.respond(req).await?;
    assert_eq!(200, res.status());
    let _feature: Feature = res.body_json().await?;
    // println!("{:#?}", feature);

    // update
    // db.update_feature(&feature).await?;

    // delete feature
    let req = Request::new(
        Method::Delete,
        format!("http://{}:{}/collections/test/items/{}", &host, &port, &id).as_str(),
    );
    let res: Response = app.respond(req).await?;
    assert_eq!(204, res.status());

    // delete collection
    let req = Request::new(
        Method::Delete,
        format!("http://{}:{}/collections/{}", &host, &port, &collection.id).as_str(),
    );
    let res: Response = app.respond(req).await?;
    assert_eq!(204, res.status());

    Ok(())
}
