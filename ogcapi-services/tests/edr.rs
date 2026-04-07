mod setup;

#[ignore]
#[cfg(feature = "edr")]
#[tokio::test]
async fn edr() -> anyhow::Result<()> {
    use ogcapi_client::Client;
    use ogcapi_types::{common::Crs, edr::Query, features::FeatureCollection};
    use url::Url;

    let (addr, _database_url) = setup::spawn_app().await?;

    let public_url = Url::parse(&format!("http://{addr}"))?;

    // load data
    let input = "../data/ne_110m_admin_0_countries.geojson";
    data_loader::geojson::client::load(input, "countries", None, &public_url).await?;

    let input = "../data/ne_110m_populated_places.geojson";
    data_loader::geojson::client::load(input, "places", None, &public_url).await?;

    // let input = "../data/ne_10m_railroads_north_america.geojson";
    // data_loader::geojson::db::load_with_client(input, "railroads", None, &public_url).await?;

    let client = Client::new(public_url)?;

    // query position
    let query = Query {
        coords: "POINT(2600000 1200000)".to_string(),
        parameter_name: Some("NAME,ISO_A2,CONTINENT".to_string()),
        crs: Some(Crs::from_epsg(2056)),
        ..Default::default()
    };

    let url = format!(
        "http://{}/collections/countries/position?{}",
        addr,
        serde_qs::to_string(&query)?
    );

    let fc: FeatureCollection = client.fetch(&url).await?;

    assert_eq!(fc.number_matched, Some(1));
    assert_eq!(fc.number_returned, Some(1));
    let feature = &fc.features[0];
    assert_eq!(feature.properties.len(), 3);
    assert_eq!(feature.properties["NAME"].as_str(), Some("Switzerland"));

    // query area
    let query = Query {
        coords: "POLYGON((6 45, 6 49, 9 49, 9 45, 6 45))".to_string(),
        parameter_name: Some("NAME,ISO_A2,ADM0NAME".to_string()),
        ..Default::default()
    };

    let url = format!(
        "http://{}/collections/places/area?{}",
        addr,
        serde_qs::to_string(&query)?
    );
    let fc: FeatureCollection = client.fetch(&url).await?;

    assert_eq!(fc.number_matched, Some(2));
    assert_eq!(fc.number_returned, Some(2));
    let feature = &fc
        .features
        .into_iter()
        .find(|f| f.properties["NAME"].as_str() == Some("Bern"));
    assert!(feature.is_some());

    // query radius
    let query = Query {
        coords: "POINT(7.5 47)".to_string(),
        parameter_name: Some("NAME,ISO_A2,ADM0NAME".to_string()),
        within: Some("1000".to_string()),
        within_units: Some("km".to_string()),
        ..Default::default()
    };

    let url = format!(
        "http://{}/collections/places/radius?{}",
        addr,
        serde_qs::to_string(&query)?
    );
    let fc: FeatureCollection = client.fetch(&url).await?;
    println!("{}", serde_json::to_string_pretty(&fc.number_matched)?);
    assert_eq!(fc.number_matched, Some(22));
    assert_eq!(fc.number_returned, Some(22));
    let feature = &fc
        .features
        .into_iter()
        .find(|f| f.properties["NAME"].as_str() == Some("Bern"));
    assert!(feature.is_some());

    Ok(())
}
