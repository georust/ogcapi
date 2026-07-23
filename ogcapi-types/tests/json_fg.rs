//! JSON-FG support on the OGC API Features types, behind the `json-fg` feature: the extra
//! members on `Feature`/`FeatureCollection` and the `into_json_fg` conversion helpers.
#![cfg(feature = "json-fg")]

use ogcapi_types::common::media_type::JSON_FG;
use ogcapi_types::features::{Feature, FeatureCollection};
use ogcapi_types::jsonfg::CoordRefSys;

fn geojson_point_feature() -> Feature {
    serde_json::from_value(serde_json::json!({
        "type": "Feature",
        "id": "f1",
        "geometry": { "type": "Point", "coordinates": [155000.0, 463000.0] },
        "properties": {}
    }))
    .unwrap()
}

#[test]
fn media_type_available() {
    assert_eq!(JSON_FG, "application/vnd.ogc.fg+json");
}

#[test]
fn feature_native_crs_goes_to_place() {
    let v =
        serde_json::to_value(geojson_point_feature().into_json_fg(CoordRefSys::from_epsg(28992)))
            .unwrap();
    assert_eq!(v["place"]["type"], "Point");
    assert_eq!(
        v["place"]["coordinates"],
        serde_json::json!([155000.0, 463000.0])
    );
    assert!(v["geometry"].is_null());
    assert_eq!(
        v["coordRefSys"],
        "http://www.opengis.net/def/crs/EPSG/0/28992"
    );
    assert!(
        v["conformsTo"]
            .as_array()
            .unwrap()
            .iter()
            .any(|u| u.as_str().unwrap().contains("json-fg-1/1.0/conf/core"))
    );
}

#[test]
fn feature_wgs84_stays_in_geometry() {
    let v =
        serde_json::to_value(geojson_point_feature().into_json_fg(CoordRefSys::crs84())).unwrap();
    assert_eq!(v["geometry"]["type"], "Point");
    assert!(v.get("place").is_none());
    assert!(v.get("coordRefSys").is_none());
}

#[test]
fn collection_sets_crs_once_and_features_omit_it() {
    let fc = FeatureCollection::new(vec![geojson_point_feature()]);
    let v = serde_json::to_value(fc.into_json_fg(CoordRefSys::from_epsg(28992))).unwrap();
    assert_eq!(
        v["coordRefSys"],
        "http://www.opengis.net/def/crs/EPSG/0/28992"
    );
    // The contained feature inherits the collection CRS and omits its own.
    assert!(v["features"][0].get("coordRefSys").is_none());
    assert!(v["features"][0].get("conformsTo").is_none());
    assert_eq!(v["features"][0]["place"]["type"], "Point");
    assert!(v["features"][0]["geometry"].is_null());
}
