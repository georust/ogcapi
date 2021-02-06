INSERT INTO features (
    collection,
    feature_type,
    properties,
    geom,
    links,
    stac_version,
    stac_extensions,
    assets
    ) VALUES ($1, $2, $3, ST_SetSRID(ST_GeomFromGeoJSON($4),4326), $5, $6, $7, $8)
RETURNING
    id AS "id?",
    collection AS "collection?",
    feature_type AS "feature_type: Json<FeatureType>",
    properties,
    ST_AsGeoJSON (geom)::jsonb AS "geometry!: Json<Geometry>",
    links AS "links: Json<Vec<Link>>",
    stac_version,
    stac_extensions,
    bbox AS "bbox: Json<Bbox>",
    assets AS "assets: Json<Assets>"
