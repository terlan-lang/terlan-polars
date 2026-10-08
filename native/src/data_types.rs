//! Portable Polars data-type inspection and transformation.
//!
//! The public functions in this module operate on the package's canonical
//! string representation. They keep Polars-owned category mappings, Arrow
//! fields, and Rust callbacks behind the native package boundary.

use crate::TerlanPolarsError;

/// Returns the stable error used when the real Polars feature is unavailable.
#[cfg(not(feature = "real-polars"))]
fn unavailable<T>() -> Result<T, TerlanPolarsError> {
    Err(TerlanPolarsError::new(
        "native_unavailable",
        "data-type operations require the real-polars feature",
    ))
}

/// Encodes one Polars data type in the package's canonical representation.
#[cfg(feature = "real-polars")]
pub(crate) fn encode_data_type(
    data_type: &polars::prelude::DataType,
) -> Result<String, TerlanPolarsError> {
    use polars::prelude::{DataType, TimeUnit};

    let scalar = match data_type {
        DataType::Boolean => Some("Boolean"),
        DataType::UInt8 => Some("UInt8"),
        DataType::UInt16 => Some("UInt16"),
        DataType::UInt32 => Some("UInt32"),
        DataType::UInt64 => Some("UInt64"),
        DataType::UInt128 => Some("UInt128"),
        DataType::Int8 => Some("Int8"),
        DataType::Int16 => Some("Int16"),
        DataType::Int32 => Some("Int32"),
        DataType::Int64 => Some("Int64"),
        DataType::Int128 => Some("Int128"),
        DataType::Float16 => Some("Float16"),
        DataType::Float32 => Some("Float32"),
        DataType::Float64 => Some("Float64"),
        DataType::String => Some("Utf8"),
        DataType::Binary => Some("BinaryType"),
        DataType::BinaryOffset => Some("BinaryOffsetType"),
        DataType::Date => Some("DateType"),
        DataType::Time => Some("TimeType"),
        DataType::Datetime(TimeUnit::Milliseconds, None) => Some("DatetimeMilliseconds"),
        DataType::Datetime(TimeUnit::Microseconds, None) => Some("DatetimeMicroseconds"),
        DataType::Datetime(TimeUnit::Nanoseconds, None) => Some("DatetimeNanoseconds"),
        DataType::Duration(TimeUnit::Milliseconds) => Some("DurationMilliseconds"),
        DataType::Duration(TimeUnit::Microseconds) => Some("DurationMicroseconds"),
        DataType::Duration(TimeUnit::Nanoseconds) => Some("DurationNanoseconds"),
        DataType::Null => Some("NullType"),
        _ => None,
    };
    if let Some(scalar) = scalar {
        return Ok(scalar.to_string());
    }

    match data_type {
        DataType::List(element) => crate::expressions::data_type_list(&encode_data_type(element)?),
        DataType::Map(key, value) => {
            crate::expressions::data_type_map(&encode_data_type(key)?, &encode_data_type(value)?)
        }
        DataType::Array(element, width) => crate::expressions::data_type_array(
            &encode_data_type(element)?,
            i64::try_from(*width).map_err(|_| {
                TerlanPolarsError::new("invalid_data_type", "array width exceeds Int range")
            })?,
        ),
        DataType::Decimal(precision, scale) => crate::expressions::data_type_decimal(
            i64::try_from(*precision).map_err(|_| {
                TerlanPolarsError::new("invalid_data_type", "decimal precision exceeds Int range")
            })?,
            i64::try_from(*scale).map_err(|_| {
                TerlanPolarsError::new("invalid_data_type", "decimal scale exceeds Int range")
            })?,
        ),
        DataType::Categorical(_, _) => crate::expressions::data_type_categorical(),
        DataType::Enum(categories, _) => {
            let values = categories
                .categories()
                .values_iter()
                .map(str::to_string)
                .collect::<Vec<_>>();
            crate::expressions::data_type_enum(&values)
        }
        DataType::Datetime(unit, Some(time_zone)) => {
            crate::expressions::data_type_datetime(time_unit_name(*unit), time_zone.as_ref())
        }
        DataType::Struct(fields) => {
            let fields = fields
                .iter()
                .map(|field| {
                    crate::expressions::data_type_field(
                        field.name().as_str(),
                        &encode_data_type(field.dtype())?,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            crate::expressions::data_type_struct(&fields)
        }
        DataType::Extension(extension, storage) => {
            let storage = encode_data_type(storage)?;
            let name = extension.name();
            match extension.serialize_metadata() {
                Some(metadata) => crate::expressions::data_type_extension_metadata(
                    name.as_ref(),
                    &metadata,
                    &storage,
                ),
                None => crate::expressions::data_type_extension(name.as_ref(), &storage),
            }
        }
        DataType::Unknown(_) => Err(TerlanPolarsError::new(
            "unknown_data_type",
            "unknown Polars data types cannot cross the package boundary",
        )),
        _ => Err(TerlanPolarsError::new(
            "unsupported_data_type",
            format!("cannot encode Polars data type `{data_type}`"),
        )),
    }
}

/// Returns the canonical name used by data-type descriptors for a time unit.
#[cfg(feature = "real-polars")]
fn time_unit_name(unit: polars::prelude::TimeUnit) -> &'static str {
    match unit {
        polars::prelude::TimeUnit::Milliseconds => "milliseconds",
        polars::prelude::TimeUnit::Microseconds => "microseconds",
        polars::prelude::TimeUnit::Nanoseconds => "nanoseconds",
    }
}

/// Runs one read-only operation against a parsed Polars data type.
#[cfg(feature = "real-polars")]
fn inspect<T>(
    value: &str,
    operation: impl FnOnce(&polars::prelude::DataType) -> T,
) -> Result<T, TerlanPolarsError> {
    let data_type = crate::expressions::parse_data_type(value)?;
    Ok(operation(&data_type))
}

/// Formats a data type using Polars' human-readable nested representation.
pub fn data_type_pretty_format(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        inspect(value, polars::prelude::DataType::pretty_format)
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

macro_rules! data_type_predicates {
    ($(#[$meta:meta] $name:ident => $method:ident;)+) => {
        $(
            #[$meta]
            pub fn $name(value: &str) -> Result<bool, TerlanPolarsError> {
                #[cfg(feature = "real-polars")]
                {
                    inspect(value, polars::prelude::DataType::$method)
                }
                #[cfg(not(feature = "real-polars"))]
                {
                    let _ = value;
                    unavailable()
                }
            }
        )+
    };
}

data_type_predicates! {
    /// Returns whether the complete data type is known.
    data_type_is_known => is_known;
    /// Returns whether the data type is logical rather than physical.
    data_type_is_logical => is_logical;
    /// Returns whether the data type is temporal.
    data_type_is_temporal => is_temporal;
    /// Returns whether the data type is primitive.
    data_type_is_primitive => is_primitive;
    /// Returns whether the data type is a primitive numeric type.
    data_type_is_primitive_numeric => is_primitive_numeric;
    /// Returns whether the data type is Boolean.
    data_type_is_bool => is_bool;
    /// Returns whether the data type is List.
    data_type_is_list => is_list;
    /// Returns whether the data type is Map.
    data_type_is_map => is_map;
    /// Returns whether the data type is Array.
    data_type_is_array => is_array;
    /// Returns whether the data type is nested.
    data_type_is_nested => is_nested;
    /// Returns whether the data type is Struct.
    data_type_is_struct => is_struct;
    /// Returns whether the data type is Binary.
    data_type_is_binary => is_binary;
    /// Returns whether the data type is Date.
    data_type_is_date => is_date;
    /// Returns whether the data type is Datetime.
    data_type_is_datetime => is_datetime;
    /// Returns whether the data type is Duration.
    data_type_is_duration => is_duration;
    /// Returns whether the data type is an Object type.
    data_type_is_object => is_object;
    /// Returns whether the data type is Null.
    data_type_is_null => is_null;
    /// Returns whether the type tree contains String or Binary views.
    data_type_contains_views => contains_views;
    /// Returns whether the type tree contains Categorical storage.
    data_type_contains_categoricals => contains_categoricals;
    /// Returns whether the type tree contains Enum storage.
    data_type_contains_enums => contains_enums;
    /// Returns whether the type tree contains Object storage.
    data_type_contains_objects => contains_objects;
    /// Returns whether the type tree contains a List.
    data_type_contains_list_recursive => contains_list_recursive;
    /// Returns whether the type tree contains an unknown type.
    data_type_contains_unknown => contains_unknown;
    /// Returns whether values of this type have a total ordering.
    data_type_is_ord => is_ord;
    /// Returns whether the data type is Decimal.
    data_type_is_decimal => is_decimal;
    /// Returns whether the data type is floating point.
    data_type_is_float => is_float;
    /// Returns whether the data type is an integer.
    data_type_is_integer => is_integer;
    /// Returns whether the data type is a signed integer.
    data_type_is_signed_integer => is_signed_integer;
    /// Returns whether the data type is an unsigned integer.
    data_type_is_unsigned_integer => is_unsigned_integer;
    /// Returns whether the data type is String.
    data_type_is_string => is_string;
    /// Returns whether the data type is Categorical.
    data_type_is_categorical => is_categorical;
    /// Returns whether the data type is Enum.
    data_type_is_enum => is_enum;
    /// Returns whether the data type is an extension.
    data_type_is_extension => is_extension;
    /// Returns whether the entire nested type consists of Null leaves.
    data_type_is_nested_null => is_nested_null;
    /// Returns whether the data type is unknown.
    data_type_is_unknown => is_unknown;
    /// Returns whether the data type is numeric, including Decimal.
    data_type_is_numeric => is_numeric;
}

/// Returns the complete fixed-size Array shape.
pub fn data_type_shape(value: &str) -> Result<Vec<i64>, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let shape = inspect(value, polars::prelude::DataType::get_shape)?.ok_or_else(|| {
            TerlanPolarsError::new("invalid_data_type", "data type is not an Array")
        })?;
        shape
            .into_iter()
            .map(|dimension| {
                i64::try_from(dimension).map_err(|_| {
                    TerlanPolarsError::new("invalid_data_type", "array dimension exceeds Int range")
                })
            })
            .collect()
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Returns the immediate List or Array element type.
pub fn data_type_inner(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let data_type = crate::expressions::parse_data_type(value)?;
        let inner = data_type.inner_dtype().ok_or_else(|| {
            TerlanPolarsError::new("invalid_data_type", "data type is not a List or Array")
        })?;
        encode_data_type(inner)
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Returns the key type of a Map.
pub fn data_type_map_key(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let data_type = crate::expressions::parse_data_type(value)?;
        let (key, _) = data_type
            .as_map()
            .ok_or_else(|| TerlanPolarsError::new("invalid_data_type", "data type is not a Map"))?;
        encode_data_type(key)
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Returns the value type of a Map.
pub fn data_type_map_value(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let data_type = crate::expressions::parse_data_type(value)?;
        let (_, map_value) = data_type
            .as_map()
            .ok_or_else(|| TerlanPolarsError::new("invalid_data_type", "data type is not a Map"))?;
        encode_data_type(map_value)
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Returns the absolute List or Array leaf type.
pub fn data_type_leaf(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let data_type = crate::expressions::parse_data_type(value)?;
        encode_data_type(data_type.leaf_dtype())
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Returns the absolute Array leaf type.
pub fn data_type_array_leaf(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let data_type = crate::expressions::parse_data_type(value)?;
        let leaf = data_type.array_leaf_dtype().ok_or_else(|| {
            TerlanPolarsError::new("invalid_data_type", "data type is not an Array")
        })?;
        encode_data_type(leaf)
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Replaces every List or Array leaf with the requested type.
pub fn data_type_cast_leaf(value: &str, target: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let data_type = crate::expressions::parse_data_type(value)?;
        let target = crate::expressions::parse_data_type(target)?;
        encode_data_type(&data_type.cast_leaf(target))
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (value, target);
        unavailable()
    }
}

/// Returns whether Polars knows that a cast is valid.
pub fn data_type_can_cast_to(value: &str, target: &str) -> Result<bool, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let value = crate::expressions::parse_data_type(value)?;
        let target = crate::expressions::parse_data_type(target)?;
        value.can_cast_to(&target).ok_or_else(|| {
            TerlanPolarsError::new(
                "unknown_cast_compatibility",
                format!("Polars cannot determine whether `{value}` can cast to `{target}`"),
            )
        })
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (value, target);
        unavailable()
    }
}

/// Wraps the data type in one variable-width List level.
pub fn data_type_implode(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let data_type = crate::expressions::parse_data_type(value)?;
        encode_data_type(&data_type.implode())
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Converts the data type recursively to its physical representation.
pub fn data_type_to_physical(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let data_type = crate::expressions::parse_data_type(value)?;
        encode_data_type(&data_type.to_physical())
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Converts an extension recursively to its storage representation.
pub fn data_type_to_storage(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let data_type = crate::expressions::parse_data_type(value)?;
        encode_data_type(&data_type.to_storage())
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Returns whether this type is accepted by List arithmetic.
pub fn data_type_is_supported_list_arithmetic_input(
    value: &str,
) -> Result<bool, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        inspect(
            value,
            polars::prelude::DataType::is_supported_list_arithmetic_input,
        )
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Returns whether the type tree contains the requested type.
pub fn data_type_contains(value: &str, target: &str) -> Result<bool, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let value = crate::expressions::parse_data_type(value)?;
        let target = crate::expressions::parse_data_type(target)?;
        Ok(value.contains_dtype_recursive(&target))
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (value, target);
        unavailable()
    }
}

/// Applies Polars schema matching and reports whether a cast is required.
pub fn data_type_matches_schema(value: &str, schema_type: &str) -> Result<bool, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let value = crate::expressions::parse_data_type(value)?;
        let schema_type = crate::expressions::parse_data_type(schema_type)?;
        value
            .matches_schema_type(&schema_type)
            .map_err(|error| TerlanPolarsError::new("schema_mismatch", error.to_string()))
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (value, schema_type);
        unavailable()
    }
}

/// Returns the number of nested List or Array levels.
pub fn data_type_nesting_level(value: &str) -> Result<i64, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let level = inspect(value, polars::prelude::DataType::nesting_level)?;
        i64::try_from(level).map_err(|_| {
            TerlanPolarsError::new("invalid_data_type", "nesting level exceeds Int range")
        })
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Returns the unsigned integer type with the same numeric bit representation.
pub fn data_type_unsigned_bit_repr(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let value = crate::expressions::parse_data_type(value)?;
        let representation = value.numeric_to_unsigned_bit_repr().ok_or_else(|| {
            TerlanPolarsError::new("invalid_data_type", "data type is not numeric")
        })?;
        encode_data_type(&representation)
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Returns the maximum scalar bound in a lossless text representation.
pub fn data_type_max(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let value = crate::expressions::parse_data_type(value)?;
        value
            .max()
            .map(|scalar| scalar.value().to_string())
            .map_err(|error| TerlanPolarsError::new("invalid_data_type", error.to_string()))
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}

/// Returns the minimum scalar bound in a lossless text representation.
pub fn data_type_min(value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        let value = crate::expressions::parse_data_type(value)?;
        value
            .min()
            .map(|scalar| scalar.value().to_string())
            .map_err(|error| TerlanPolarsError::new("invalid_data_type", error.to_string()))
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = value;
        unavailable()
    }
}
