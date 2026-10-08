# Composable Polars LazyFrame

Exercises lazy local-file scans, expression pipelines, grouping, sorting,
limits, joins, textual and tree logical-plan inspection, per-node execution
profiling, ordinary collection, and streaming collection.
It also exercises an all-disabled optimizer baseline and named optimizer-pass
configuration without consuming the source plan, plus eager and lazy
sequential projection and column-mutation execution. Eager and lazy inverse
filters are covered as well; they remove true predicate rows while retaining
both false and null predicate rows.

```bash
terlc run examples/polars_lazy_full
```
