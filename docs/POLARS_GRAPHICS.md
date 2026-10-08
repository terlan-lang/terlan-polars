# Polars graphics contract

`terlan-polars` exposes a typed, bounded graphics surface over package-owned
DataFrames. It follows the Polars 2 plotting names where Polars defines them,
while using Rust renderers instead of returning Python Altair objects.

| Chart | Default static API | Interactive API | Raster API | Color grouping |
| --- | --- | --- | --- | --- |
| Point/scatter | `plot_point_svg` / `plot_scatter_svg` | `plot_point_html` / `plot_scatter_html` | `plot_point_png` / `plot_scatter_png` | Yes |
| Line | `plot_line_svg` | `plot_line_html` | `plot_line_png` | No |
| Bar | `plot_bar_svg` | `plot_bar_html` | `plot_bar_png` | Yes |
| Histogram | `plot_histogram_svg` | `plot_histogram_html` | `plot_histogram_png` | Yes |
| Box | `plot_box_svg` | `plot_box_html` | `plot_box_png` | Yes |

SVG is always the lightweight default and is rendered through
Plotlars/Plotters. Interactive HTML uses Plotlars/Plotly behind the optional
`plotly-html` Cargo feature. PNG uses Plotlars/Plotters behind the optional
`plot-png` feature. All three output paths return in-memory `Bytes` values.

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
boundary. Terlan's contract is the five chart families in the table, a title,
axis columns, and the listed color grouping support.

The line utility renders one line from one x/y pair. A Polars/Altair call such
as `df.plot.line(x="date", y="price", color="stock")` needs separate series
preparation or an external visualization adapter because the current Plotlars
line API groups multiple lines by y columns rather than by a categorical color
column. Bar faceting and arbitrary Altair encodings likewise remain external.

The executable coverage consists of Terlan point/scatter, line, bar,
histogram, box, and validation tests; Rust tests for all five SVG renderers;
and feature-gated Rust tests for all five Plotly HTML and PNG renderers. Run
`make visualization-feature-check` for the optional-backend matrix and
`make release-check` for the complete package gate.
