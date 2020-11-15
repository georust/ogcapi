UPDATE
    features
SET
    collection = $2,
    feature_type = $3,
    properties = $4,
    geometry = ST_GeomFromGeoJSON($5),
    links = $6,
    stac_version = $7,
    stac_extensions = $8,
    bbox = $9,
    assets = $10
WHERE
    id = $1
RETURNING
    id AS "id?",
    collection AS "collection?",
    feature_type AS "feature_type: FeatureType",
    properties,
    ST_AsGeoJSON(geometry)::jsonb AS "geometry!: Json<Geometry>",
    links AS "links: Json<Vec<Link>>",
    stac_version,
    stac_extensions,
    bbox,
    assets AS "assets: Json<Assets>"
