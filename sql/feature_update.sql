UPDATE
    features
SET
    collection = $2,
    feature_type = $3,
    properties = $4,
    geom = ST_GeomFromGeoJSON($5),
    links = $6,
    stac_version = $7,
    stac_extensions = $8,
    assets = $9
WHERE
    id = $1
RETURNING
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
