//! Declarative, escaped, and bounded HTML table presentation.

#[cfg(feature = "real-polars")]
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::{TerlanPolarsDataFrame, TerlanPolarsError};

const STYLE_VERSION: u8 = 1;
const MAX_STYLE_BYTES: usize = 64 * 1024;
#[cfg(feature = "real-polars")]
const MAX_TABLE_ROWS: usize = 1_000;
#[cfg(feature = "real-polars")]
const MAX_HTML_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TableStyle {
    version: u8,
    title: String,
    subtitle: String,
    stub: String,
    labels: Vec<ColumnLabel>,
    numbers: Vec<NumberFormat>,
    nanoplots: Vec<String>,
    colors: Vec<ColorScale>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ColumnLabel {
    column: String,
    label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct NumberFormat {
    columns: Vec<String>,
    decimals: usize,
    separators: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ColorScale {
    columns: Vec<String>,
    low: String,
    high: String,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            version: STYLE_VERSION,
            title: String::new(),
            subtitle: String::new(),
            stub: String::new(),
            labels: Vec::new(),
            numbers: Vec::new(),
            nanoplots: Vec::new(),
            colors: Vec::new(),
        }
    }
}

/// Creates an empty immutable table-style descriptor.
pub fn gt() -> String {
    encode(&TableStyle::default()).expect("the default table style is serializable")
}

/// Selects the column rendered as row-header cells.
pub fn tab_stub(descriptor: &str, column: &str) -> Result<String, TerlanPolarsError> {
    require_name(column, "stub column")?;
    update(descriptor, |style| style.stub = column.to_string())
}

/// Assigns escaped display labels to columns.
pub fn cols_label(
    descriptor: &str,
    columns: &[String],
    labels: &[String],
) -> Result<String, TerlanPolarsError> {
    if columns.is_empty() || columns.len() != labels.len() {
        return Err(style_error(
            "column labels require equally sized, non-empty column and label lists",
        ));
    }
    for column in columns {
        require_name(column, "labeled column")?;
    }
    update(descriptor, |style| {
        for (column, label) in columns.iter().zip(labels) {
            style.labels.retain(|entry| entry.column != *column);
            style.labels.push(ColumnLabel {
                column: column.clone(),
                label: label.clone(),
            });
        }
    })
}

/// Sets escaped table title and subtitle text.
pub fn tab_header(
    descriptor: &str,
    title: &str,
    subtitle: &str,
) -> Result<String, TerlanPolarsError> {
    update(descriptor, |style| {
        style.title = title.to_string();
        style.subtitle = subtitle.to_string();
    })
}

/// Applies fixed-point number formatting to selected columns.
pub fn fmt_number(
    descriptor: &str,
    columns: &[String],
    decimals: i64,
    separators: bool,
) -> Result<String, TerlanPolarsError> {
    validate_columns(columns)?;
    let decimals = usize::try_from(decimals)
        .ok()
        .filter(|value| *value <= 12)
        .ok_or_else(|| style_error("number-format decimals must be between 0 and 12"))?;
    update(descriptor, |style| {
        style.numbers.push(NumberFormat {
            columns: columns.to_vec(),
            decimals,
            separators,
        });
    })
}

/// Replaces numeric cell text in selected columns with accessible inline SVG bars.
pub fn fmt_nanoplot(descriptor: &str, columns: &[String]) -> Result<String, TerlanPolarsError> {
    validate_columns(columns)?;
    update(descriptor, |style| {
        for column in columns {
            if !style.nanoplots.contains(column) {
                style.nanoplots.push(column.clone());
            }
        }
    })
}

/// Adds a two-color numeric background scale to selected columns.
pub fn data_color(
    descriptor: &str,
    columns: &[String],
    low: &str,
    high: &str,
) -> Result<String, TerlanPolarsError> {
    validate_columns(columns)?;
    parse_hex_color(low)?;
    parse_hex_color(high)?;
    update(descriptor, |style| {
        style.colors.push(ColorScale {
            columns: columns.to_vec(),
            low: normalize_color(low),
            high: normalize_color(high),
        });
    })
}

fn update(
    descriptor: &str,
    mutate: impl FnOnce(&mut TableStyle),
) -> Result<String, TerlanPolarsError> {
    let mut style = decode(descriptor)?;
    mutate(&mut style);
    encode(&style)
}

fn encode(style: &TableStyle) -> Result<String, TerlanPolarsError> {
    let encoded = serde_json::to_string(style).map_err(|error| style_error(error.to_string()))?;
    if encoded.len() > MAX_STYLE_BYTES {
        return Err(style_error(format!(
            "table-style descriptor exceeds {MAX_STYLE_BYTES} bytes"
        )));
    }
    Ok(encoded)
}

fn decode(descriptor: &str) -> Result<TableStyle, TerlanPolarsError> {
    if descriptor.len() > MAX_STYLE_BYTES {
        return Err(style_error(format!(
            "table-style descriptor exceeds {MAX_STYLE_BYTES} bytes"
        )));
    }
    let style: TableStyle = serde_json::from_str(descriptor)
        .map_err(|_| style_error("invalid table-style descriptor"))?;
    if style.version != STYLE_VERSION {
        return Err(style_error("unsupported table-style descriptor version"));
    }
    Ok(style)
}

fn validate_columns(columns: &[String]) -> Result<(), TerlanPolarsError> {
    if columns.is_empty() {
        return Err(style_error("at least one table column is required"));
    }
    for column in columns {
        require_name(column, "table column")?;
    }
    Ok(())
}

fn require_name(value: &str, label: &str) -> Result<(), TerlanPolarsError> {
    if value.is_empty() {
        Err(style_error(format!("{label} cannot be empty")))
    } else {
        Ok(())
    }
}

fn style_error(message: impl Into<String>) -> TerlanPolarsError {
    TerlanPolarsError::new("invalid_table_style", message)
}

fn normalize_color(value: &str) -> String {
    if value.len() == 4 {
        let bytes = value.as_bytes();
        format!(
            "#{0}{0}{1}{1}{2}{2}",
            (bytes[1] as char).to_ascii_uppercase(),
            (bytes[2] as char).to_ascii_uppercase(),
            (bytes[3] as char).to_ascii_uppercase()
        )
    } else {
        value.to_ascii_uppercase()
    }
}

fn parse_hex_color(value: &str) -> Result<(u8, u8, u8), TerlanPolarsError> {
    let normalized = normalize_color(value);
    if normalized.len() != 7
        || !normalized.starts_with('#')
        || !normalized[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(style_error(
            "table colors must use #RGB or #RRGGBB hexadecimal syntax",
        ));
    }
    let component = |range| u8::from_str_radix(&normalized[range], 16).unwrap();
    Ok((component(1..3), component(3..5), component(5..7)))
}

#[cfg(feature = "real-polars")]
fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(feature = "real-polars")]
fn format_number(value: f64, decimals: usize, separators: bool) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    let raw = format!("{value:.decimals$}");
    if !separators {
        return raw;
    }
    let (integer, fraction) = raw.split_once('.').unwrap_or((&raw, ""));
    let (sign, digits) = integer
        .strip_prefix('-')
        .map_or(("", integer), |digits| ("-", digits));
    let mut grouped = String::with_capacity(raw.len() + raw.len() / 3);
    grouped.push_str(sign);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if !fraction.is_empty() {
        grouped.push('.');
        grouped.push_str(fraction);
    }
    grouped
}

#[cfg(feature = "real-polars")]
fn numeric(value: &polars::prelude::AnyValue<'_>) -> Option<f64> {
    value.extract::<f64>().filter(|number| number.is_finite())
}

#[cfg(feature = "real-polars")]
fn extrema(
    dataframe: &polars::prelude::DataFrame,
    column: &str,
    rows: usize,
) -> Option<(f64, f64)> {
    let column = dataframe.column(column).ok()?;
    let mut range: Option<(f64, f64)> = None;
    for row in 0..rows {
        let value = column.get(row).ok()?;
        if let Some(value) = numeric(&value) {
            range = Some(match range {
                None => (value, value),
                Some((min, max)) => (min.min(value), max.max(value)),
            });
        }
    }
    range
}

#[cfg(feature = "real-polars")]
fn interpolate_color(low: &str, high: &str, ratio: f64) -> String {
    let low = parse_hex_color(low).expect("validated descriptor color");
    let high = parse_hex_color(high).expect("validated descriptor color");
    let ratio = ratio.clamp(0.0, 1.0);
    let mix = |left: u8, right: u8| {
        (f64::from(left) + (f64::from(right) - f64::from(left)) * ratio).round() as u8
    };
    format!(
        "#{:02X}{:02X}{:02X}",
        mix(low.0, high.0),
        mix(low.1, high.1),
        mix(low.2, high.2)
    )
}

/// Renders a styled DataFrame as a bounded, escaped HTML fragment.
#[cfg(feature = "real-polars")]
pub fn table_html(
    dataframe: &TerlanPolarsDataFrame,
    descriptor: &str,
    max_rows: i64,
) -> Result<Vec<u8>, TerlanPolarsError> {
    let style = decode(descriptor)?;
    let limit = usize::try_from(max_rows)
        .ok()
        .filter(|limit| *limit <= MAX_TABLE_ROWS)
        .ok_or_else(|| {
            TerlanPolarsError::new(
                "html_limit_exceeded",
                format!("HTML table row limit must be between 0 and {MAX_TABLE_ROWS}"),
            )
        })?;
    let rows = dataframe.inner.height().min(limit);
    let column_names = dataframe.inner.get_column_names();
    let has_column = |name: &str| column_names.iter().any(|column| column.as_str() == name);
    if !style.stub.is_empty() && !has_column(&style.stub) {
        return Err(style_error(format!(
            "stub column `{}` does not exist",
            style.stub
        )));
    }
    for name in style
        .labels
        .iter()
        .map(|entry| entry.column.as_str())
        .chain(
            style
                .numbers
                .iter()
                .flat_map(|entry| entry.columns.iter().map(String::as_str)),
        )
        .chain(style.nanoplots.iter().map(String::as_str))
        .chain(
            style
                .colors
                .iter()
                .flat_map(|entry| entry.columns.iter().map(String::as_str)),
        )
    {
        if !has_column(name) {
            return Err(style_error(format!(
                "styled column `{name}` does not exist"
            )));
        }
    }

    let mut html = String::with_capacity(rows.saturating_mul(dataframe.inner.width()) * 32 + 1024);
    html.push_str("<div class=\"terlan-polars-table\"><style>.terlan-polars-table{font-family:system-ui,sans-serif;max-width:100%;overflow:auto}.terlan-polars-table table{border-collapse:collapse;width:100%}.terlan-polars-table caption{text-align:left;padding:.5rem}.terlan-polars-table th,.terlan-polars-table td{border-bottom:1px solid #ddd;padding:.4rem .55rem;text-align:left}.terlan-polars-table thead th{border-bottom:2px solid #777}.terlan-polars-table .subtitle{display:block;font-size:.85em;font-weight:normal;color:#555}.terlan-polars-table svg{display:block;max-width:100%}</style><table>");
    if !style.title.is_empty() || !style.subtitle.is_empty() {
        html.push_str("<caption>");
        html.push_str(&escape_html(&style.title));
        if !style.subtitle.is_empty() {
            html.push_str("<span class=\"subtitle\">");
            html.push_str(&escape_html(&style.subtitle));
            html.push_str("</span>");
        }
        html.push_str("</caption>");
    }
    html.push_str("<thead><tr>");
    for name in &column_names {
        let label = style
            .labels
            .iter()
            .rev()
            .find(|entry| entry.column == name.as_str())
            .map_or(name.as_str(), |entry| entry.label.as_str());
        write!(html, "<th scope=\"col\">{}</th>", escape_html(label)).unwrap();
    }
    html.push_str("</tr></thead><tbody>");

    let color_ranges: Vec<(&ColorScale, Vec<(&str, Option<(f64, f64)>)>)> = style
        .colors
        .iter()
        .map(|scale| {
            let ranges = scale
                .columns
                .iter()
                .map(|column| (column.as_str(), extrema(&dataframe.inner, column, rows)))
                .collect();
            (scale, ranges)
        })
        .collect();
    let nanoplot_ranges: Vec<(&str, Option<(f64, f64)>)> = style
        .nanoplots
        .iter()
        .map(|column| (column.as_str(), extrema(&dataframe.inner, column, rows)))
        .collect();

    for row in 0..rows {
        html.push_str("<tr>");
        for (index, name) in column_names.iter().enumerate() {
            let value = dataframe.inner[index]
                .get(row)
                .map_err(|error| TerlanPolarsError::new("polars_error", error.to_string()))?;
            let number = numeric(&value);
            let mut cell = match value {
                polars::prelude::AnyValue::Null => String::new(),
                polars::prelude::AnyValue::String(value) => value.to_string(),
                polars::prelude::AnyValue::StringOwned(value) => value.to_string(),
                _ => value.to_string(),
            };
            if let Some(number) = number {
                if let Some(format) = style
                    .numbers
                    .iter()
                    .rev()
                    .find(|format| format.columns.iter().any(|column| column == name.as_str()))
                {
                    cell = format_number(number, format.decimals, format.separators);
                }
            }
            let color = number.and_then(|number| {
                color_ranges.iter().rev().find_map(|(scale, ranges)| {
                    ranges
                        .iter()
                        .find(|(column, _)| *column == name.as_str())
                        .map(|(_, range)| {
                            let ratio = range.map_or(0.5, |(min, max)| {
                                if min == max {
                                    0.5
                                } else {
                                    (number - min) / (max - min)
                                }
                            });
                            interpolate_color(&scale.low, &scale.high, ratio)
                        })
                })
            });
            let is_stub = style.stub == name.as_str();
            let tag = if is_stub { "th" } else { "td" };
            html.push('<');
            html.push_str(tag);
            if is_stub {
                html.push_str(" scope=\"row\"");
            }
            if let Some(color) = color {
                write!(html, " style=\"background-color:{color}\"").unwrap();
            }
            html.push('>');
            if let (Some(number), Some((_, range))) = (
                number,
                nanoplot_ranges
                    .iter()
                    .find(|(column, _)| *column == name.as_str()),
            ) {
                let ratio = range.map_or(0.5, |(min, max)| {
                    if min == max {
                        0.5
                    } else {
                        (number - min) / (max - min)
                    }
                });
                let width = (ratio.clamp(0.0, 1.0) * 96.0).round();
                write!(html, "<svg viewBox=\"0 0 100 14\" role=\"img\" aria-label=\"{}\"><rect x=\"2\" y=\"3\" width=\"{width}\" height=\"8\" fill=\"#4472C4\"/></svg>", escape_html(&cell)).unwrap();
            } else {
                html.push_str(&escape_html(&cell));
            }
            write!(html, "</{tag}>").unwrap();
        }
        html.push_str("</tr>");
        if html.len() > MAX_HTML_BYTES {
            return Err(TerlanPolarsError::new(
                "html_limit_exceeded",
                format!("HTML table exceeds {MAX_HTML_BYTES} bytes"),
            ));
        }
    }
    html.push_str("</tbody></table></div>");
    if dataframe.inner.height() > rows {
        write!(
            html,
            "<!-- {} of {} rows rendered -->",
            rows,
            dataframe.inner.height()
        )
        .unwrap();
    }
    if html.len() > MAX_HTML_BYTES {
        return Err(TerlanPolarsError::new(
            "html_limit_exceeded",
            format!("HTML table exceeds {MAX_HTML_BYTES} bytes"),
        ));
    }
    Ok(html.into_bytes())
}

/// Reports the unavailable real-Polars renderer in the skeleton build.
#[cfg(not(feature = "real-polars"))]
pub fn table_html(
    _dataframe: &TerlanPolarsDataFrame,
    descriptor: &str,
    max_rows: i64,
) -> Result<Vec<u8>, TerlanPolarsError> {
    let _ = decode(descriptor)?;
    let _ = max_rows;
    Err(TerlanPolarsError::new(
        "native_unavailable",
        "HTML table rendering requires the Rust native adapter target capability",
    ))
}

#[cfg(all(test, feature = "real-polars"))]
mod tests {
    use super::*;
    use polars::df;

    #[test]
    fn renders_escaped_styled_html() {
        let dataframe = TerlanPolarsDataFrame {
            inner: df![
                "name" => ["<script>alert('x')</script>", "safe & sound"],
                "value" => [1234.5, 2500.0]
            ]
            .unwrap(),
        };
        let mut style = gt();
        style = tab_header(&style, "<Title>", "A & B").unwrap();
        style = tab_stub(&style, "name").unwrap();
        style = cols_label(&style, &["value".into()], &["Value <USD>".into()]).unwrap();
        style = fmt_number(&style, &["value".into()], 2, true).unwrap();
        style = data_color(&style, &["value".into()], "#fff", "#336699").unwrap();
        let html = String::from_utf8(table_html(&dataframe, &style, 1).unwrap()).unwrap();

        assert!(html.contains("&lt;Title&gt;"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("1,234.50"));
        assert!(html.contains("scope=\"row\""));
        assert!(html.contains("1 of 2 rows rendered"));
    }

    #[test]
    fn renders_accessible_nanoplots_and_rejects_css_injection() {
        let dataframe = TerlanPolarsDataFrame {
            inner: df!["value" => [1.0, 2.0]].unwrap(),
        };
        let style = fmt_nanoplot(&gt(), &["value".into()]).unwrap();
        let html = String::from_utf8(table_html(&dataframe, &style, 2).unwrap()).unwrap();
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("<rect"));
        assert!(data_color(&gt(), &["value".into()], "red;display:none", "#fff").is_err());
    }
}
