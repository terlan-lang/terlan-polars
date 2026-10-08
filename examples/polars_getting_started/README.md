# Polars getting-started examples

This executable Terlan project ports every DataFrame example from the official
[Polars getting-started guide](https://docs.pola.rs/user-guide/getting-started/):
CSV reading/writing, expression selection and expansion, `with_columns`, both
filter forms, grouped counts and aggregations, the chained query, a left join,
vertical concatenation, and reproducible fixed-count/fractional row sampling.

Run from the `terlan-polars` repository root:

```sh
../terlan/target/debug/terlc run examples/polars_getting_started
```
