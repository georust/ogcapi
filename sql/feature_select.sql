SELECT
    id AS "id?",
    collection AS "collection?",
    feature_type AS "feature_type: Json<FeatureType>",
    properties,
    ST_AsGeoJSON(geom)::jsonb AS "geometry!: Json<Geometry>",
    links AS "links: Json<Vec<Link>>",
    stac_version,
    stac_extensions,
    bbox AS "bbox: Json<Bbox>",
    assets AS "assets: Json<Assets>"
FROM
    features
WHERE
    id = $1
