# Polars expression namespaces

Exercises calendar and subsecond temporal component extraction, local
date/time/datetime projection, month boundaries, epoch timestamps,
duration-string bucketing/offsets, all Duration total units, string-to-list conversion, List
indexing, membership, sorting, statistics, slicing, shifting, joining,
null removal, seeded and unseeded sampling, gathering, differences, match
counting, set algebra, element evaluation/aggregation, configurable null/order
behavior, and List-to-Array/Struct conversion; plus fixed-width Array
indexing, membership, sorting, statistics, slicing, shifting, joining,
exploding, match counting, element evaluation/aggregation, configurable
sort/get/null behavior, and List/Struct conversion. It also casts
real columns to Categorical and Binary storage, then exercises category
inspection/predicates and binary containment, affix, size, indexing, head, and
tail operations. The remaining pinned namespace surface is covered by
categorical and binary slicing, hex/Base64 round trips, and endian-explicit
binary reinterpretation. Struct coverage exercises positional and multi-field
selection, renaming, JSON encoding, and adding a field. Random coverage checks
seeded and unseeded shuffle and sampling, while Bitwise coverage checks every
integer bit-count and reduction expression. Statistics coverage checks scalar
and ordered reductions, NaN-propagating extrema, and all histogram bin modes.
Trigonometry coverage checks every circular, inverse, hyperbolic, and
inverse-hyperbolic expression, quadrant-aware `arctan2`, and degree/radian
conversion. Boolean coverage checks null/finite/NaN masks, distinctness,
null-aware comparisons, interval closure, list membership, approximate
equality, logical and horizontal composition, and scalar reductions.
Cumulative coverage checks forward and reverse count, sum, product, minimum,
and maximum expressions. Fixed rolling coverage checks minimum, maximum, mean,
sum, median, variance, and standard deviation with readiness and centered-label
configuration, plus quantile, rank, skewness, and kurtosis options.
Rolling-by coverage checks all nine duration/index-window aggregates with
explicit closed-side behavior.
Expression-name coverage checks root-name retention, affixes, literal/regex
replacement, casing, and Struct field affixes.
Expression-metadata coverage checks root/output names, projection and literal
classification, multi-output and regex detection, and tree formatting.
Numeric-transform coverage checks rounding modes and significant figures,
truncation, floor/ceiling, absolute values, clipping, logarithms/exponentials,
pi, product, entropy, moments, and data-type bounds.
Scalar-aggregation coverage checks count/length null semantics, first/last
non-null values, singleton and List aggregation, quantiles, modes, unique
values and indices, extrema indices, dot products, and null/NaN removal.
Generic sequence coverage checks slicing, append/rechunk, head/tail, sorting,
index lookup and insertion search, gather/get, reversal and shifts, repeat-by,
differences and percentage change, strided gather, and constant extension.
Ranking-selection coverage checks every deterministic tie method, seeded
random ties, and top/bottom-k selection with direct and expression-key order.

```bash
terlc run examples/polars_expression_namespaces
```
