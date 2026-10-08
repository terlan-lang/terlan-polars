//! Versioned, bounded descriptors for Polars-compatible plot options.

use serde::{Deserialize, Serialize};

use crate::TerlanPolarsError;

const OPTIONS_VERSION: u8 = 1;
const MAX_OPTIONS_BYTES: usize = 16 * 1024;
const MAX_TEXT_BYTES: usize = 1_024;
pub(crate) const MAX_FACETS: usize = 8;
const MIN_DIMENSION: usize = 64;
const MAX_DIMENSION: usize = 4_096;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PlotOptions {
    version: u8,
    pub(crate) title: String,
    pub(crate) color: String,
    pub(crate) facet: String,
    pub(crate) facet_columns: Option<usize>,
    pub(crate) width: Option<usize>,
    pub(crate) height: Option<usize>,
}

impl Default for PlotOptions {
    fn default() -> Self {
        Self {
            version: OPTIONS_VERSION,
            title: String::new(),
            color: String::new(),
            facet: String::new(),
            facet_columns: None,
            width: None,
            height: None,
        }
    }
}

/// Creates an empty immutable plot-options descriptor.
pub fn plot_options() -> String {
    encode(&PlotOptions::default()).expect("the default plot options are serializable")
}

/// Sets the plot title. An empty title removes it.
pub fn with_plot_title(descriptor: &str, title: &str) -> Result<String, TerlanPolarsError> {
    validate_text(title, "plot title")?;
    update(descriptor, |options| options.title = title.to_string())
}

/// Sets the categorical color/group column. An empty name removes grouping.
pub fn with_plot_color(descriptor: &str, column: &str) -> Result<String, TerlanPolarsError> {
    validate_text(column, "plot color column")?;
    update(descriptor, |options| options.color = column.to_string())
}

/// Sets the bar-facet column and optional number of facet columns.
///
/// A zero `columns` value uses the backend's automatic layout. An empty facet
/// column clears faceting and its column count.
pub fn with_plot_facet(
    descriptor: &str,
    column: &str,
    columns: i64,
) -> Result<String, TerlanPolarsError> {
    validate_text(column, "plot facet column")?;
    let facet_columns = if column.is_empty() || columns == 0 {
        None
    } else {
        Some(
            usize::try_from(columns)
                .ok()
                .filter(|value| (1..=MAX_FACETS).contains(value))
                .ok_or_else(|| {
                    options_error(format!(
                        "facet columns must be between 1 and {MAX_FACETS}, or zero for automatic layout"
                    ))
                })?,
        )
    };
    update(descriptor, |options| {
        options.facet = column.to_string();
        options.facet_columns = facet_columns;
    })
}

/// Sets fixed output dimensions in pixels.
pub fn with_plot_dimensions(
    descriptor: &str,
    width: i64,
    height: i64,
) -> Result<String, TerlanPolarsError> {
    let width = validate_dimension(width, "plot width")?;
    let height = validate_dimension(height, "plot height")?;
    update(descriptor, |options| {
        options.width = Some(width);
        options.height = Some(height);
    })
}

pub(crate) fn decode_plot_options(descriptor: &str) -> Result<PlotOptions, TerlanPolarsError> {
    if descriptor.len() > MAX_OPTIONS_BYTES {
        return Err(options_error(format!(
            "plot-options descriptor exceeds {MAX_OPTIONS_BYTES} bytes"
        )));
    }
    let options: PlotOptions = serde_json::from_str(descriptor)
        .map_err(|_| options_error("invalid plot-options descriptor"))?;
    if options.version != OPTIONS_VERSION {
        return Err(options_error("unsupported plot-options descriptor version"));
    }
    validate_text(&options.title, "plot title")?;
    validate_text(&options.color, "plot color column")?;
    validate_text(&options.facet, "plot facet column")?;
    if options.facet.is_empty() && options.facet_columns.is_some() {
        return Err(options_error(
            "facet columns require a non-empty facet column",
        ));
    }
    if let Some(columns) = options.facet_columns {
        if !(1..=MAX_FACETS).contains(&columns) {
            return Err(options_error(format!(
                "facet columns must be between 1 and {MAX_FACETS}"
            )));
        }
    }
    match (options.width, options.height) {
        (Some(width), Some(height)) => {
            validate_dimension(width as i64, "plot width")?;
            validate_dimension(height as i64, "plot height")?;
        }
        (None, None) => {}
        _ => {
            return Err(options_error(
                "plot width and height must be specified together",
            ));
        }
    }
    Ok(options)
}

fn update(
    descriptor: &str,
    mutate: impl FnOnce(&mut PlotOptions),
) -> Result<String, TerlanPolarsError> {
    let mut options = decode_plot_options(descriptor)?;
    mutate(&mut options);
    encode(&options)
}

fn encode(options: &PlotOptions) -> Result<String, TerlanPolarsError> {
    let encoded =
        serde_json::to_string(options).map_err(|error| options_error(error.to_string()))?;
    if encoded.len() > MAX_OPTIONS_BYTES {
        return Err(options_error(format!(
            "plot-options descriptor exceeds {MAX_OPTIONS_BYTES} bytes"
        )));
    }
    Ok(encoded)
}

fn validate_text(value: &str, label: &str) -> Result<(), TerlanPolarsError> {
    if value.len() > MAX_TEXT_BYTES {
        Err(options_error(format!(
            "{label} exceeds {MAX_TEXT_BYTES} UTF-8 bytes"
        )))
    } else {
        Ok(())
    }
}

fn validate_dimension(value: i64, label: &str) -> Result<usize, TerlanPolarsError> {
    usize::try_from(value)
        .ok()
        .filter(|value| (MIN_DIMENSION..=MAX_DIMENSION).contains(value))
        .ok_or_else(|| {
            options_error(format!(
                "{label} must be between {MIN_DIMENSION} and {MAX_DIMENSION} pixels"
            ))
        })
}

fn options_error(message: impl Into<String>) -> TerlanPolarsError {
    TerlanPolarsError::new("invalid_plot_options", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composes_and_decodes_plot_options() {
        let descriptor = with_plot_title(&plot_options(), "Sales").unwrap();
        let descriptor = with_plot_color(&descriptor, "region").unwrap();
        let descriptor = with_plot_facet(&descriptor, "year", 2).unwrap();
        let descriptor = with_plot_dimensions(&descriptor, 960, 540).unwrap();
        let options = decode_plot_options(&descriptor).unwrap();
        assert_eq!(options.title, "Sales");
        assert_eq!(options.color, "region");
        assert_eq!(options.facet, "year");
        assert_eq!(options.facet_columns, Some(2));
        assert_eq!((options.width, options.height), (Some(960), Some(540)));
    }

    #[test]
    fn rejects_invalid_descriptors_and_dimensions() {
        assert_eq!(
            decode_plot_options("{}").unwrap_err().code(),
            "invalid_plot_options"
        );
        assert_eq!(
            with_plot_dimensions(&plot_options(), 32, 600)
                .unwrap_err()
                .code(),
            "invalid_plot_options"
        );
        assert_eq!(
            with_plot_facet(&plot_options(), "year", 9)
                .unwrap_err()
                .code(),
            "invalid_plot_options"
        );
    }
}
