INSERT INTO features (
    collection,
    feature_type,
    properties,
    geometry,
    links,
    stac_version,
    stac_extensions,
    bbox,
    assets
    ) VALUES ($1, $2, $3, ST_GeomFromGeoJSON($4), $5, $6, $7, $8, $9)
RETURNING
    id AS "id?",
    collection AS "collection?",
    feature_type AS "feature_type: FeatureType",
    properties,
    ST_AsGeoJSON (geometry)::jsonb AS "geometry!: Json<Geometry>",
    links AS "links: Json<Vec<Link>>",
    stac_version,
    stac_extensions,
    bbox,
    assets AS "assets: Json<Assets>"
