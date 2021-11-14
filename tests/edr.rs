#[async_std::test]
async fn edr() -> tide::Result<()> {
    use std::env;
    use std::path::PathBuf;
    use std::str::FromStr;

    use geojson::{Geometry, Value};
    use sqlx::types::Json;
    use tide::http::{Method, Request, Response, Url};

    use ogcapi::common::crs::Crs;
    use ogcapi::edr::Query;
    use ogcapi::features::FeatureCollection;
    use ogcapi::import::{self, Args};

    // setup app
    dotenv::dotenv().ok();
    let database_url = Url::parse(&env::var("DATABASE_URL")?)?;
    let app = ogcapi::server::server(&database_url).await;

    // load data
    import::ogr::load(
        Args {
            input: PathBuf::from_str("data/ne_10m_admin_0_countries.geojson")?,
            collection: Some("countries".to_string()),
            ..Default::default()
        },
        &database_url,
    )
    .await?;

    import::ogr::load(
        Args {
            input: PathBuf::from_str("data/ne_10m_populated_places.geojson")?,
            collection: Some("places".to_string()),
            ..Default::default()
        },
        &database_url,
    )
    .await?;

    import::ogr::load(
        Args {
            input: PathBuf::from_str("data/ne_10m_railroads.geojson")?,
            collection: Some("railroads".to_string()),
            ..Default::default()
        },
        &database_url,
    )
    .await?;

    // query position
    let mut req = Request::new(
        Method::Get,
        "http://ogcapi.rs/collections/countries/position",
    );
    req.set_query(&Query {
        coords: "POINT(2600000 1200000)".to_string(),
        parameter_name: Some("NAME,ISO_A2,CONTINENT".to_string()),
        crs: Crs::from(2056),
        ..Default::default()
    })?;

    let mut res: Response = app.respond(req).await?;
    assert_eq!(200, res.status());

    let fc: FeatureCollection = res.body_json().await?;
    assert_eq!(fc.number_matched, Some(1));
    assert_eq!(fc.number_returned, Some(1));
    let feature = &fc.features[0];
    assert_eq!(feature.properties.as_ref().unwrap().0.len(), 3);
    assert_eq!(
        feature.properties.as_ref().unwrap().0["NAME"].as_str(),
        Some("Switzerland")
    );

    // query area
    let mut req = Request::new(Method::Get, "http://ogcapi.rs/collections/places/area");
    req.set_query(&Query {
        coords: "POLYGON((7 46, 7 48, 9 48, 9 46, 7 46))".to_string(),
        parameter_name: Some("NAME,ISO_A2,ADM0NAME".to_string()),
        ..Default::default()
    })?;

    let mut res: Response = app.respond(req).await?;
    assert_eq!(200, res.status());

    let fc: FeatureCollection = res.body_json().await?;
    assert_eq!(fc.number_matched, Some(19));
    assert_eq!(fc.number_returned, Some(19));
    let feature = &fc
        .features
        .into_iter()
        .find(|f| f.properties.as_ref().unwrap().0["NAME"].as_str() == Some("Bern"));
    assert!(feature.is_some());

    // query radius
    let mut req = Request::new(Method::Get, "http://ogcapi.rs/collections/countries/radius");
    req.set_query(&Query {
        coords: "POINT(7.5 47)".to_string(),
        parameter_name: Some("NAME,ISO_A2,ADM0NAME".to_string()),
        within: Some("1000".to_string()),
        within_units: Some("km".to_string()),
        ..Default::default()
    })?;

    let mut res: Response = app.respond(req).await?;
    assert_eq!(200, res.status());

    let mut fc: FeatureCollection = res.body_json().await?;
    for mut feature in fc.features.iter_mut() {
        feature.geometry = Json(Geometry::new(Value::Point(vec![0.0, 0.0])));
    }

    tide::log::debug!("{}", serde_json::to_string_pretty(&fc.number_matched)?);
    // assert_eq!(features.number_matched, Some(19));
    // assert_eq!(features.number_returned, Some(19));
    // let feature = &features
    //     .features
    //     .into_iter()
    //     .find(|f| f.properties.as_ref().unwrap().0["NAME"].as_str() == Some("Bern"));
    // assert!(feature.is_some());

    Ok(())
}
