mod setup;

#[cfg(feature = "features")]
#[tokio::test]
async fn minimal_feature_crud() -> anyhow::Result<()> {
    use geojson::Geometry;

    use ogcapi_client::Client;
    use ogcapi_types::{
        common::{Collection, link_rel::SELF},
        features::Feature,
    };

    // setup app
    let (addr, _) = setup::spawn_app().await?;

    let client = Client::new(format!("http://{addr}"))?;

    // create collection
    let collection = Collection::new("test.me-_");
    let location = client.create_collection(&collection).await?;
    println!("Location: {location}");

    // create feature
    let feature: Feature = Feature::new(Geometry::new_point([7.428959, 1.513394]));

    let location = client.create_item(&collection.id, &feature).await?;
    println!("Location: {location}");

    // read feauture
    let feature_id = location.split('/').next_back().unwrap();

    let item = client.item(&collection.id, feature_id).await?;
    assert_eq!(
        item.id.map(|id| id.to_string()),
        Some(feature_id.to_string())
    );
    assert_eq!(item.collection.as_ref(), Some(&collection.id));

    let itself = item.links.iter().find(|l| l.rel == SELF);
    assert_eq!(itself.map(|l| &l.href), Some(&location));

    // delete feature
    client.delete_item(&collection.id, feature_id).await?;

    // delete collection
    client.delete_collection(&collection.id).await?;

    Ok(())
}
