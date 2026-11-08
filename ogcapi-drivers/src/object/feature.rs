use anyhow::Context;
use chrono::{DateTime, Utc};
use futures::{StreamExt as _, TryStreamExt as _};
use object_store::{ObjectStoreExt as _, PutMode, path::Path};
use serde_json::Value;
use uuid::Uuid;

use ogcapi_types::{
    common::{
        Bbox, Crs, Exception, Link, Projection,
        link_rel::{COLLECTION, PARENT, ROOT},
        media_type::JSON,
    },
    features::{Feature, FeatureCollection, FeatureId, Geometry, Query},
};

use crate::FeatureTransactions;

use super::{
    ObjectDriver, answered_links, collection_part, collection_path, feature_path, invalid_id,
    not_found, stored_links,
};

#[async_trait::async_trait]
impl FeatureTransactions for ObjectDriver {
    async fn create_feature(&self, feature: &Feature) -> anyhow::Result<String> {
        let collection = feature
            .collection
            .as_deref()
            .context("feature.collection is required")?;
        self.collection_required(collection).await?;

        let mut feature = feature.clone();
        let id = feature
            .id
            .get_or_insert_with(|| FeatureId::String(Uuid::new_v4().to_string()))
            .to_string();
        let location = feature_path(collection, &id).ok_or_else(|| invalid_id(&id))?;

        let payload = serde_json::to_vec(&stored(&feature))?.into();
        match self.write(&location, payload, PutMode::Create).await? {
            Some(_) => Ok(id),
            None => Err(Exception::new_from_status(409)
                .detail(format!("a feature `{id}` exists"))
                .into()),
        }
    }

    async fn read_feature(
        &self,
        collection: &str,
        id: &str,
        crs: &Crs,
    ) -> anyhow::Result<Option<Feature>> {
        let projection = projection(crs)?;
        let Some(location) = feature_path(collection, id) else {
            return Ok(None);
        };
        let mut feature: Option<Feature> = self.read(&location).await?;
        if let Some(feature) = &mut feature {
            answered_links(&mut feature.links);
            if let Some(geometry) = &mut feature.geometry {
                project(geometry, projection);
            }
        }
        Ok(feature)
    }

    async fn update_feature(&self, feature: &Feature) -> anyhow::Result<()> {
        let collection = feature
            .collection
            .as_deref()
            .context("feature.collection is required")?;
        let id = feature
            .id
            .as_ref()
            .context("feature.id is required")?
            .to_string();
        let location = feature_path(collection, &id).ok_or_else(not_found)?;
        if !self.exists(&location).await? {
            return Err(not_found());
        }
        self.store
            .put(&location, serde_json::to_vec(&stored(feature))?.into())
            .await?;
        Ok(())
    }

    async fn delete_feature(&self, collection: &str, id: &str) -> anyhow::Result<()> {
        let location = feature_path(collection, id).ok_or_else(not_found)?;
        if !self.exists(&location).await? {
            return Err(not_found());
        }
        self.store.delete(&location).await?;
        Ok(())
    }

    async fn list_items(
        &self,
        collection_id: &str,
        query: &Query,
    ) -> anyhow::Result<FeatureCollection> {
        self.collection_required(collection_id).await?;
        let collection = collection_part(collection_id).ok_or_else(not_found)?;

        if query.filter.is_some() {
            return Err(Exception::new_from_status(400)
                .detail("no `filter` is taken here")
                .into());
        }
        let projection = projection(query.crs.as_ref().unwrap_or(&Crs::default2d()))?;
        let bbox = match &query.bbox {
            None => None,
            Some(bbox) => {
                let bbox_crs = query.bbox_crs.clone().unwrap_or_else(|| match bbox {
                    Bbox::Bbox2D(_) => Crs::default2d(),
                    Bbox::Bbox3D(_) => Crs::default3d(),
                });
                Some(bbox.to_crs84(&bbox_crs).ok_or_else(|| {
                    Exception::new_from_status(400).detail(
                        "a bbox here is in CRS84 or Web Mercator, within its extent, \
                        its south not north of its north",
                    )
                })?)
            }
        };

        // the features' ids, their directories', in order, for pages that
        // follow each other
        let listed = self
            .store
            .list_with_delimiter(Some(&Path::from_iter([collection])))
            .await?;
        let mut ids: Vec<String> = listed
            .common_prefixes
            .iter()
            .filter_map(Path::filename)
            .map(str::to_owned)
            .collect();
        ids.sort();

        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(usize::MAX);
        let selects =
            bbox.is_some() || query.datetime.is_some() || !query.additional_parameters.is_empty();
        let (number_matched, mut features) = if selects {
            // every feature is read to select from
            let features = self.read_features(collection_id, &ids).await?;
            let selected: Vec<Feature> = features
                .into_iter()
                .filter(|feature| selected(feature, bbox.as_ref(), query))
                .collect();
            let number_matched = selected.len();
            let page = selected.into_iter().skip(offset).take(limit).collect();
            (number_matched, page)
        } else {
            let page: Vec<String> = ids.iter().skip(offset).take(limit).cloned().collect();
            (ids.len(), self.read_features(collection_id, &page).await?)
        };

        for geometry in features.iter_mut().filter_map(|f| f.geometry.as_mut()) {
            project(geometry, projection);
        }
        let mut fc = FeatureCollection::new(features);
        fc.number_matched = Some(u64::try_from(number_matched)?);
        Ok(fc)
    }
}

impl ObjectDriver {
    /// The refusal of a collection that doesn't exist.
    async fn collection_required(&self, collection: &str) -> anyhow::Result<()> {
        let collection = collection_part(collection).ok_or_else(not_found)?;
        if self.exists(&collection_path(collection)).await? {
            Ok(())
        } else {
            Err(not_found())
        }
    }

    /// The features of `ids`, read at once; one deleted meanwhile is left out.
    async fn read_features(
        &self,
        collection: &str,
        ids: &[String],
    ) -> anyhow::Result<Vec<Feature>> {
        // owned ids, so that the closure's futures borrow nothing of a lifetime
        // of their own, as a `Send` future of the trait needs
        let features: Vec<Option<Feature>> = futures::stream::iter(ids.to_vec())
            .map(|id| async move {
                match feature_path(collection, &id) {
                    Some(location) => self.read::<Feature>(&location).await,
                    None => Ok(None),
                }
            })
            .buffered(16)
            .try_collect()
            .await?;
        let mut features: Vec<Feature> = features.into_iter().flatten().collect();
        for feature in &mut features {
            answered_links(&mut feature.links);
        }
        Ok(features)
    }
}

/// A feature as stored, linking the root's catalog as its root and its
/// collection's description as its parent and collection.
fn stored(feature: &Feature) -> Feature {
    let link = |href: &str, rel: &str| Link::new(href, rel).mediatype(JSON);
    Feature {
        links: stored_links(
            &feature.links,
            [
                link("../../catalog.json", ROOT),
                link("../collection.json", PARENT),
                link("../collection.json", COLLECTION),
            ],
        ),
        ..feature.clone()
    }
}

/// The projection a response's `crs` asks for; a refusal of a system other
/// than CRS84 and Web Mercator, which the features, in CRS84, are not
/// transformed to here.
fn projection(crs: &Crs) -> anyhow::Result<Projection> {
    crs.projection().ok_or_else(|| {
        Exception::new_from_status(400)
            .detail(format!(
                "features here are in CRS84 or Web Mercator, not {crs}"
            ))
            .into()
    })
}

/// Transform a geometry in CRS84 by `projection`; its box no longer holds.
fn project(geometry: &mut Geometry, projection: Projection) {
    use geojson::{GeometryValue, Position};

    if projection == Projection::Crs84 {
        return;
    }
    let forward = |position: &mut Position| {
        if let [x, y, ..] = position.as_slice_mut() {
            (*x, *y) = projection.forward(*x, *y);
        }
    };
    match &mut geometry.value {
        GeometryValue::Point { coordinates } => forward(coordinates),
        GeometryValue::MultiPoint { coordinates } | GeometryValue::LineString { coordinates } => {
            coordinates.iter_mut().for_each(forward);
        }
        GeometryValue::MultiLineString { coordinates } | GeometryValue::Polygon { coordinates } => {
            coordinates.iter_mut().flatten().for_each(forward);
        }
        GeometryValue::MultiPolygon { coordinates } => {
            coordinates.iter_mut().flatten().flatten().for_each(forward);
        }
        GeometryValue::GeometryCollection { geometries } => {
            for geometry in geometries {
                project(geometry, projection);
            }
        }
    }
    geometry.bbox = None;
}

/// Whether a feature is among those a query selects, as the Postgres
/// driver selects them: a feature without a geometry meets any box, one
/// without a time any `datetime`, and one without a property any value of it.
fn selected(feature: &Feature, bbox: Option<&Bbox>, query: &Query) -> bool {
    let in_box = bbox.is_none_or(|bbox| {
        feature
            .geometry
            .as_ref()
            .and_then(Bbox::of_geometry)
            .is_none_or(|extent| extent.intersects(bbox))
    });
    let in_time = query.datetime.as_ref().is_none_or(|datetime| {
        time(feature).is_none_or(|(start, end)| datetime.intersects(start, end))
    });
    let properties = feature.properties.as_ref();
    let of_values = query.additional_parameters.iter().all(|(key, value)| {
        match properties.and_then(|properties| properties.get(key)) {
            None => true,
            Some(Value::Null) => false,
            Some(Value::String(text)) => text == value,
            // a value of another type compared as JSON: `1.0` and `1`, `true`
            Some(other) => serde_json::from_str::<Value>(value).is_ok_and(|value| value == *other),
        }
    });
    in_box && in_time && of_values
}

/// The time of a feature: its property `datetime`, or the interval from
/// `start_datetime` to `end_datetime`.
fn time(feature: &Feature) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let properties = feature.properties.as_ref()?;
    let at = |key: &str| properties.get(key)?.as_str()?.parse::<DateTime<Utc>>().ok();
    match at("datetime") {
        Some(instant) => Some((instant, instant)),
        None => Some((at("start_datetime")?, at("end_datetime")?)),
    }
}

#[cfg(test)]
mod tests {
    use ogcapi_types::{
        common::{Bbox, Collection, Crs, Exception},
        features::{Feature, Query},
    };

    use crate::{CollectionTransactions, FeatureTransactions};

    use super::ObjectDriver;

    fn status(error: &anyhow::Error) -> Option<u16> {
        error.downcast_ref::<Exception>().and_then(|e| e.status)
    }

    fn feature(id: &str, x: f64, datetime: &str) -> Feature {
        serde_json::from_value(serde_json::json!({
            "type": "Feature",
            "id": id,
            "collection": "places",
            "geometry": { "type": "Point", "coordinates": [x, 0.0] },
            "properties": { "datetime": datetime, "name": id }
        }))
        .unwrap()
    }

    async fn places() -> ObjectDriver {
        let driver = ObjectDriver::default();
        driver
            .create_collection(&Collection::new("places"))
            .await
            .unwrap();
        for (i, x) in [0.0, 10.0, 20.0, 30.0].into_iter().enumerate() {
            let datetime = format!("2026-11-0{}T00:00:00Z", i + 1);
            driver
                .create_feature(&feature(&format!("p{i}"), x, &datetime))
                .await
                .unwrap();
        }
        driver
    }

    #[tokio::test]
    async fn features_are_created_read_updated_and_deleted() {
        let driver = places().await;

        let error = driver
            .create_feature(&feature("p0", 0.0, "2026-11-01T00:00:00Z"))
            .await
            .unwrap_err();
        assert_eq!(status(&error), Some(409));

        let mut unnamed = feature("x", 5.0, "2026-11-08T00:00:00Z");
        unnamed.id = None;
        let id = driver.create_feature(&unnamed).await.unwrap();
        assert!(
            driver
                .read_feature("places", &id, &Crs::default2d())
                .await
                .unwrap()
                .is_some()
        );

        let mut moved = feature("p1", 11.0, "2026-11-02T00:00:00Z");
        driver.update_feature(&moved).await.unwrap();
        let read = driver
            .read_feature("places", "p1", &Crs::default2d())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(read.geometry, moved.geometry);

        moved.id = Some(ogcapi_types::features::FeatureId::String(
            "none".to_string(),
        ));
        let error = driver.update_feature(&moved).await.unwrap_err();
        assert_eq!(status(&error), Some(404));

        driver.delete_feature("places", "p1").await.unwrap();
        let error = driver.delete_feature("places", "p1").await.unwrap_err();
        assert_eq!(status(&error), Some(404));

        // no feature in a collection that doesn't exist
        let mut lost = feature("p9", 0.0, "2026-11-01T00:00:00Z");
        lost.collection = Some("none".to_string());
        let error = driver.create_feature(&lost).await.unwrap_err();
        assert_eq!(status(&error), Some(404));
        assert!(driver.list_collections(&Default::default()).await.is_ok());
    }

    #[tokio::test]
    async fn features_are_listed_by_page_and_selected() {
        let driver = places().await;

        let page = |offset, limit| Query {
            offset: Some(offset),
            limit: Some(limit),
            ..Default::default()
        };
        let fc = driver.list_items("places", &page(1, 2)).await.unwrap();
        let ids: Vec<String> = fc
            .features
            .iter()
            .map(|f| f.id.as_ref().unwrap().to_string())
            .collect();
        assert_eq!(ids, ["p1", "p2"]);
        assert_eq!(fc.number_matched, Some(4));
        assert_eq!(fc.number_returned, Some(2));

        let boxed = Query {
            bbox: Some(Bbox::Bbox2D([5.0, -1.0, 25.0, 1.0])),
            ..Default::default()
        };
        let fc = driver.list_items("places", &boxed).await.unwrap();
        assert_eq!(fc.number_matched, Some(2));

        let timed = Query {
            datetime: Some("2026-11-03T00:00:00Z/..".parse().unwrap()),
            ..Default::default()
        };
        assert_eq!(
            driver
                .list_items("places", &timed)
                .await
                .unwrap()
                .number_matched,
            Some(2)
        );

        let mut named = Query::default();
        named
            .additional_parameters
            .insert("name".to_string(), "p3".to_string());
        assert_eq!(
            driver
                .list_items("places", &named)
                .await
                .unwrap()
                .number_matched,
            Some(1)
        );

        let error = driver
            .list_items("none", &Query::default())
            .await
            .unwrap_err();
        assert_eq!(status(&error), Some(404));
    }

    #[tokio::test]
    async fn features_are_answered_in_web_mercator() {
        let driver = places().await;

        let read = driver
            .read_feature("places", "p1", &Crs::from_epsg(3857))
            .await
            .unwrap()
            .unwrap();
        let geojson::GeometryValue::Point { coordinates } = read.geometry.unwrap().value else {
            panic!("a point");
        };
        assert!((coordinates.as_slice()[0] - 1_113_194.9).abs() < 0.1);

        let error = driver
            .read_feature("places", "p1", &Crs::from_epsg(2056))
            .await
            .unwrap_err();
        assert_eq!(status(&error), Some(400));
    }

    #[tokio::test]
    async fn a_directory_of_the_file_system_is_a_static_catalog() {
        let directory = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let url = url::Url::from_directory_path(&directory).unwrap();
        let driver = ObjectDriver::from_url(&url).unwrap();
        driver
            .create_collection(&Collection::new("places"))
            .await
            .unwrap();
        driver
            .create_feature(&feature("p0", 0.0, "2026-11-01T00:00:00Z"))
            .await
            .unwrap();

        for file in [
            "catalog.json",
            "places/collection.json",
            "places/p0/p0.json",
        ] {
            assert!(directory.join(file).is_file(), "{file}");
        }
        let fc = driver
            .list_items("places", &Query::default())
            .await
            .unwrap();
        assert_eq!(fc.number_matched, Some(1));
        assert_eq!(fc.features[0].links, []);

        // a feature's directory goes with it
        driver.delete_feature("places", "p0").await.unwrap();
        assert!(!directory.join("places/p0").exists());
        let fc = driver
            .list_items("places", &Query::default())
            .await
            .unwrap();
        assert_eq!(fc.number_matched, Some(0));

        std::fs::remove_dir_all(directory).unwrap();
    }
}
