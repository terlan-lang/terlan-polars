//! Bounded in-memory visualization for package-owned DataFrames.

use crate::{TerlanPolarsDataFrame, TerlanPolarsError};

#[cfg(feature = "real-polars")]
const MAX_PLOT_ROWS: usize = 100_000;
#[cfg(feature = "real-polars")]
const MAX_SVG_BYTES: usize = 8 * 1024 * 1024;
#[cfg(feature = "plotly-html")]
const MAX_HTML_BYTES: usize = 8 * 1024 * 1024;

#[cfg(feature = "real-polars")]
use plotlars_plotters::PlottersExt;

#[cfg(feature = "real-polars")]
fn validate_plot(
    dataframe: &TerlanPolarsDataFrame,
    required_columns: &[&str],
) -> Result<(), TerlanPolarsError> {
    if dataframe.inner.height() > MAX_PLOT_ROWS {
        return Err(TerlanPolarsError::new(
            "plot_limit_exceeded",
            format!(
                "plot input has {} rows; the maximum is {MAX_PLOT_ROWS}",
                dataframe.inner.height()
            ),
        ));
    }
    if dataframe.inner.height() == 0 {
        return Err(TerlanPolarsError::new(
            "plot_error",
            "plot input must contain at least one row",
        ));
    }
    for column in required_columns {
        if column.is_empty() {
            return Err(TerlanPolarsError::new(
                "plot_error",
                "plot column names cannot be empty",
            ));
        }
        if dataframe.inner.column(column).is_err() {
            return Err(TerlanPolarsError::new(
                "plot_error",
                format!("plot column `{column}` does not exist"),
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "real-polars")]
fn optional_column<'a>(name: &'a str, columns: &mut Vec<&'a str>) -> Option<&'a str> {
    if name.is_empty() {
        None
    } else {
        columns.push(name);
        Some(name)
    }
}

#[cfg(feature = "real-polars")]
fn render_svg<P: PlottersExt>(plot: &P) -> Result<Vec<u8>, TerlanPolarsError> {
    let svg =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plot.to_svg())).map_err(|_| {
            TerlanPolarsError::new(
                "plot_error",
                "Plotlars/Plotters failed while rendering the SVG",
            )
        })?;
    if svg.len() > MAX_SVG_BYTES {
        return Err(TerlanPolarsError::new(
            "plot_limit_exceeded",
            format!(
                "rendered SVG is {} bytes; the maximum is {MAX_SVG_BYTES}",
                svg.len()
            ),
        ));
    }
    if !svg.contains("<svg") {
        return Err(TerlanPolarsError::new(
            "plot_error",
            "Plotlars/Plotters returned an invalid SVG document",
        ));
    }
    Ok(svg.into_bytes())
}

#[cfg(feature = "real-polars")]
fn plot_build_error(error: plotlars_core::io::PlotlarsError) -> TerlanPolarsError {
    TerlanPolarsError::new("plot_error", format!("could not build plot: {error}"))
}

/// Renders a scatter plot as bounded UTF-8 SVG bytes. Empty `group` disables grouping.
#[cfg(feature = "real-polars")]
pub fn plot_scatter_svg(
    dataframe: &TerlanPolarsDataFrame,
    x: &str,
    y: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::scatterplot::ScatterPlot;

    let mut columns = vec![x, y];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = ScatterPlot::try_new(
        &dataframe.inner,
        x,
        y,
        group,
        None, // group sorter
        None, // facet
        None, // facet configuration
        None, // opacity
        None, // size
        None, // color
        None, // colors
        None, // shape
        None, // shapes
        (!title.is_empty()).then(|| title.into()),
        Some(x.into()),
        Some(y.into()),
        group.map(Into::into),
        None, // x axis
        None, // y axis
        None, // legend
    )
    .map_err(plot_build_error)?;
    render_svg(&plot)
}

/// Renders a line plot as bounded UTF-8 SVG bytes.
#[cfg(feature = "real-polars")]
pub fn plot_line_svg(
    dataframe: &TerlanPolarsDataFrame,
    x: &str,
    y: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::lineplot::LinePlot;

    validate_plot(dataframe, &[x, y])?;
    let plot = LinePlot::try_new(
        &dataframe.inner,
        x,
        y,
        None, // additional lines
        None, // facet
        None, // facet configuration
        None, // size
        None, // color
        None, // colors
        None, // shape
        None, // shapes
        None, // width
        None, // line
        None, // lines
        (!title.is_empty()).then(|| title.into()),
        Some(x.into()),
        Some(y.into()),
        None, // secondary y title
        None, // legend title
        None, // x axis
        None, // y axis
        None, // secondary y axis
        None, // legend
    )
    .map_err(plot_build_error)?;
    render_svg(&plot)
}

/// Renders a bar plot as bounded UTF-8 SVG bytes. Empty `group` disables grouping.
#[cfg(feature = "real-polars")]
pub fn plot_bar_svg(
    dataframe: &TerlanPolarsDataFrame,
    labels: &str,
    values: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::barplot::BarPlot;

    let mut columns = vec![labels, values];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = BarPlot::try_new(
        &dataframe.inner,
        labels,
        values,
        None,
        group,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(labels.into()),
        Some(values.into()),
        group.map(Into::into),
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_svg(&plot)
}

/// Renders a histogram as bounded UTF-8 SVG bytes. Empty `group` disables grouping.
#[cfg(feature = "real-polars")]
pub fn plot_histogram_svg(
    dataframe: &TerlanPolarsDataFrame,
    x: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::histogram::Histogram;

    let mut columns = vec![x];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = Histogram::try_new(
        &dataframe.inner,
        x,
        group,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(x.into()),
        Some("count".into()),
        group.map(Into::into),
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_svg(&plot)
}

/// Renders a box plot as bounded UTF-8 SVG bytes. Empty `group` disables grouping.
#[cfg(feature = "real-polars")]
pub fn plot_box_svg(
    dataframe: &TerlanPolarsDataFrame,
    labels: &str,
    values: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::boxplot::BoxPlot;

    let mut columns = vec![labels, values];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = BoxPlot::try_new(
        &dataframe.inner,
        labels,
        values,
        None,
        group,
        None, // group sorter
        None, // facet
        None, // facet configuration
        None, // box points
        None, // point offset
        None, // jitter
        None, // opacity
        None, // color
        None, // colors
        (!title.is_empty()).then(|| title.into()),
        Some(labels.into()),
        Some(values.into()),
        group.map(Into::into),
        None, // x axis
        None, // y axis
        None, // legend
    )
    .map_err(plot_build_error)?;
    render_svg(&plot)
}

#[cfg(feature = "plotly-html")]
fn render_plotly_html<P: plotlars_plotly::PlotlyExt>(
    plot: &P,
) -> Result<Vec<u8>, TerlanPolarsError> {
    let html = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plot.to_html())).map_err(
        |_| TerlanPolarsError::new("plot_error", "Plotlars/Plotly failed while rendering HTML"),
    )?;
    if html.len() > MAX_HTML_BYTES {
        return Err(TerlanPolarsError::new(
            "plot_limit_exceeded",
            format!(
                "rendered Plotly HTML is {} bytes; the maximum is {MAX_HTML_BYTES}",
                html.len()
            ),
        ));
    }
    if !html.contains("Plotly") && !html.contains("plotly") {
        return Err(TerlanPolarsError::new(
            "plot_error",
            "Plotlars/Plotly returned an invalid HTML document",
        ));
    }
    Ok(html.into_bytes())
}

/// Renders an interactive Plotly scatter plot as bounded HTML bytes.
#[cfg(feature = "plotly-html")]
pub fn plot_scatter_html(
    dataframe: &TerlanPolarsDataFrame,
    x: &str,
    y: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::scatterplot::ScatterPlot;

    let mut columns = vec![x, y];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = ScatterPlot::try_new(
        &dataframe.inner,
        x,
        y,
        group,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(x.into()),
        Some(y.into()),
        group.map(Into::into),
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_plotly_html(&plot)
}

/// Renders an interactive Plotly line plot as bounded HTML bytes.
#[cfg(feature = "plotly-html")]
pub fn plot_line_html(
    dataframe: &TerlanPolarsDataFrame,
    x: &str,
    y: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::lineplot::LinePlot;

    validate_plot(dataframe, &[x, y])?;
    let plot = LinePlot::try_new(
        &dataframe.inner,
        x,
        y,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(x.into()),
        Some(y.into()),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_plotly_html(&plot)
}

/// Renders an interactive Plotly bar plot as bounded HTML bytes.
#[cfg(feature = "plotly-html")]
pub fn plot_bar_html(
    dataframe: &TerlanPolarsDataFrame,
    labels: &str,
    values: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::barplot::BarPlot;

    let mut columns = vec![labels, values];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = BarPlot::try_new(
        &dataframe.inner,
        labels,
        values,
        None,
        group,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(labels.into()),
        Some(values.into()),
        group.map(Into::into),
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_plotly_html(&plot)
}

/// Renders an interactive Plotly histogram as bounded HTML bytes.
#[cfg(feature = "plotly-html")]
pub fn plot_histogram_html(
    dataframe: &TerlanPolarsDataFrame,
    x: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::histogram::Histogram;

    let mut columns = vec![x];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = Histogram::try_new(
        &dataframe.inner,
        x,
        group,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(x.into()),
        Some("count".into()),
        group.map(Into::into),
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_plotly_html(&plot)
}

/// Renders an interactive Plotly box plot as bounded HTML bytes.
#[cfg(feature = "plotly-html")]
pub fn plot_box_html(
    dataframe: &TerlanPolarsDataFrame,
    labels: &str,
    values: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::boxplot::BoxPlot;

    let mut columns = vec![labels, values];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = BoxPlot::try_new(
        &dataframe.inner,
        labels,
        values,
        None,
        group,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(labels.into()),
        Some(values.into()),
        group.map(Into::into),
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_plotly_html(&plot)
}

#[cfg(not(feature = "plotly-html"))]
macro_rules! unavailable_plotly {
    ($name:ident($($arg:ident: $kind:ty),*)) => {
        #[doc = "Returns a stable unavailable error when optional Plotly HTML support is disabled."]
        pub fn $name($($arg: $kind),*) -> Result<Vec<u8>, TerlanPolarsError> {
            $(let _ = $arg;)*
            Err(TerlanPolarsError::new(
                "native_unavailable",
                "interactive Plotly HTML requires the optional plotly-html native feature",
            ))
        }
    };
}

#[cfg(not(feature = "plotly-html"))]
unavailable_plotly!(plot_scatter_html(dataframe: &TerlanPolarsDataFrame, x: &str, y: &str, group: &str, title: &str));
#[cfg(not(feature = "plotly-html"))]
unavailable_plotly!(plot_line_html(dataframe: &TerlanPolarsDataFrame, x: &str, y: &str, title: &str));
#[cfg(not(feature = "plotly-html"))]
unavailable_plotly!(plot_bar_html(dataframe: &TerlanPolarsDataFrame, labels: &str, values: &str, group: &str, title: &str));
#[cfg(not(feature = "plotly-html"))]
unavailable_plotly!(plot_histogram_html(dataframe: &TerlanPolarsDataFrame, x: &str, group: &str, title: &str));
#[cfg(not(feature = "plotly-html"))]
unavailable_plotly!(plot_box_html(dataframe: &TerlanPolarsDataFrame, labels: &str, values: &str, group: &str, title: &str));

#[cfg(feature = "plot-png")]
fn render_png<P: PlottersExt>(plot: &P) -> Result<Vec<u8>, TerlanPolarsError> {
    use std::sync::atomic::{AtomicU64, Ordering};

    const MAX_PNG_BYTES: usize = 8 * 1024 * 1024;
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

    let mut directory = None;
    for _ in 0..16 {
        let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let candidate = std::env::temp_dir().join(format!(
            "terlan-polars-png-{}-{sequence}",
            std::process::id()
        ));
        match std::fs::create_dir(&candidate) {
            Ok(()) => {
                directory = Some(candidate);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(TerlanPolarsError::new(
                    "plot_error",
                    format!("could not create PNG render directory: {error}"),
                ));
            }
        }
    }
    let directory = directory.ok_or_else(|| {
        TerlanPolarsError::new("plot_error", "could not allocate a PNG render directory")
    })?;
    let path = directory.join("plot.png");
    let rendered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        plot.save(path.to_string_lossy().as_ref());
    }))
    .map_err(|_| {
        TerlanPolarsError::new("plot_error", "Plotlars/Plotters failed while rendering PNG")
    });
    let bytes = rendered.and_then(|()| {
        std::fs::read(&path).map_err(|error| {
            TerlanPolarsError::new(
                "plot_error",
                format!("could not read rendered PNG: {error}"),
            )
        })
    });
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&directory);
    let bytes = bytes?;
    if bytes.len() > MAX_PNG_BYTES {
        return Err(TerlanPolarsError::new(
            "plot_limit_exceeded",
            format!(
                "rendered PNG is {} bytes; the maximum is {MAX_PNG_BYTES}",
                bytes.len()
            ),
        ));
    }
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(TerlanPolarsError::new(
            "plot_error",
            "Plotlars/Plotters returned an invalid PNG document",
        ));
    }
    Ok(bytes)
}

/// Renders an optional PNG scatter plot.
#[cfg(feature = "plot-png")]
pub fn plot_scatter_png(
    dataframe: &TerlanPolarsDataFrame,
    x: &str,
    y: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::scatterplot::ScatterPlot;
    let mut columns = vec![x, y];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = ScatterPlot::try_new(
        &dataframe.inner,
        x,
        y,
        group,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(x.into()),
        Some(y.into()),
        group.map(Into::into),
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_png(&plot)
}

/// Renders an optional PNG line plot.
#[cfg(feature = "plot-png")]
pub fn plot_line_png(
    dataframe: &TerlanPolarsDataFrame,
    x: &str,
    y: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::lineplot::LinePlot;
    validate_plot(dataframe, &[x, y])?;
    let plot = LinePlot::try_new(
        &dataframe.inner,
        x,
        y,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(x.into()),
        Some(y.into()),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_png(&plot)
}

/// Renders an optional PNG bar plot.
#[cfg(feature = "plot-png")]
pub fn plot_bar_png(
    dataframe: &TerlanPolarsDataFrame,
    labels: &str,
    values: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::barplot::BarPlot;
    let mut columns = vec![labels, values];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = BarPlot::try_new(
        &dataframe.inner,
        labels,
        values,
        None,
        group,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(labels.into()),
        Some(values.into()),
        group.map(Into::into),
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_png(&plot)
}

/// Renders an optional PNG histogram.
#[cfg(feature = "plot-png")]
pub fn plot_histogram_png(
    dataframe: &TerlanPolarsDataFrame,
    x: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::histogram::Histogram;
    let mut columns = vec![x];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = Histogram::try_new(
        &dataframe.inner,
        x,
        group,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(x.into()),
        Some("count".into()),
        group.map(Into::into),
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_png(&plot)
}

/// Renders an optional PNG box plot.
#[cfg(feature = "plot-png")]
pub fn plot_box_png(
    dataframe: &TerlanPolarsDataFrame,
    labels: &str,
    values: &str,
    group: &str,
    title: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use plotlars_core::plots::boxplot::BoxPlot;
    let mut columns = vec![labels, values];
    let group = optional_column(group, &mut columns);
    validate_plot(dataframe, &columns)?;
    let plot = BoxPlot::try_new(
        &dataframe.inner,
        labels,
        values,
        None,
        group,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        (!title.is_empty()).then(|| title.into()),
        Some(labels.into()),
        Some(values.into()),
        group.map(Into::into),
        None,
        None,
        None,
    )
    .map_err(plot_build_error)?;
    render_png(&plot)
}

#[cfg(not(feature = "plot-png"))]
macro_rules! unavailable_png {
    ($name:ident($($arg:ident: $kind:ty),*)) => {
        #[doc = "Returns a stable unavailable error when optional PNG support is disabled."]
        pub fn $name($($arg: $kind),*) -> Result<Vec<u8>, TerlanPolarsError> {
            $(let _ = $arg;)*
            Err(TerlanPolarsError::new(
                "native_unavailable",
                "PNG rendering requires the optional plot-png native feature",
            ))
        }
    };
}

#[cfg(not(feature = "plot-png"))]
unavailable_png!(plot_scatter_png(dataframe: &TerlanPolarsDataFrame, x: &str, y: &str, group: &str, title: &str));
#[cfg(not(feature = "plot-png"))]
unavailable_png!(plot_line_png(dataframe: &TerlanPolarsDataFrame, x: &str, y: &str, title: &str));
#[cfg(not(feature = "plot-png"))]
unavailable_png!(plot_bar_png(dataframe: &TerlanPolarsDataFrame, labels: &str, values: &str, group: &str, title: &str));
#[cfg(not(feature = "plot-png"))]
unavailable_png!(plot_histogram_png(dataframe: &TerlanPolarsDataFrame, x: &str, group: &str, title: &str));
#[cfg(not(feature = "plot-png"))]
unavailable_png!(plot_box_png(dataframe: &TerlanPolarsDataFrame, labels: &str, values: &str, group: &str, title: &str));

#[cfg(not(feature = "real-polars"))]
macro_rules! unavailable_plot {
    ($name:ident($($arg:ident: $kind:ty),*)) => {
        #[doc = "Returns a stable unavailable error when native Polars support is disabled."]
        pub fn $name($($arg: $kind),*) -> Result<Vec<u8>, TerlanPolarsError> {
            $(let _ = $arg;)*
            Err(TerlanPolarsError::new(
                "native_unavailable",
                "visualization requires the Rust native adapter target capability",
            ))
        }
    };
}

#[cfg(not(feature = "real-polars"))]
unavailable_plot!(plot_scatter_svg(dataframe: &TerlanPolarsDataFrame, x: &str, y: &str, group: &str, title: &str));
#[cfg(not(feature = "real-polars"))]
unavailable_plot!(plot_line_svg(dataframe: &TerlanPolarsDataFrame, x: &str, y: &str, title: &str));
#[cfg(not(feature = "real-polars"))]
unavailable_plot!(plot_bar_svg(dataframe: &TerlanPolarsDataFrame, labels: &str, values: &str, group: &str, title: &str));
#[cfg(not(feature = "real-polars"))]
unavailable_plot!(plot_histogram_svg(dataframe: &TerlanPolarsDataFrame, x: &str, group: &str, title: &str));
#[cfg(not(feature = "real-polars"))]
unavailable_plot!(plot_box_svg(dataframe: &TerlanPolarsDataFrame, labels: &str, values: &str, group: &str, title: &str));

#[cfg(all(test, feature = "real-polars"))]
mod tests {
    use super::*;
    use polars::df;

    fn frame() -> TerlanPolarsDataFrame {
        TerlanPolarsDataFrame {
            inner: df![
                "x" => [1.0, 2.0, 3.0, 4.0],
                "y" => [4.0, 2.0, 5.0, 3.0],
                "label" => ["a", "b", "c", "d"],
                "group" => ["one", "one", "two", "two"]
            ]
            .unwrap(),
        }
    }

    fn is_svg(result: Result<Vec<u8>, TerlanPolarsError>) -> bool {
        let bytes = result.unwrap();
        std::str::from_utf8(&bytes).unwrap().contains("<svg")
    }

    #[test]
    fn renders_all_default_svg_plot_types() {
        let df = frame();
        assert!(is_svg(plot_scatter_svg(&df, "x", "y", "group", "Scatter")));
        assert!(is_svg(plot_line_svg(&df, "x", "y", "Line")));
        assert!(is_svg(plot_bar_svg(&df, "label", "y", "group", "Bar")));
        assert!(is_svg(plot_histogram_svg(&df, "x", "group", "Histogram")));
        assert!(is_svg(plot_box_svg(&df, "label", "y", "group", "Box")));
    }

    #[test]
    fn reports_missing_columns_before_plotlars_runs() {
        let error = plot_line_svg(&frame(), "missing", "y", "").unwrap_err();
        assert_eq!(error.code(), "plot_error");
        assert!(error.message().contains("does not exist"));
    }

    #[cfg(feature = "plotly-html")]
    #[test]
    fn renders_all_optional_plotly_html_plot_types() {
        let df = frame();
        for result in [
            plot_scatter_html(&df, "x", "y", "group", "Scatter"),
            plot_line_html(&df, "x", "y", "Line"),
            plot_bar_html(&df, "label", "y", "group", "Bar"),
            plot_histogram_html(&df, "x", "group", "Histogram"),
            plot_box_html(&df, "label", "y", "group", "Box"),
        ] {
            let html = String::from_utf8(result.unwrap()).unwrap();
            assert!(html.contains("plotly") || html.contains("Plotly"));
        }
    }

    #[cfg(feature = "plot-png")]
    #[test]
    fn renders_all_optional_png_plot_types() {
        let df = frame();
        for result in [
            plot_scatter_png(&df, "x", "y", "group", "Scatter"),
            plot_line_png(&df, "x", "y", "Line"),
            plot_bar_png(&df, "label", "y", "group", "Bar"),
            plot_histogram_png(&df, "x", "group", "Histogram"),
            plot_box_png(&df, "label", "y", "group", "Box"),
        ] {
            assert!(result.unwrap().starts_with(b"\x89PNG\r\n\x1a\n"));
        }
    }
}
