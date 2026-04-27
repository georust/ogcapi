mod setup;

#[ignore]
#[cfg(feature = "edr")]
#[tokio::test]
async fn edr() -> anyhow::Result<()> {
    use data_loader::Args;
    use ogcapi_client::Client;
    use ogcapi_types::{common::Crs, edr::Query, features::FeatureCollection};

    let (addr, database_url) = setup::spawn_app().await?;

    let client = Client::new(format!("http://{addr}"))?;

    // load data
    let args = Args::new(
        "../data/ne_110m_admin_0_countries.geojson",
        "countries",
        &database_url,
    );
    data_loader::geojson::load(args).await?;

    let args = Args::new(
        "../data/ne_110m_populated_places.geojson",
        "places",
        &database_url,
    );
    data_loader::geojson::load(args).await?;

    // data_loader::geojson::load(
    //     Args {
    //         input: PathBuf::from_str("../data/ne_10m_railroads_north_america.geojson")?,
    //         collection: "railroads".to_string(),
    //         ..Default::default()
    //     },
    //     &database_url,
    //     false,
    // )
    // .await?;

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
    assert_eq!(feature.properties.as_ref().unwrap().len(), 3);
    assert_eq!(
        feature.properties.as_ref().unwrap()["NAME"].as_str(),
        Some("Switzerland")
    );

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
        .find(|f| f.properties.as_ref().unwrap()["NAME"].as_str() == Some("Bern"));
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
        .find(|f| f.properties.as_ref().unwrap()["NAME"].as_str() == Some("Bern"));
    assert!(feature.is_some());

    Ok(())
}
