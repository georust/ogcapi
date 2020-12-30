SELECT
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
FROM
    collections
WHERE
    id = $1
