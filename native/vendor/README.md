# Vendored Plotlars compatibility crates

Plotlars 0.12.6 is the newest published Plotlars release and pins Rust Polars
0.54.4. `terlan-polars` vendors `plotlars-core`, `plotlars-plotters`, and
`plotlars-plotly` so their manifests can resolve the package's Polars 0.55.2
dependency graph without linking two incompatible Polars versions.

The vendored Rust sources are unchanged from Plotlars 0.12.6. The local patch
only updates the Polars dependency version and connects the three crates with
relative path dependencies. Replace these directories with an upstream
release once Plotlars publishes support for Polars 0.55 or newer.

Plotlars is licensed under MIT. Its upstream repository is
<https://github.com/alceal/plotlars>.
