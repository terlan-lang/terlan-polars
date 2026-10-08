# Polars graphics contract

`terlan-polars` exposes a typed, bounded graphics surface over package-owned
DataFrames. It follows the Polars 2 plotting names where Polars defines them,
while using Rust renderers instead of returning Python Altair objects.

| Chart | Default static API | Interactive API | Raster API | Color grouping | Faceting |
| --- | --- | --- | --- | --- | --- |
| Point/scatter | `plot_point_svg` / `plot_scatter_svg` | `plot_point_html` / `plot_scatter_html` | `plot_point_png` / `plot_scatter_png` | Yes | Yes |
| Line | `plot_line_svg` | `plot_line_html` | `plot_line_png` | Yes | Yes, without color grouping |
| Bar | `plot_bar_svg` | `plot_bar_html` | `plot_bar_png` | Yes | Yes |
| Histogram | `plot_histogram_svg` | `plot_histogram_html` | `plot_histogram_png` | Yes | Yes |
| Box | `plot_box_svg` | `plot_box_html` | `plot_box_png` | Yes | Yes |

SVG is always the lightweight default and is rendered through
Plotlars/Plotters. Interactive HTML uses Plotlars/Plotly behind the optional
`plotly-html` Cargo feature. PNG uses Plotlars/Plotters behind the optional
`plot-png` feature. All three output paths return in-memory `Bytes` values.

The legacy calls accept their common title/group arguments directly. The
`*_with_options` variants take an immutable `PlotOptions` descriptor created
with `plot_options` and updated through `with_plot_title`, `with_plot_color`,
`with_plot_column` (or `with_plot_facet`), and `with_plot_dimensions`. One
descriptor has the same meaning for SVG, Plotly HTML, and PNG. Color is the
Polars-compatible categorical grouping channel; facet creates bounded small
multiples. Facets are limited to eight and fixed dimensions range from 64 to
4096 pixels.

Every plot validates required columns, rejects empty frames, limits inputs to
100,000 rows, and caps each rendered document at 8 MiB. SVG and HTML outputs
are checked for their expected document markers, and PNG output is checked for
its file signature. Rendering panics are converted into stable `plot_error`
results. Optional backends return `native_unavailable` when their feature is
disabled.

Polars 2's Python `DataFrame.plot` property is a convenience namespace over
Altair. Its canonical point spelling is `plot.point`, with `plot.scatter` as an
alias; Terlan therefore exposes both point and scatter names. Polars forwards
arbitrary marks, encoding channels, facets, and chained configuration to
Altair. Those open-ended Python chart objects are outside the native package
boundary. Terlan directly covers the five chart families in the table,
including `df.plot.line(x="date", y="price", color="stock")` style categorical
line grouping and `df.plot.bar(..., column="region")` style facets. Combining
line color grouping and line faceting in one chart remains unsupported.

The executable coverage consists of Terlan point/scatter, line, bar,
histogram, box, typed options, and validation tests; Rust tests for grouped
lines, faceted bars, fixed dimensions, and all five SVG renderers; and
feature-gated Rust tests for all five Plotly HTML and PNG renderers. Run
`make visualization-feature-check` for the optional-backend matrix and
`make release-check` for the complete package gate.
