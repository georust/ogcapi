# Processes

The `ogcapi-processes` crate contains the [`Processor`](./src/processor.rs) trait and implementations. Have a look at the [`Greeter`](./src/greeter.rs) for starters.

## Examples

| Process | Feature  | Description |
| ------- | -------- | ----------- |
| Greeter | `greeter` | The *"Hello, World!"* process. Expects a `name` as input and returns `Hello, {name}!` |
| Echo    | -         | Echoes back the inputs it receives, to verify an OGC API Processes implementation. |
| GeoJSON loader | `geojson-loader` | Loads a GeoJSON file into a collection of a PostgreSQL database, which it replaces. |
| GDAL loader | `gdal-loader` | Loads a layer of a file GDAL reads into a collection of a PostgreSQL database, which it replaces. |
