//! Safe declarative user-defined expressions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::expressions::{
    expr_strict_cast, expression_from_value, expression_parameter, expression_value,
};
use crate::TerlanPolarsError;

const UDF_FORMAT_VERSION: u8 = 1;
const MAX_UDF_ARITY: usize = 64;

/// Serialized declarative expression template.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpressionUdf {
    /// Format version used to reject incompatible serialized templates.
    version: u8,
    /// Exact number of arguments accepted by the template.
    arity: usize,
    /// Expression tree containing indexed parameter nodes.
    template: Value,
    /// Polars data type enforced on the applied expression.
    output_type: String,
}

/// Creates an indexed parameter for a declarative expression UDF.
pub fn expression_udf_parameter(index: i64) -> Result<String, TerlanPolarsError> {
    let index = usize::try_from(index).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_udf_parameter",
            "expression UDF parameter index must be non-negative",
        )
    })?;
    if index >= MAX_UDF_ARITY {
        return Err(TerlanPolarsError::new(
            "invalid_udf_parameter",
            format!("expression UDF parameter index must be below {MAX_UDF_ARITY}"),
        ));
    }
    expression_parameter(index)
}

/// Defines an immutable expression UDF from a parameterized expression.
pub fn define_expression_udf(
    arity: i64,
    output_type: &str,
    template: &str,
) -> Result<String, TerlanPolarsError> {
    let arity = validate_arity(arity)?;
    validate_output_type(output_type)?;
    let template = expression_value(template)?;
    validate_parameters(&template, arity)?;
    serde_json::to_string(&ExpressionUdf {
        version: UDF_FORMAT_VERSION,
        arity,
        template,
        output_type: output_type.into(),
    })
    .map_err(|error| {
        TerlanPolarsError::new(
            "invalid_udf",
            format!("cannot encode expression UDF: {error}"),
        )
    })
}

/// Applies a declarative expression UDF by substituting its indexed parameters.
pub fn apply_expression_udf(
    encoded_udf: &str,
    arguments: &[String],
) -> Result<String, TerlanPolarsError> {
    let udf: ExpressionUdf = serde_json::from_str(encoded_udf).map_err(|error| {
        TerlanPolarsError::new(
            "invalid_udf",
            format!("cannot decode expression UDF: {error}"),
        )
    })?;
    if udf.version != UDF_FORMAT_VERSION {
        return Err(TerlanPolarsError::new(
            "invalid_udf",
            format!("unsupported expression UDF version {}", udf.version),
        ));
    }
    if arguments.len() != udf.arity {
        return Err(TerlanPolarsError::new(
            "udf_arity_mismatch",
            format!(
                "expression UDF expects {} arguments but received {}",
                udf.arity,
                arguments.len()
            ),
        ));
    }
    let arguments = arguments
        .iter()
        .map(|argument| expression_value(argument))
        .collect::<Result<Vec<_>, _>>()?;
    let substituted = substitute_parameters(udf.template, &arguments)?;
    let expression = expression_from_value(substituted)?;
    expr_strict_cast(&expression, &udf.output_type)
}

/// Rejects empty or invalid UDF output data types.
fn validate_output_type(output_type: &str) -> Result<(), TerlanPolarsError> {
    if output_type.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_udf_output_type",
            "expression UDF output type cannot be empty",
        ));
    }
    #[cfg(feature = "real-polars")]
    crate::expressions::parse_data_type(output_type).map_err(|error| {
        TerlanPolarsError::new(
            "invalid_udf_output_type",
            format!("invalid expression UDF output type: {}", error.message()),
        )
    })?;
    Ok(())
}

/// Validates and converts a public signed UDF arity.
fn validate_arity(arity: i64) -> Result<usize, TerlanPolarsError> {
    let arity = usize::try_from(arity).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_udf_arity",
            "expression UDF arity must be non-negative",
        )
    })?;
    if arity > MAX_UDF_ARITY {
        return Err(TerlanPolarsError::new(
            "invalid_udf_arity",
            format!("expression UDF arity must not exceed {MAX_UDF_ARITY}"),
        ));
    }
    Ok(arity)
}

/// Rejects parameter indexes outside the declared UDF arity.
fn validate_parameters(template: &Value, arity: usize) -> Result<(), TerlanPolarsError> {
    visit_parameters(template, &mut |index| {
        if index >= arity {
            Err(TerlanPolarsError::new(
                "invalid_udf_parameter",
                format!("expression UDF parameter {index} is outside declared arity {arity}"),
            ))
        } else {
            Ok(())
        }
    })
}

/// Replaces every parameter node in an expression tree with its argument tree.
fn substitute_parameters(
    mut template: Value,
    arguments: &[Value],
) -> Result<Value, TerlanPolarsError> {
    substitute_value(&mut template, arguments)?;
    Ok(template)
}

/// Visits all parameter nodes in a serialized expression tree.
fn visit_parameters(
    value: &Value,
    visitor: &mut impl FnMut(usize) -> Result<(), TerlanPolarsError>,
) -> Result<(), TerlanPolarsError> {
    if let Some(index) = parameter_index(value)? {
        return visitor(index);
    }
    match value {
        Value::Array(values) => {
            for value in values {
                visit_parameters(value, visitor)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                visit_parameters(value, visitor)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Substitutes parameter nodes within one JSON expression subtree.
fn substitute_value(value: &mut Value, arguments: &[Value]) -> Result<(), TerlanPolarsError> {
    if let Some(index) = parameter_index(value)? {
        let argument = arguments.get(index).ok_or_else(|| {
            TerlanPolarsError::new(
                "invalid_udf_parameter",
                format!("expression UDF parameter {index} has no argument"),
            )
        })?;
        *value = argument.clone();
        return Ok(());
    }
    match value {
        Value::Array(values) => {
            for value in values {
                substitute_value(value, arguments)?;
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                substitute_value(value, arguments)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Returns an expression parameter index when the value is a parameter node.
fn parameter_index(value: &Value) -> Result<Option<usize>, TerlanPolarsError> {
    let Value::Object(fields) = value else {
        return Ok(None);
    };
    if fields.get("op").and_then(Value::as_str) != Some("parameter") {
        return Ok(None);
    }
    let index = fields
        .get("index")
        .and_then(Value::as_u64)
        .ok_or_else(|| TerlanPolarsError::new("invalid_udf", "invalid UDF parameter node"))?;
    usize::try_from(index)
        .map(Some)
        .map_err(|_| TerlanPolarsError::new("invalid_udf", "UDF parameter index is too large"))
}
