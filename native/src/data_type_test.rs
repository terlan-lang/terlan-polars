//! Tests for parameterized Polars data-type descriptors.

use crate::data_types::*;
use crate::expressions::{
    data_type_array, data_type_categorical, data_type_datetime, data_type_decimal, data_type_enum,
    data_type_extension, data_type_extension_metadata, data_type_field, data_type_list,
    data_type_struct, parse_data_type,
};

#[test]
fn extension_descriptor_preserves_name_metadata_and_storage() {
    let descriptor =
        data_type_extension_metadata("terlan.uuid", "version=1", "Binary").expect("descriptor");
    let data_type = parse_data_type(&descriptor).expect("parse extension data type");
    let polars::prelude::DataType::Extension(extension, storage) = data_type else {
        panic!("expected extension data type");
    };

    assert_eq!(extension.name(), "terlan.uuid");
    assert_eq!(extension.serialize_metadata().as_deref(), Some("version=1"));
    assert_eq!(*storage, polars::prelude::DataType::Binary);
}

#[test]
fn extension_descriptor_rejects_invalid_boundaries() {
    for name in ["", &"x".repeat(257), "bad\0name"] {
        assert_eq!(
            data_type_extension(name, "Binary")
                .expect_err("invalid name must fail")
                .code,
            "invalid_data_type"
        );
    }
    assert_eq!(
        data_type_extension("terlan.uuid", "")
            .expect_err("empty storage type must fail")
            .code,
        "invalid_data_type"
    );
    assert_eq!(
        data_type_extension_metadata("terlan.uuid", &"x".repeat(65_537), "Binary")
            .expect_err("oversized metadata must fail")
            .code,
        "invalid_data_type"
    );
}

#[test]
fn scalar_classification_matches_polars_semantics() {
    assert!(data_type_is_known("Int64").expect("known"));
    assert!(data_type_is_primitive("Int64").expect("primitive"));
    assert!(data_type_is_primitive_numeric("Int64").expect("primitive numeric"));
    assert!(data_type_is_numeric("Int64").expect("numeric"));
    assert!(data_type_is_integer("Int64").expect("integer"));
    assert!(data_type_is_signed_integer("Int64").expect("signed"));
    assert!(!data_type_is_unsigned_integer("Int64").expect("not unsigned"));
    assert!(data_type_is_float("Float64").expect("float"));
    assert!(data_type_is_bool("Boolean").expect("bool"));
    assert!(data_type_is_string("Utf8").expect("string"));
    assert!(data_type_is_binary("BinaryType").expect("binary"));
    assert!(data_type_is_null("NullType").expect("null"));
    assert!(data_type_is_date("DateType").expect("date"));
    assert!(data_type_is_datetime("DatetimeNanoseconds").expect("datetime"));
    assert!(data_type_is_duration("DurationMicroseconds").expect("duration"));
    assert!(data_type_is_temporal("TimeType").expect("temporal"));
    assert!(data_type_is_logical("DateType").expect("logical"));
    assert!(data_type_is_ord("Utf8").expect("ordered"));
    assert!(!data_type_is_object("Utf8").expect("object feature disabled"));
    assert!(!data_type_is_unknown("Utf8").expect("known string"));
    assert!(!data_type_contains_unknown("Utf8").expect("known tree"));
    assert!(data_type_contains_views("Utf8").expect("string view"));
    assert!(data_type_is_supported_list_arithmetic_input("Boolean").expect("list arithmetic"));
    assert_eq!(
        data_type_unsigned_bit_repr("Float32").expect("unsigned representation"),
        "UInt32"
    );
}

#[test]
fn nested_classification_and_transforms_round_trip() {
    let list = data_type_list("Int64").expect("list");
    let array = data_type_array(&list, 3).expect("array");
    let decimal = data_type_decimal(12, 2).expect("decimal");
    let categorical = data_type_categorical().expect("categorical");
    let enumeration = data_type_enum(&["red".to_string(), "green".to_string()]).expect("enum");
    let field = data_type_field("values", &array).expect("field");
    let structure = data_type_struct(&[field]).expect("struct");
    let extension = data_type_extension("terlan.ids", &list).expect("extension");

    assert!(data_type_is_list(&list).expect("list"));
    assert!(data_type_is_array(&array).expect("array"));
    assert!(data_type_is_nested(&structure).expect("nested"));
    assert!(data_type_is_struct(&structure).expect("struct"));
    assert!(data_type_is_decimal(&decimal).expect("decimal"));
    assert!(data_type_is_categorical(&categorical).expect("categorical"));
    assert!(data_type_is_enum(&enumeration).expect("enum"));
    assert!(data_type_is_extension(&extension).expect("extension"));
    assert!(data_type_contains_categoricals(&categorical).expect("contains categorical"));
    assert!(data_type_contains_enums(&enumeration).expect("contains enum"));
    assert!(data_type_contains_list_recursive(&structure).expect("contains list"));
    assert!(!data_type_contains_objects(&structure).expect("contains no object"));
    assert!(data_type_contains(&structure, "Int64").expect("contains int"));
    assert_eq!(data_type_shape(&array).expect("shape"), [3]);
    assert_eq!(data_type_inner(&array).expect("inner"), list);
    assert_eq!(data_type_leaf(&array).expect("leaf"), "Int64");
    assert_eq!(data_type_array_leaf(&array).expect("array leaf"), list);
    assert_eq!(data_type_nesting_level(&array).expect("nesting"), 2);

    let cast = data_type_cast_leaf(&array, "Float32").expect("cast leaf");
    assert!(data_type_contains(&cast, "Float32").expect("contains float"));
    assert_eq!(
        data_type_to_physical("DateType").expect("physical date"),
        "Int32"
    );
    assert_eq!(
        data_type_to_storage(&extension).expect("extension storage"),
        list
    );
    assert!(data_type_implode("Int64")
        .expect("implode")
        .starts_with('{'));
    assert!(data_type_pretty_format(&structure)
        .expect("pretty format")
        .contains("values"));
}

#[test]
fn compatibility_bounds_and_null_matching_preserve_results() {
    assert!(data_type_can_cast_to("Int32", "Float64").expect("numeric cast"));
    assert_eq!(
        data_type_matches_schema("NullType", "Utf8").expect("null schema match"),
        true
    );
    assert_eq!(
        data_type_matches_schema("Utf8", "Utf8").expect("same schema"),
        false
    );
    assert_eq!(data_type_max("Int8").expect("max"), i8::MAX.to_string());
    assert_eq!(data_type_min("Int8").expect("min"), i8::MIN.to_string());

    let nested_null = data_type_list("NullType").expect("null list");
    assert!(data_type_is_nested_null(&nested_null).expect("nested null"));
}

#[test]
fn data_type_queries_reject_incompatible_inputs() {
    for result in [
        data_type_shape("Int64").map(|_| ()),
        data_type_inner("Int64").map(|_| ()),
        data_type_array_leaf("Int64").map(|_| ()),
        data_type_unsigned_bit_repr("Utf8").map(|_| ()),
        data_type_max("Utf8").map(|_| ()),
        data_type_min("Utf8").map(|_| ()),
        data_type_matches_schema("Utf8", "Int64").map(|_| ()),
    ] {
        assert!(result.is_err());
    }

    let zoned = data_type_datetime("DatetimeMicroseconds", "Europe/Riga").expect("zoned datetime");
    assert!(data_type_is_datetime(&zoned).expect("zoned datetime classification"));
}
