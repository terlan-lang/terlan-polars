//! Behavioral tests for the owned Series compatibility surface.

use crate::series::*;
use crate::series_extended::*;
use crate::{
    series_data_type, series_equal, series_equal_missing, series_from_bools, series_from_floats,
    series_from_ints, series_from_nullable_ints, series_from_strings, series_len, series_name,
    series_null_count, series_values, TerlanPolarsSeries,
};
use polars::prelude::{IntoSeries, NamedFrom};

fn ints(name: &str, values: &[i64]) -> TerlanPolarsSeries {
    series_from_ints(name, values).expect("integer Series")
}

fn values(series: &TerlanPolarsSeries) -> Vec<String> {
    series_values(series, usize::MAX.min(10_000)).expect("Series values")
}

#[test]
fn ownership_shape_and_chunk_operations_preserve_series_contracts() {
    let input = ints("value", &[3, 1, 1, 2]);
    let renamed = series_rename(&input, "renamed").expect("renamed Series");
    assert_eq!(series_name(&input), "value");
    assert_eq!(series_name(&renamed), "renamed");
    assert!(series_estimated_size(&input) > 0);
    assert_eq!(
        series_estimated_size_with_unit(&input, "kb").unwrap(),
        series_estimated_size(&input) as f64 / 1024.0
    );
    assert!(series_estimated_size_with_unit(&input, "milkshake").is_err());
    assert_eq!(series_chunk_count(&input), 1);
    assert_eq!(series_chunk_lengths(&input), vec![4]);

    let head = series_head(&input, 2).expect("head");
    let tail = series_tail(&input, 2).expect("tail");
    let slice = series_slice(&input, 1, 2).expect("slice");
    assert_eq!(values(&head), ["3", "1"]);
    assert_eq!(values(&tail), ["1", "2"]);
    assert_eq!(values(&slice), ["1", "1"]);

    let before = series_split_before(&input, 2).expect("split before");
    let after = series_split_after(&input, 2).expect("split after");
    assert_eq!(values(&before), ["3", "1"]);
    assert_eq!(values(&after), ["1", "2"]);

    let clear = series_clear(&input).expect("clear");
    assert_eq!(series_len(&clear), 0);
    assert_eq!(clear.inner.dtype(), input.inner.dtype());
    assert_eq!(series_name(&clear), series_name(&input));

    let rechunked = series_rechunk(&input).expect("rechunk");
    let shrunk = series_shrink_to_fit(&input).expect("shrink");
    assert!(series_equal(&input, &rechunked));
    assert!(series_equal(&input, &shrunk));
}

#[test]
fn ordering_distinct_and_concatenation_operations_execute() {
    let input = ints("value", &[3, 1, 1, 2]);
    let sorted = series_sort(&input, false, false, true).expect("stable sort");
    assert_eq!(values(&sorted), ["1", "1", "2", "3"]);
    assert_eq!(series_sorted_flag(&sorted), "ascending");

    let reversed = series_reverse(&input).expect("reverse");
    assert_eq!(values(&reversed), ["2", "1", "1", "3"]);
    let unique = series_unique_stable(&input).expect("stable unique");
    assert_eq!(values(&unique), ["3", "1", "2"]);

    let other = ints("value", &[4, 5]);
    let appended = series_append(&input, &other).expect("append");
    assert_eq!(values(&appended), ["3", "1", "1", "2", "4", "5"]);
    assert!(series_chunk_count(&appended) >= 2);
    let extended = series_extend(&input, &other).expect("extend");
    assert_eq!(values(&extended), ["3", "1", "1", "2", "4", "5"]);

    let first_chunk = series_select_chunk(&appended, 0).expect("first chunk");
    let second_chunk = series_select_chunk(&appended, 1).expect("second chunk");
    assert_eq!(values(&first_chunk), ["3", "1", "1", "2"]);
    assert_eq!(values(&second_chunk), ["4", "5"]);
    assert_eq!(
        series_select_chunk(&appended, 2)
            .expect_err("chunk bounds")
            .code(),
        "row_index_out_of_bounds"
    );
}

#[test]
fn null_sampling_and_reduction_operations_execute() {
    let nullable =
        series_from_nullable_ints("value", &[Some(1), None, None, Some(4), None, Some(6)])
            .expect("nullable Series");
    let forward = series_fill_null(&nullable, "forward", Some(1)).expect("forward fill");
    assert_eq!(values(&forward), ["1", "1", "null", "4", "4", "6"]);
    assert_eq!(series_null_count(&forward), 1);
    let zero = series_fill_null(&nullable, "zero", None).expect("zero fill");
    assert_eq!(series_null_count(&zero), 0);

    let gathered = series_gather_every(&nullable, 2, 1).expect("gather every");
    assert_eq!(values(&gathered), ["null", "4", "6"]);

    let input = ints("value", &[1, 2, 3, 4]);
    let shuffled = series_shuffle_seeded(&input, 9).expect("seeded shuffle");
    let shuffled_again = series_shuffle_seeded(&input, 9).expect("repeated seeded shuffle");
    assert!(series_equal(&shuffled, &shuffled_again));
    assert_eq!(series_len(&series_shuffle(&input).expect("shuffle")), 4);

    let sample = series_sample_n_seeded(&input, 3, false, true, 11).expect("seeded sample");
    let sample_again =
        series_sample_n_seeded(&input, 3, false, true, 11).expect("repeated seeded sample");
    assert!(series_equal(&sample, &sample_again));
    assert_eq!(
        series_len(&series_sample_n(&input, 2, false, false).expect("sample")),
        2
    );
    assert_eq!(
        series_len(
            &series_sample_fraction_seeded(&input, 0.5, false, true, 12).expect("fraction sample")
        ),
        2
    );
    assert_eq!(
        series_len(&series_sample_fraction(&input, 0.5, false, false).expect("fraction sample")),
        2
    );

    assert_eq!(values(&series_sum(&input).expect("sum")), ["10"]);
    assert_eq!(values(&series_min(&input).expect("minimum")), ["1"]);
    assert_eq!(values(&series_max(&input).expect("maximum")), ["4"]);
    assert_eq!(values(&series_product(&input).expect("product")), ["24"]);
    assert_eq!(values(&series_mean(&input).expect("mean")), ["2.5"]);
}

#[test]
fn explode_and_invalid_arguments_are_explicit() {
    let first = polars::prelude::Series::new("".into(), &[1_i64, 2]);
    let second = polars::prelude::Series::new("".into(), &[3_i64]);
    let list = TerlanPolarsSeries {
        inner: polars::prelude::ListChunked::from_iter(
            [Some(first), None, Some(second)].into_iter(),
        )
        .with_name("items".into())
        .into_series(),
    };
    let exploded = series_explode(&list, true, true).expect("explode");
    assert_eq!(values(&exploded), ["1", "2", "null", "3"]);

    assert_eq!(
        series_gather_every(&ints("value", &[1]), 0, 0)
            .expect_err("zero step")
            .code(),
        "invalid_series_step"
    );
    assert_eq!(
        series_sample_fraction(&ints("value", &[1]), f64::NAN, false, false)
            .expect_err("NaN fraction")
            .code(),
        "invalid_sample_fraction"
    );
    assert_eq!(
        series_fill_null(&ints("value", &[1]), "unknown", None)
            .expect_err("unknown strategy")
            .code(),
        "invalid_fill_strategy"
    );
}

#[test]
fn arithmetic_bitwise_and_comparison_operations_execute() {
    let left = ints("value", &[3, 1, 1, 2]);
    let right = ints("other", &[1, 3, 3, 4]);
    assert_eq!(
        values(&series_add(&left, &right).expect("add")),
        ["4", "4", "4", "6"]
    );
    assert_eq!(
        values(&series_subtract(&left, &right).expect("subtract")),
        ["2", "-2", "-2", "-2"]
    );
    assert_eq!(
        values(&series_multiply(&left, &right).expect("multiply")),
        ["3", "3", "3", "8"]
    );
    assert_eq!(
        values(&series_divide(&left, &right).expect("divide")),
        ["3", "0", "0", "0"]
    );
    assert_eq!(
        values(&series_remainder(&left, &right).expect("remainder")),
        ["0", "1", "1", "2"]
    );

    let bits_left = ints("bits", &[1, 2]);
    let bits_right = ints("bits", &[3, 1]);
    assert_eq!(
        values(&series_bit_and(&bits_left, &bits_right).expect("bit and")),
        ["1", "0"]
    );
    assert_eq!(
        values(&series_bit_or(&bits_left, &bits_right).expect("bit or")),
        ["3", "3"]
    );
    assert_eq!(
        values(&series_bit_xor(&bits_left, &bits_right).expect("bit xor")),
        ["2", "3"]
    );

    assert_eq!(
        values(&series_equal_values(&left, &right).expect("equal values")),
        ["false", "false", "false", "false"]
    );
    assert_eq!(
        values(&series_not_equal_values(&left, &right).expect("not equal values")),
        ["true", "true", "true", "true"]
    );
    assert_eq!(
        values(&series_less_than(&left, &right).expect("less than")),
        ["false", "true", "true", "true"]
    );
    assert_eq!(
        values(&series_less_than_or_equal(&left, &right).expect("less or equal")),
        ["false", "true", "true", "true"]
    );
    assert_eq!(
        values(&series_greater_than(&left, &right).expect("greater than")),
        ["true", "false", "false", "false"]
    );
    assert_eq!(
        values(&series_greater_than_or_equal(&left, &right).expect("greater or equal")),
        ["true", "false", "false", "false"]
    );
}

#[test]
fn floating_validity_masks_execute() {
    let input = series_from_floats("value", &[1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY])
        .expect("floating Series");
    assert_eq!(
        values(&series_is_nan(&input).expect("NaN mask")),
        ["false", "true", "false", "false"]
    );
    assert_eq!(
        values(&series_is_not_nan(&input).expect("not NaN mask")),
        ["true", "false", "true", "true"]
    );
    assert_eq!(
        values(&series_is_finite(&input).expect("finite mask")),
        ["true", "false", "false", "false"]
    );
    assert_eq!(
        values(&series_is_infinite(&input).expect("infinite mask")),
        ["false", "false", "true", "true"]
    );
}

#[test]
fn null_cast_and_storage_operations_preserve_owned_values() {
    let nulls = series_full_null("value", 3, "Int64").expect("typed null Series");
    assert_eq!(series_len(&nulls), 3);
    assert_eq!(series_null_count(&nulls), 3);
    assert_eq!(format!("{:?}", nulls.inner.dtype()), "Int64");

    let untyped = series_new_null("missing", 2).expect("Null Series");
    assert_eq!(format!("{:?}", untyped.inner.dtype()), "Null");
    assert_eq!(series_null_count(&untyped), 2);

    let strings = series_from_strings(
        "value",
        &["1".to_string(), "invalid".to_string(), "300".to_string()],
    )
    .expect("String Series");
    assert!(series_strict_cast(&strings, "UInt8").is_err());
    let non_strict = series_non_strict_cast(&strings, "UInt8").expect("non-strict cast");
    assert_eq!(values(&non_strict), ["1", "null", "null"]);
    let overflowing =
        series_overflowing_cast(&ints("value", &[1, 300]), "UInt8").expect("overflowing cast");
    assert_eq!(values(&overflowing), ["1", "44"]);
    let cast_with_options =
        series_cast_with_options(&ints("value", &[1, 300]), "UInt8", "overflowing")
            .expect("cast with options");
    assert_eq!(values(&cast_with_options), ["1", "44"]);
    assert_eq!(
        series_cast_with_options(&strings, "UInt8", "lossy")
            .expect_err("invalid cast mode")
            .code(),
        "invalid_cast_mode"
    );

    let physical = series_to_physical(&non_strict).expect("physical representation");
    let storage = series_to_storage(&physical).expect("storage representation");
    assert!(series_equal_missing(&physical, &storage));
}

#[test]
fn metadata_reshape_and_constant_operations_execute() {
    let input = ints("value", &[1, 2, 3, 4]);
    let sorted =
        series_set_sorted_flag(&input, "descending").expect("set descending sorted metadata");
    assert_eq!(series_sorted_flag(&input), "not_sorted");
    assert_eq!(series_sorted_flag(&sorted), "descending");
    assert!(!series_should_rechunk(&input));
    assert_eq!(
        series_set_sorted_flag(&input, "sideways")
            .expect_err("invalid sorted flag")
            .code(),
        "invalid_sorted_flag"
    );

    let imploded = series_implode(&input).expect("implode");
    let as_list = series_as_list(&input).expect("as list");
    let unit_list = series_to_unit_list(&input).expect("unit list");
    assert_eq!(series_len(&imploded), 1);
    assert_eq!(series_len(&as_list), 4);
    assert!(values(&imploded)[0].starts_with("[1, 2,"));
    assert_eq!(values(&as_list), ["[1]", "[2]", "[3]", "[4]"]);
    assert!(series_equal(&as_list, &unit_list));

    let category_ids = TerlanPolarsSeries {
        inner: polars::prelude::Series::new("size".into(), [0_u8, 2, 1]),
    };
    let enum_type = crate::expressions::data_type_enum(&[
        "small".to_string(),
        "medium".to_string(),
        "large".to_string(),
    ])
    .expect("Enum data type");
    let categories = series_from_cats_and_dtype(&category_ids, &enum_type, true)
        .expect("categories from physical IDs");
    assert_eq!(
        values(&categories),
        ["\"small\"", "\"large\"", "\"medium\""]
    );
    assert!(series_data_type(&categories).contains("Enum"));

    let list = series_reshape_list(&input, &[2, 2]).expect("reshape list");
    assert_eq!(series_len(&list), 2);
    assert!(format!("{:?}", list.inner.dtype()).starts_with("List("));
    let array = series_reshape_array(&input, &[2, 2]).expect("reshape array");
    assert_eq!(series_len(&array), 2);
    assert!(format!("{:?}", array.inner.dtype()).starts_with("Array("));
    assert_eq!(
        series_reshape_list(&input, &[])
            .expect_err("empty dimensions")
            .code(),
        "invalid_reshape_dimensions"
    );
    assert_eq!(
        series_reshape_array(&input, &[-1, -1])
            .expect_err("multiple inferred dimensions")
            .code(),
        "invalid_reshape_dimensions"
    );

    let extended = series_extend_int(&input, 9, 2).expect("extend integer constant");
    assert_eq!(values(&extended), ["1", "2", "3", "4", "9", "9"]);
    let with_null = series_extend_null(&input, 1).expect("extend null");
    assert_eq!(values(&with_null), ["1", "2", "3", "4", "null"]);

    let leaf = series_leaf_values(&array).expect("Array leaf values");
    assert_eq!(values(&leaf), ["1", "2", "3", "4"]);
    let floats = series_to_float(&input).expect("numeric to float");
    assert_eq!(values(&floats), ["1.0", "2.0", "3.0", "4.0"]);
}

#[test]
fn conditional_selection_and_serialization_round_trip() {
    let input = ints("value", &[1, 2, 3]);
    let other = ints("other", &[10, 20, 30]);
    let mask = crate::series_from_bools("mask", &[true, false, true]).expect("Boolean mask");
    let selected = series_zip_with(&input, &mask, &other).expect("conditional selection");
    assert_eq!(values(&selected), ["1", "20", "3"]);

    let nullable_left =
        series_from_nullable_ints("value", &[Some(1), None]).expect("nullable left");
    let nullable_right =
        series_from_nullable_ints("value", &[Some(2), None]).expect("nullable right");
    let missing_inequality =
        series_not_equal_missing(&nullable_left, &nullable_right).expect("missing inequality");
    assert_eq!(values(&missing_inequality), ["true", "false"]);

    let bytes = series_serialize(&selected).expect("serialize Series");
    assert!(!bytes.is_empty());
    let restored = series_deserialize(&bytes).expect("deserialize Series");
    assert!(series_equal(&selected, &restored));
    assert_eq!(series_name(&restored), series_name(&selected));
    assert!(series_deserialize(b"not a Polars Series").is_err());
}

#[test]
fn typed_scalar_extraction_is_checked_and_lossless() {
    let integers = ints("integer", &[-2, 0, 7]);
    assert_eq!(
        series_integer_values(&integers).expect("integer values"),
        [-2, 0, 7]
    );

    let unsigned = TerlanPolarsSeries {
        inner: polars::prelude::Series::new("unsigned".into(), [1_u64, 8_u64]),
    };
    assert_eq!(
        series_integer_values(&unsigned).expect("unsigned values"),
        [1, 8]
    );

    let floats = series_from_floats("float", &[1.25, -2.5]).expect("construct floating Series");
    assert_eq!(
        series_float_values(&floats).expect("floating values"),
        [1.25, -2.5]
    );

    let booleans = series_from_bools("boolean", &[true, false]).expect("construct Boolean Series");
    assert_eq!(
        series_boolean_values(&booleans).expect("Boolean values"),
        [true, false]
    );

    let strings = series_from_strings("string", &["first".to_string(), "second".to_string()])
        .expect("construct String Series");
    assert_eq!(
        series_string_values(&strings).expect("String values"),
        ["first", "second"]
    );

    let nullable =
        series_from_nullable_ints("nullable", &[Some(1), None]).expect("nullable Series");
    assert_eq!(
        series_integer_values(&nullable)
            .expect_err("nullable extraction")
            .code(),
        "nullable_values_require_option"
    );

    let overflowing = TerlanPolarsSeries {
        inner: polars::prelude::Series::new("overflowing".into(), [u64::MAX]),
    };
    assert!(series_integer_values(&overflowing).is_err());
    assert!(series_boolean_values(&integers).is_err());
    assert!(series_string_values(&integers).is_err());
}

#[test]
fn wide_integer_decimal_boundaries_are_lossless_and_validated() {
    let signed_values = vec![
        i128::MIN.to_string(),
        "0".to_string(),
        i128::MAX.to_string(),
    ];
    let signed =
        series_from_int128_strings("signed", &signed_values).expect("construct Int128 Series");
    assert_eq!(
        series_int128_strings(&signed).expect("extract Int128 values"),
        signed_values
    );

    let unsigned_values = vec!["0".to_string(), u128::MAX.to_string()];
    let unsigned = series_from_uint128_strings("unsigned", &unsigned_values)
        .expect("construct UInt128 Series");
    assert_eq!(
        series_uint128_strings(&unsigned).expect("extract UInt128 values"),
        unsigned_values
    );

    let error = series_from_int128_strings(
        "invalid",
        &["170141183460469231731687303715884105728".to_string()],
    )
    .expect_err("reject value above Int128 maximum");
    assert_eq!(error.code(), "invalid_wide_integer");
    assert!(error.message().contains("index 0"));
    assert!(series_uint128_strings(&signed).is_err());
    assert!(series_int128_strings(&unsigned).is_err());
}

#[test]
fn clone_and_row_encoding_preserve_series_ownership_and_shape() {
    let source = ints("value", &[3, 1, 2]);
    let cloned = series_clone(&source).expect("clone Series");
    let renamed = series_rename(&cloned, "copy").expect("rename cloned Series");

    assert_eq!(series_name(&source), "value");
    assert_eq!(series_name(&renamed), "copy");
    assert!(series_equal(&source, &cloned));

    let unordered =
        series_row_encode(&source, false, false, false).expect("unordered row encoding");
    let ascending = series_row_encode(&source, true, false, false).expect("ascending row encoding");
    let descending =
        series_row_encode(&source, true, true, false).expect("descending row encoding");

    assert_eq!(series_len(&unordered), series_len(&source));
    assert_eq!(series_len(&ascending), series_len(&source));
    assert_eq!(series_len(&descending), series_len(&source));
    assert_eq!(series_data_type(&unordered), "BinaryOffset");
    assert_eq!(series_data_type(&ascending), "BinaryOffset");
    assert!(!series_equal(&ascending, &descending));
}
