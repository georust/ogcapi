# OGC API Client

## Client

The `ogcapi_client` crate provides a client for accessing geospatial datasets served through [OGC API](https://ogcapi.ogc.org/) or [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) with the following features:

* Depth first iterator over catalog tree
* Iterator over collections
* Item search
* Lazy pagination handling

## Example

```rust
use ogcapi_client::Client;

// Setup a client for a given STAC endpoint
let endpoint = "https://data.geo.admin.ch/api/stac/v0.9/";
let client = Client::new(endpoint).unwrap();

// Fetch root catalog and print `id`
let root = client.root().unwrap();
println!("Root catalog id: {}", catalog.id);

// Count catalogs
let catalogs = client.catalogs().unwrap();
println!("Found {} catalogs!", catalogs.count());

// Search items
let bbox = vec![7.4473, 46.9479, 7.4475, 46.9481];
let params = SearchParams::new()
    .with_bbox(&bbox)
    .with_collections(&["ch.swisstopo.swissalti3d"])
    .build();
let items = client.search(params).unwrap();
printl!("Found {} items!", items.count());
```
