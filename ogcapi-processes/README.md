# Processes

The `ogcapi-processes` crate contains the [`Processor`](./src/processor.rs) trait and implementations. Have a look at the [`Greeter`](./src/greeter.rs) for starters.

## Examples

| Process | Feature  | Description |
| ------- | -------- | ----------- |
| Greeter | -        | The *"Hello, World!"* process. Expects a `name` as input and returns `Hello, {name}!` |
| Loader  | `loader` | Load vector data to PostgreSQL. |
