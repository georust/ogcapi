SELECT
    id AS "id?",
    collection AS "collection?",
    feature_type AS "feature_type: Json<FeatureType>",
    properties,
    ST_AsGeoJSON(ST_Transform(geom, $2::int))::jsonb as "geometry!: Json<Geometry>",
    links AS "links: Json<Vec<Link>>",
    stac_version,
    stac_extensions,
    ST_AsGeoJSON(ST_Transform(geom, $2::int), 9, 1)::jsonb -> 'bbox' AS "bbox: Json<Bbox>",
    assets AS "assets: Json<Assets>"
FROM
    features
WHERE
    id = $1
