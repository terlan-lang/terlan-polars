//! Native worker dispatch for portable Polars data-type operations.

use super::*;
use terlan_polars_native::data_types::*;

impl Worker {
    /// Executes one data-type operation after constructor dispatch.
    pub(super) fn data_type_operation(&self, operation: &str, args: Vec<NativeArg>) -> String {
        match operation {
            "polars.dtype.pretty_format" => {
                data_type_text_arg(args, "pretty_format", data_type_pretty_format)
            }
            "polars.dtype.is_known" => data_type_bool_arg(args, "is_known", data_type_is_known),
            "polars.dtype.get_shape" => {
                data_type_result_ints_arg(args, "array_shape", data_type_shape)
            }
            "polars.dtype.inner_dtype" => {
                data_type_result_text_arg(args, "inner_data_type", data_type_inner)
            }
            "polars.dtype.map_key" => {
                data_type_result_text_arg(args, "map_key_data_type", data_type_map_key)
            }
            "polars.dtype.map_value" => {
                data_type_result_text_arg(args, "map_value_data_type", data_type_map_value)
            }
            "polars.dtype.leaf_dtype" => data_type_text_arg(args, "leaf_data_type", data_type_leaf),
            "polars.dtype.array_leaf_dtype" => {
                data_type_result_text_arg(args, "array_leaf_data_type", data_type_array_leaf)
            }
            "polars.dtype.cast_leaf" => {
                data_type_two_text_arg(args, "cast_leaf", data_type_cast_leaf)
            }
            "polars.dtype.can_cast_to" => {
                data_type_result_two_text_bool_arg(args, "can_cast_to", data_type_can_cast_to)
            }
            "polars.dtype.implode" => data_type_text_arg(args, "implode", data_type_implode),
            "polars.dtype.to_physical" => {
                data_type_text_arg(args, "data_type_to_physical", data_type_to_physical)
            }
            "polars.dtype.to_storage" => {
                data_type_text_arg(args, "data_type_to_storage", data_type_to_storage)
            }
            "polars.dtype.is_supported_list_arithmetic_input" => data_type_bool_arg(
                args,
                "is_supported_list_arithmetic_input",
                data_type_is_supported_list_arithmetic_input,
            ),
            "polars.dtype.is_logical" => {
                data_type_bool_arg(args, "is_logical", data_type_is_logical)
            }
            "polars.dtype.is_temporal" => {
                data_type_bool_arg(args, "is_temporal", data_type_is_temporal)
            }
            "polars.dtype.is_primitive" => {
                data_type_bool_arg(args, "is_primitive", data_type_is_primitive)
            }
            "polars.dtype.is_primitive_numeric" => {
                data_type_bool_arg(args, "is_primitive_numeric", data_type_is_primitive_numeric)
            }
            "polars.dtype.is_bool" => data_type_bool_arg(args, "is_bool", data_type_is_bool),
            "polars.dtype.is_list" => data_type_bool_arg(args, "is_list", data_type_is_list),
            "polars.dtype.is_map" => data_type_bool_arg(args, "is_map", data_type_is_map),
            "polars.dtype.is_array" => data_type_bool_arg(args, "is_array", data_type_is_array),
            "polars.dtype.is_nested" => data_type_bool_arg(args, "is_nested", data_type_is_nested),
            "polars.dtype.is_struct" => data_type_bool_arg(args, "is_struct", data_type_is_struct),
            "polars.dtype.is_binary" => data_type_bool_arg(args, "is_binary", data_type_is_binary),
            "polars.dtype.is_date" => data_type_bool_arg(args, "is_date", data_type_is_date),
            "polars.dtype.is_datetime" => {
                data_type_bool_arg(args, "is_datetime", data_type_is_datetime)
            }
            "polars.dtype.is_duration" => {
                data_type_bool_arg(args, "is_duration", data_type_is_duration)
            }
            "polars.dtype.is_object" => data_type_bool_arg(args, "is_object", data_type_is_object),
            "polars.dtype.is_null" => {
                data_type_bool_arg(args, "data_type_is_null", data_type_is_null)
            }
            "polars.dtype.contains_views" => {
                data_type_bool_arg(args, "contains_views", data_type_contains_views)
            }
            "polars.dtype.contains_categoricals" => data_type_bool_arg(
                args,
                "contains_categoricals",
                data_type_contains_categoricals,
            ),
            "polars.dtype.contains_enums" => {
                data_type_bool_arg(args, "contains_enums", data_type_contains_enums)
            }
            "polars.dtype.contains_objects" => {
                data_type_bool_arg(args, "contains_objects", data_type_contains_objects)
            }
            "polars.dtype.contains_list_recursive" => {
                data_type_bool_arg(args, "contains_list", data_type_contains_list_recursive)
            }
            "polars.dtype.contains_unknown" => {
                data_type_bool_arg(args, "contains_unknown", data_type_contains_unknown)
            }
            "polars.dtype.contains_dtype_recursive" => {
                data_type_two_text_bool_arg(args, "contains_data_type", data_type_contains)
            }
            "polars.dtype.is_ord" => data_type_bool_arg(args, "is_ordered", data_type_is_ord),
            "polars.dtype.is_decimal" => {
                data_type_bool_arg(args, "is_decimal", data_type_is_decimal)
            }
            "polars.dtype.is_float" => data_type_bool_arg(args, "is_float", data_type_is_float),
            "polars.dtype.is_integer" => {
                data_type_bool_arg(args, "is_integer", data_type_is_integer)
            }
            "polars.dtype.is_signed_integer" => {
                data_type_bool_arg(args, "is_signed_integer", data_type_is_signed_integer)
            }
            "polars.dtype.is_unsigned_integer" => {
                data_type_bool_arg(args, "is_unsigned_integer", data_type_is_unsigned_integer)
            }
            "polars.dtype.is_string" => data_type_bool_arg(args, "is_string", data_type_is_string),
            "polars.dtype.is_categorical" => {
                data_type_bool_arg(args, "is_categorical", data_type_is_categorical)
            }
            "polars.dtype.is_enum" => data_type_bool_arg(args, "is_enum", data_type_is_enum),
            "polars.dtype.is_extension" => {
                data_type_bool_arg(args, "is_extension", data_type_is_extension)
            }
            "polars.dtype.max" => data_type_result_text_arg(args, "data_type_max", data_type_max),
            "polars.dtype.min" => data_type_result_text_arg(args, "data_type_min", data_type_min),
            "polars.dtype.is_nested_null" => {
                data_type_bool_arg(args, "is_nested_null", data_type_is_nested_null)
            }
            "polars.dtype.matches_schema_type" => data_type_result_two_text_bool_arg(
                args,
                "matches_schema_type",
                data_type_matches_schema,
            ),
            "polars.dtype.is_unknown" => {
                data_type_bool_arg(args, "is_unknown", data_type_is_unknown)
            }
            "polars.dtype.nesting_level" => {
                data_type_int_arg(args, "nesting_level", data_type_nesting_level)
            }
            "polars.dtype.is_numeric" => {
                data_type_bool_arg(args, "is_numeric", data_type_is_numeric)
            }
            "polars.dtype.numeric_to_unsigned_bit_repr" => data_type_result_text_arg(
                args,
                "unsigned_bit_representation",
                data_type_unsigned_bit_repr,
            ),
            _ => protocol_error(
                "native_operation_unknown",
                &format!("unsupported data-type operation `{operation}`"),
            ),
        }
    }
}

/// Decodes one data-type argument and returns a plain string.
fn data_type_text_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<String, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(value)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects one data type"));
    };
    expression_reply(function(value))
}

/// Decodes two data-type arguments and returns a plain string.
fn data_type_two_text_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str) -> Result<String, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(value), NativeArg::Text(target)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects two data types"));
    };
    expression_reply(function(value, target))
}

/// Decodes one data-type argument and returns a plain Boolean.
fn data_type_bool_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<bool, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(value)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects one data type"));
    };
    expression_bool_reply(function(value))
}

/// Decodes two data-type arguments and returns a plain Boolean.
fn data_type_two_text_bool_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str) -> Result<bool, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(value), NativeArg::Text(target)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects two data types"));
    };
    expression_bool_reply(function(value, target))
}

/// Decodes one data-type argument and returns a plain integer.
fn data_type_int_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<i64, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(value)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects one data type"));
    };
    match function(value) {
        Ok(value) => format!("ok_int {value}"),
        Err(error) => protocol_error(error.code(), error.message()),
    }
}

/// Decodes one data-type argument and returns a typed string Result.
fn data_type_result_text_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<String, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(value)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects one data type"));
    };
    expression_result_reply(function(value))
}

/// Decodes one data-type argument and returns a typed integer-list Result.
fn data_type_result_ints_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<Vec<i64>, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(value)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects one data type"));
    };
    match function(value) {
        Ok(values) if values.is_empty() => "result_ok_ints".to_string(),
        Ok(values) => format!(
            "result_ok_ints {}",
            values
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Err(error) => adapter_result_error(error.code(), error.message()),
    }
}

/// Decodes two data-type arguments and returns a typed Boolean Result.
fn data_type_result_two_text_bool_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str) -> Result<bool, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(value), NativeArg::Text(target)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects two data types"));
    };
    match function(value, target) {
        Ok(value) => format!("result_ok_bool {value}"),
        Err(error) => adapter_result_error(error.code(), error.message()),
    }
}
