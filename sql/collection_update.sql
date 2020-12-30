UPDATE
    collections
SET
    title = $2,
    description = $3,
    links = $4,
    extent = $5,
    item_type = $6,
    crs = $7,
    storage_crs = $8,
    storage_crs_coordinate_epoch = $9,
    stac_version = $10,
    stac_extensions = $11,
    keywords = $12,
    licence = $13,
    providers = $14,
    summaries = $15
WHERE
    id = $1
RETURNING
    id,
    title,
    description,
    links AS "links: Json<Vec<Link>>",
    extent AS "extent: Json<Extent>",
    item_type AS "item_type: Json<ItemType>",
    crs,
    storage_crs,
    storage_crs_coordinate_epoch,
    stac_version,
    stac_extensions,
    keywords,
    licence,
    providers AS "providers: Json<Vec<Provider>>",
    summaries AS "summaries: Json<Summaries>"
