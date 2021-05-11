INSERT INTO features (
    id,
    collection,
    feature_type,
    properties,
    geom
    ) VALUES ($1, $2, '"Feature"', $3, ST_GeomFromWKB($4, 4326))
