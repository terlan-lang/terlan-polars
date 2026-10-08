//! Bounded in-memory visualization for package-owned DataFrames.

#[cfg(feature = "real-polars")]
use crate::plot_options::{decode_plot_options, PlotOptions, MAX_FACETS};
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

#[cfg(feature = "real-polars")]
#[derive(Clone)]
struct OwnedPlot {
    traces: Vec<plotlars_core::ir::trace::TraceIR>,
    layout: plotlars_core::ir::layout::LayoutIR,
}

#[cfg(feature = "real-polars")]
impl plotlars_core::Plot for OwnedPlot {
    fn ir_traces(&self) -> &[plotlars_core::ir::trace::TraceIR] {
        &self.traces
    }

    fn ir_layout(&self) -> &plotlars_core::ir::layout::LayoutIR {
        &self.layout
    }
}

#[cfg(feature = "real-polars")]
impl OwnedPlot {
    fn from_plot(plot: &impl plotlars_core::Plot, options: &PlotOptions) -> Self {
        let mut layout = plot.ir_layout().clone();
        if let (Some(width), Some(height)) = (options.width, options.height) {
            layout.dimensions = Some(
                plotlars_core::components::Dimensions::new()
                    .width(width)
                    .height(height)
                    .auto_size(false),
            );
        }
        Self {
            traces: plot.ir_traces().to_vec(),
            layout,
        }
    }
}

#[cfg(feature = "real-polars")]
fn facet_config(options: &PlotOptions) -> Option<plotlars_core::components::FacetConfig> {
    options
        .facet_columns
        .map(|columns| plotlars_core::components::FacetConfig::new().cols(columns))
}

#[cfg(feature = "real-polars")]
fn validate_facet_count(
    dataframe: &TerlanPolarsDataFrame,
    facet: Option<&str>,
) -> Result<(), TerlanPolarsError> {
    if let Some(facet) = facet {
        let count = dataframe
            .inner
            .column(facet)
            .and_then(|column| column.n_unique())
            .map_err(|error| TerlanPolarsError::new("plot_error", error.to_string()))?;
        if count > MAX_FACETS {
            return Err(TerlanPolarsError::new(
                "plot_limit_exceeded",
                format!("facet column `{facet}` has {count} values; the maximum is {MAX_FACETS}"),
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "real-polars")]
#[allow(clippy::too_many_arguments)]
fn basic_line_plot(
    data: &polars::prelude::DataFrame,
    x: &str,
    y: &str,
    facet: Option<&str>,
    config: Option<&plotlars_core::components::FacetConfig>,
    color: Option<plotlars_core::components::Rgb>,
    title: &str,
    legend_title: Option<&str>,
    options: &PlotOptions,
) -> Result<OwnedPlot, TerlanPolarsError> {
    use plotlars_core::{plots::lineplot::LinePlot, plots::timeseriesplot::TimeSeriesPlot};

    let numeric_x = data
        .column(x)
        .map_err(|error| TerlanPolarsError::new("plot_error", error.to_string()))?
        .dtype()
        .is_numeric();
    macro_rules! build {
        ($plot:ty) => {{
            let plot = <$plot>::try_new(
                data,
                x,
                y,
                None,
                facet,
                config,
                None,
                color,
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
                legend_title.map(Into::into),
                None,
                None,
                None,
                None,
            )
            .map_err(plot_build_error)?;
            OwnedPlot::from_plot(&plot, options)
        }};
    }
    Ok(if numeric_x {
        build!(LinePlot)
    } else {
        build!(TimeSeriesPlot)
    })
}

#[cfg(feature = "real-polars")]
fn grouped_line_plot(
    dataframe: &TerlanPolarsDataFrame,
    x: &str,
    y: &str,
    group: &str,
    options: &PlotOptions,
) -> Result<OwnedPlot, TerlanPolarsError> {
    use plotlars_core::{components::Rgb, ir::trace::TraceIR};
    use polars::prelude::{col, lit, DataType, IntoLazy};

    const MAX_GROUPS: usize = 64;
    const COLORS: [Rgb; 10] = [
        Rgb(31, 119, 180),
        Rgb(255, 127, 14),
        Rgb(44, 160, 44),
        Rgb(214, 39, 40),
        Rgb(148, 103, 189),
        Rgb(140, 86, 75),
        Rgb(227, 119, 194),
        Rgb(127, 127, 127),
        Rgb(188, 189, 34),
        Rgb(23, 190, 207),
    ];

    let groups = plotlars_core::data::get_unique_groups(&dataframe.inner, group, None);
    if groups.len() > MAX_GROUPS {
        return Err(TerlanPolarsError::new(
            "plot_limit_exceeded",
            format!(
                "line color column `{group}` has {} values; the maximum is {MAX_GROUPS}",
                groups.len()
            ),
        ));
    }

    let mut result = basic_line_plot(
        &dataframe.inner,
        x,
        y,
        None,
        None,
        None,
        &options.title,
        Some(group),
        options,
    )?;
    result.traces.clear();

    for (index, group_name) in groups.iter().enumerate() {
        let filtered = dataframe
            .inner
            .clone()
            .lazy()
            .filter(
                col(group)
                    .cast(DataType::String)
                    .eq(lit(group_name.as_str())),
            )
            .collect()
            .map_err(|error| TerlanPolarsError::new("plot_error", error.to_string()))?;
        let line = basic_line_plot(
            &filtered,
            x,
            y,
            None,
            None,
            Some(COLORS[index % COLORS.len()]),
            "",
            None,
            options,
        )?;
        let mut trace =
            line.traces.first().cloned().ok_or_else(|| {
                TerlanPolarsError::new("plot_error", "line group produced no trace")
            })?;
        match &mut trace {
            TraceIR::LinePlot(line) => {
                line.name = Some(group_name.clone());
                line.legend_group = Some(group_name.clone());
                line.show_legend = Some(true);
            }
            TraceIR::TimeSeriesPlot(line) => {
                line.name = Some(group_name.clone());
                line.legend_group = Some(group_name.clone());
                line.show_legend = Some(true);
            }
            _ => {
                return Err(TerlanPolarsError::new(
                    "plot_error",
                    "line group produced an unexpected trace",
                ));
            }
        }
        result.traces.push(trace);
    }
    Ok(result)
}

#[cfg(feature = "real-polars")]
fn build_plot(
    dataframe: &TerlanPolarsDataFrame,
    kind: &str,
    x: &str,
    y: &str,
    descriptor: &str,
) -> Result<OwnedPlot, TerlanPolarsError> {
    use plotlars_core::plots::{
        barplot::BarPlot, boxplot::BoxPlot, histogram::Histogram, scatterplot::ScatterPlot,
    };

    let options = decode_plot_options(descriptor)?;
    let color = (!options.color.is_empty()).then_some(options.color.as_str());
    let facet = (!options.facet.is_empty()).then_some(options.facet.as_str());
    let config = facet_config(&options);
    let config_ref = config.as_ref();

    let mut columns = vec![x];
    if kind != "histogram" {
        columns.push(y);
    }
    if let Some(color) = color {
        columns.push(color);
    }
    if let Some(facet) = facet {
        columns.push(facet);
    }
    validate_plot(dataframe, &columns)?;
    validate_facet_count(dataframe, facet)?;

    let build = || -> Result<OwnedPlot, TerlanPolarsError> {
        let plot = match kind {
            "scatter" | "point" => {
                let plot = ScatterPlot::try_new(
                    &dataframe.inner,
                    x,
                    y,
                    color,
                    None,
                    facet,
                    config_ref,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    (!options.title.is_empty()).then(|| options.title.as_str().into()),
                    Some(x.into()),
                    Some(y.into()),
                    color.map(Into::into),
                    None,
                    None,
                    None,
                )
                .map_err(plot_build_error)?;
                OwnedPlot::from_plot(&plot, &options)
            }
            "line" => {
                if color.is_some() && facet.is_some() {
                    return Err(TerlanPolarsError::new(
                        "plot_error",
                        "line plots cannot combine categorical color and faceting",
                    ));
                }
                if let Some(group) = color {
                    grouped_line_plot(dataframe, x, y, group, &options)?
                } else {
                    basic_line_plot(
                        &dataframe.inner,
                        x,
                        y,
                        facet,
                        config_ref,
                        None,
                        &options.title,
                        None,
                        &options,
                    )?
                }
            }
            "bar" => {
                let plot = BarPlot::try_new(
                    &dataframe.inner,
                    x,
                    y,
                    None,
                    color,
                    None,
                    facet,
                    config_ref,
                    None,
                    None,
                    None,
                    None,
                    (!options.title.is_empty()).then(|| options.title.as_str().into()),
                    Some(x.into()),
                    Some(y.into()),
                    color.map(Into::into),
                    None,
                    None,
                    None,
                )
                .map_err(plot_build_error)?;
                OwnedPlot::from_plot(&plot, &options)
            }
            "histogram" => {
                let plot = Histogram::try_new(
                    &dataframe.inner,
                    x,
                    color,
                    None,
                    facet,
                    config_ref,
                    None,
                    None,
                    None,
                    (!options.title.is_empty()).then(|| options.title.as_str().into()),
                    Some(x.into()),
                    Some("count".into()),
                    color.map(Into::into),
                    None,
                    None,
                    None,
                )
                .map_err(plot_build_error)?;
                OwnedPlot::from_plot(&plot, &options)
            }
            "box" => {
                let plot = BoxPlot::try_new(
                    &dataframe.inner,
                    x,
                    y,
                    None,
                    color,
                    None,
                    facet,
                    config_ref,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    (!options.title.is_empty()).then(|| options.title.as_str().into()),
                    Some(x.into()),
                    Some(y.into()),
                    color.map(Into::into),
                    None,
                    None,
                    None,
                )
                .map_err(plot_build_error)?;
                OwnedPlot::from_plot(&plot, &options)
            }
            _ => {
                return Err(TerlanPolarsError::new(
                    "plot_error",
                    format!("unknown plot kind `{kind}`"),
                ));
            }
        };
        Ok(plot)
    };

    std::panic::catch_unwind(std::panic::AssertUnwindSafe(build))
        .map_err(|_| TerlanPolarsError::new("plot_error", "Plotlars failed while building plot"))?
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

/// Renders a plot with a versioned options descriptor as bounded SVG bytes.
#[cfg(feature = "real-polars")]
pub fn plot_with_svg(
    dataframe: &TerlanPolarsDataFrame,
    kind: &str,
    x: &str,
    y: &str,
    options: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    let plot = build_plot(dataframe, kind, x, y, options)?;
    render_svg(&plot)
}

/// Renders a plot with a versioned options descriptor as bounded Plotly HTML bytes.
#[cfg(feature = "plotly-html")]
pub fn plot_with_html(
    dataframe: &TerlanPolarsDataFrame,
    kind: &str,
    x: &str,
    y: &str,
    options: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    let plot = build_plot(dataframe, kind, x, y, options)?;
    render_plotly_html(&plot)
}

/// Renders a plot with a versioned options descriptor as bounded PNG bytes.
#[cfg(feature = "plot-png")]
pub fn plot_with_png(
    dataframe: &TerlanPolarsDataFrame,
    kind: &str,
    x: &str,
    y: &str,
    options: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    let plot = build_plot(dataframe, kind, x, y, options)?;
    render_png(&plot)
}

#[cfg(not(feature = "real-polars"))]
unavailable_plot!(plot_with_svg(dataframe: &TerlanPolarsDataFrame, kind: &str, x: &str, y: &str, options: &str));

#[cfg(not(feature = "plotly-html"))]
unavailable_plotly!(plot_with_html(dataframe: &TerlanPolarsDataFrame, kind: &str, x: &str, y: &str, options: &str));

#[cfg(not(feature = "plot-png"))]
unavailable_png!(plot_with_png(dataframe: &TerlanPolarsDataFrame, kind: &str, x: &str, y: &str, options: &str));

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
                "group" => ["one", "one", "two", "two"],
                "facet" => ["first", "second", "first", "second"],
                "date" => ["2026-01-01", "2026-01-02", "2026-01-01", "2026-01-02"]
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

    #[test]
    fn typed_options_build_grouped_lines_and_faceted_bars() {
        let df = frame();
        let line_options = crate::with_plot_color(&crate::plot_options(), "group").unwrap();
        let line_options = crate::with_plot_title(&line_options, "Grouped line").unwrap();
        let line_options = crate::with_plot_dimensions(&line_options, 960, 540).unwrap();
        let line = build_plot(&df, "line", "x", "y", &line_options).unwrap();
        assert_eq!(line.traces.len(), 2);
        let dimensions = line.layout.dimensions.as_ref().unwrap();
        assert_eq!(
            (dimensions.width, dimensions.height),
            (Some(960), Some(540))
        );
        assert!(is_svg(plot_with_svg(&df, "line", "x", "y", &line_options)));
        let dated_line = build_plot(&df, "line", "date", "y", &line_options).unwrap();
        assert_eq!(dated_line.traces.len(), 2);
        assert!(dated_line
            .traces
            .iter()
            .all(|trace| matches!(trace, plotlars_core::ir::trace::TraceIR::TimeSeriesPlot(_))));

        let bar_options = crate::with_plot_color(&crate::plot_options(), "group").unwrap();
        let bar_options = crate::with_plot_facet(&bar_options, "facet", 2).unwrap();
        let bar = build_plot(&df, "bar", "label", "y", &bar_options).unwrap();
        let grid = bar.layout.grid.as_ref().unwrap();
        assert_eq!((grid.cols, grid.rows, grid.n_facets), (2, 1, 2));
        assert!(is_svg(plot_with_svg(
            &df,
            "bar",
            "label",
            "y",
            &bar_options
        )));
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

        let options = crate::with_plot_color(&crate::plot_options(), "group").unwrap();
        let options = crate::with_plot_dimensions(&options, 960, 540).unwrap();
        let html =
            String::from_utf8(plot_with_html(&df, "line", "x", "y", &options).unwrap()).unwrap();
        assert!(html.contains("one"));
        assert!(html.contains("two"));
        assert!(html.contains("\"width\":960"));
        assert!(html.contains("\"height\":540"));
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
        let options = crate::with_plot_facet(&crate::plot_options(), "facet", 2).unwrap();
        let options = crate::with_plot_dimensions(&options, 720, 480).unwrap();
        let png = plot_with_png(&df, "bar", "label", "y", &options).unwrap();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 720);
        assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 480);
    }
}
