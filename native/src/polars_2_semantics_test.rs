//! Behavioral checks for Python Polars 2.0 semantics that cross the Rust adapter.

use crate::expressions::*;
use crate::*;
use polars::df;
use polars::prelude::{
    DataFrame, DataType, IntoColumn, IntoLazy, IntoSeries, ListChunked, NamedFrom, Series, TimeUnit,
};

#[test]
fn mixed_signed_and_uint64_arithmetic_uses_exact_int128() {
    let frame = df!("signed" => [-1_i64, 2], "unsigned" => [2_u64, u64::MAX])
        .expect("mixed integer fixture");
    let signed = expr_col("signed").expect("signed expression");
    let unsigned = expr_col("unsigned").expect("unsigned expression");
    let result = expr_alias(
        &expr_add(&signed, &unsigned).expect("mixed integer addition"),
        "result",
    )
    .expect("result alias");
    let output = frame
        .lazy()
        .select(compile_many(&[result]).expect("compile mixed integer expression"))
        .collect()
        .expect("collect mixed integer expression");
    assert_eq!(output.column("result").unwrap().dtype(), &DataType::Int128);
    assert_eq!(
        output.column("result").unwrap().get(1).unwrap().to_string(),
        (i128::from(u64::MAX) + 2).to_string()
    );
}

#[test]
fn membership_coercion_is_strict_and_list_to_struct_preserves_outer_nulls() {
    let float_members = ListChunked::from_iter(
        [
            Some(Series::new("".into(), [1.99_f64])),
            Some(Series::new("".into(), [2.0_f64])),
        ]
        .into_iter(),
    )
    .with_name("members".into())
    .into_series();
    let membership = DataFrame::new(
        2,
        vec![
            Series::new("needle".into(), [1_i64, 2]).into(),
            float_members.into(),
        ],
    )
    .expect("membership fixture");
    let needle = expr_col("needle").expect("needle expression");
    let members = expr_col("members").expect("members expression");
    let check = expr_is_in(&needle, &members, true).expect("membership expression");
    assert!(membership
        .lazy()
        .select(compile_many(&[check]).expect("compile membership expression"))
        .collect()
        .is_err());

    let decimal = Series::new("needle".into(), [100_i128])
        .into_decimal(3, 2)
        .expect("Decimal needle");
    let floats =
        ListChunked::from_iter([Some(Series::new("".into(), [1.005_f64, 2.5]))].into_iter())
            .with_name("members".into())
            .into_series();
    let membership = DataFrame::new(1, vec![decimal.into_column(), floats.into_column()])
        .expect("Decimal membership fixture");
    let check = expr_is_in(
        &expr_col("needle").expect("Decimal needle expression"),
        &expr_col("members").expect("float members expression"),
        true,
    )
    .expect("Decimal membership expression");
    assert!(membership
        .lazy()
        .select(compile_many(&[check]).expect("compile Decimal membership"))
        .collect()
        .is_err());

    let exact_ns = 1_577_836_800_000_000_000_i64;
    let needle = Series::new("needle".into(), [exact_ns])
        .cast(&DataType::Datetime(TimeUnit::Nanoseconds, None))
        .expect("nanosecond needle");
    let members = Series::new("".into(), [exact_ns / 1_000])
        .cast(&DataType::Datetime(TimeUnit::Microseconds, None))
        .expect("microsecond members");
    let members = ListChunked::from_iter([Some(members)].into_iter())
        .with_name("members".into())
        .into_series();
    let membership = DataFrame::new(1, vec![needle.into_column(), members.into_column()])
        .expect("temporal membership fixture");
    let check = expr_is_in(
        &expr_col("needle").expect("temporal needle expression"),
        &expr_col("members").expect("temporal members expression"),
        true,
    )
    .expect("temporal membership expression");
    let output = membership
        .lazy()
        .select(compile_many(&[check]).expect("compile temporal membership"))
        .collect()
        .expect("collect temporal membership");
    assert_eq!(
        output.column("needle").unwrap().bool().unwrap().get(0),
        Some(true)
    );

    let needle = Series::new("needle".into(), [exact_ns / 1_000])
        .cast(&DataType::Datetime(TimeUnit::Microseconds, None))
        .expect("naive needle");
    let utc = polars::prelude::TimeZone::opt_try_new(Some("UTC".to_string()))
        .expect("valid UTC time zone");
    let members = Series::new("".into(), [exact_ns / 1_000])
        .cast(&DataType::Datetime(TimeUnit::Microseconds, utc))
        .expect("aware members");
    let members = ListChunked::from_iter([Some(members)].into_iter())
        .with_name("members".into())
        .into_series();
    let membership = DataFrame::new(1, vec![needle.into_column(), members.into_column()])
        .expect("timezone membership fixture");
    let check = expr_is_in(
        &expr_col("needle").expect("naive needle expression"),
        &expr_col("members").expect("aware members expression"),
        true,
    )
    .expect("timezone membership expression");
    assert!(membership
        .lazy()
        .select(compile_many(&[check]).expect("compile timezone membership"))
        .collect()
        .is_err());

    let enum_type = parse_data_type(
        &data_type_enum(&["a".into(), "b".into()]).expect("membership Enum descriptor"),
    )
    .expect("membership Enum type");
    let needle = Series::new("needle".into(), ["a"])
        .cast(&enum_type)
        .expect("Enum needle");
    let members = ListChunked::from_iter([Some(Series::new("".into(), ["a", "z"]))].into_iter())
        .with_name("members".into())
        .into_series();
    let membership = DataFrame::new(1, vec![needle.into_column(), members.into_column()])
        .expect("Enum membership fixture");
    let check = expr_is_in(
        &expr_col("needle").expect("Enum needle expression"),
        &expr_col("members").expect("String members expression"),
        true,
    )
    .expect("Enum membership expression");
    let output = membership
        .lazy()
        .select(compile_many(&[check]).expect("compile Enum membership"))
        .collect()
        .expect("collect Enum membership");
    assert_eq!(
        output.column("needle").unwrap().bool().unwrap().get(0),
        Some(true)
    );

    let values =
        ListChunked::from_iter([None, Some(Series::new("".into(), [1_i64, 2]))].into_iter())
            .with_name("values".into())
            .into_series();
    let frame = DataFrame::new(2, vec![values.into()]).expect("nullable list fixture");
    let values = expr_col("values").expect("list expression");
    let structure = expr_list_to_struct(&values, &["left".into(), "right".into()])
        .expect("list-to-Struct expression");
    let output = frame
        .lazy()
        .select(compile_many(&[structure]).expect("compile list-to-Struct"))
        .collect()
        .expect("collect list-to-Struct");
    assert_eq!(
        output.column("values").unwrap().is_null().get(0),
        Some(true)
    );
    assert_eq!(
        output.column("values").unwrap().is_null().get(1),
        Some(false)
    );
}

#[test]
fn datetime_and_repeat_use_the_leftmost_argument_name() {
    let frame =
        df!("year" => [2001_i64], "month" => [1_i64], "day" => [1_i64]).expect("datetime fixture");
    let integer = |value| expr_lit(&TerlanPolarsScalar::Int(value)).expect("integer literal");
    let components = vec![
        expr_col("year").expect("year"),
        expr_col("month").expect("month"),
        expr_col("day").expect("day"),
        integer(23),
        integer(0),
        integer(0),
        integer(0),
        expr_lit(&TerlanPolarsScalar::String("raise".into())).expect("ambiguous policy"),
    ];
    let datetime = expr_datetime(&components, "us", "").expect("datetime expression");
    let output = frame
        .lazy()
        .select(compile_many(&[datetime]).expect("compile datetime"))
        .collect()
        .expect("collect datetime");
    assert_eq!(output.get_column_names(), ["year"]);

    let value = expr_lit(&TerlanPolarsScalar::Int(3)).expect("repeat value");
    let count = expr_lit(&TerlanPolarsScalar::Int(3)).expect("repeat count");
    let repeated = expr_repeat(&value, &count).expect("repeat expression");
    let output = DataFrame::empty()
        .lazy()
        .select(compile_many(&[repeated]).expect("compile repeat"))
        .collect()
        .expect("collect repeat");
    assert_eq!(output.get_column_names(), ["literal"]);
    assert_eq!(output.height(), 3);
}

#[test]
fn removed_lossy_casts_and_non_numeric_math_raise() {
    let strings = series_from_strings("value", &["2026-10-06".to_string()]).expect("String Series");
    assert!(series_strict_cast(&strings, "Date").is_err());

    let integers = series_from_ints("value", &[0, 1]).expect("integer Series");
    let list_type = data_type_list("Int64").expect("List type");
    assert!(series_strict_cast(&integers, &list_type).is_err());
    let enum_type = data_type_enum(&["zero".into(), "one".into()]).expect("Enum type");
    assert!(series_strict_cast(&integers, &enum_type).is_err());

    let structures = df!("a" => [1_i64], "b" => [2_i64]).expect("Struct cast fixture");
    let structure = expr_as_struct(&[
        expr_col("a").expect("field a"),
        expr_col("b").expect("field b"),
    ])
    .expect("Struct expression");
    let target_field = data_type_field("a", "Int64").expect("target Struct field");
    let target = data_type_struct(&[target_field]).expect("target Struct type");
    let truncated = expr_strict_cast(&structure, &target).expect("strict Struct cast descriptor");
    assert!(structures
        .lazy()
        .select(compile_many(&[truncated]).expect("compile strict Struct cast"))
        .collect()
        .is_err());

    let value = expr_col("value").expect("String expression");
    let base = expr_lit(&TerlanPolarsScalar::Int(2)).expect("log base");
    let logarithm = expr_log(&value, &base).expect("log expression");
    assert!(series_to_frame(&strings)
        .expect("String frame")
        .inner
        .lazy()
        .select(compile_many(&[logarithm]).expect("compile log expression"))
        .collect()
        .is_err());
}

#[test]
fn removed_boolean_duration_flat_list_and_struct_shims_raise() {
    let frame = df!("flag" => [true, false], "number" => [1_i32, 2]).expect("bitwise fixture");
    let flag = expr_col("flag").expect("Boolean expression");
    let number = expr_col("number").expect("integer expression");
    let bitwise = expr_and(&flag, &number).expect("bitwise expression");
    assert!(frame
        .clone()
        .lazy()
        .select(compile_many(&[bitwise]).expect("compile bitwise expression"))
        .collect()
        .is_err());

    let duration = Series::new("duration".into(), [1_i64, 2])
        .cast(&DataType::Duration(TimeUnit::Microseconds))
        .expect("Duration Series");
    let duration_frame = DataFrame::new(2, vec![duration.into_column()]).expect("Duration frame");
    let duration = expr_col("duration").expect("Duration expression");
    let standard_deviation = expr_std(&duration, 1).expect("std expression");
    assert!(duration_frame
        .lazy()
        .select(compile_many(&[standard_deviation]).expect("compile std expression"))
        .collect()
        .is_err());

    let lists = ListChunked::from_iter(
        [
            Some(Series::new("".into(), [1_i64, 2])),
            Some(Series::new("".into(), [3_i64, 4])),
        ]
        .into_iter(),
    )
    .with_name("values".into())
    .into_series();
    let list_frame = DataFrame::new(
        2,
        vec![
            lists.into_column(),
            Series::new("index".into(), [0_i64, 1]).into_column(),
        ],
    )
    .expect("list gather fixture");
    let values = expr_col("values").expect("list expression");
    let index = expr_col("index").expect("flat index expression");
    let gather = expr_list_gather(&values, &index, false).expect("list gather expression");
    assert!(list_frame
        .lazy()
        .select(compile_many(&[gather]).expect("compile list gather"))
        .collect()
        .is_err());

    let left = expr_col("flag").expect("left struct field");
    let right = expr_col("number").expect("right struct field");
    let structure = expr_as_struct(&[left, right]).expect("Struct expression");
    let renamed =
        expr_struct_rename_fields(&structure, &["only".into()]).expect("Struct rename expression");
    assert!(frame
        .lazy()
        .select(compile_many(&[renamed]).expect("compile Struct rename"))
        .collect()
        .is_err());
}

#[test]
fn eager_series_arithmetic_uses_int128_and_rejects_boolean_integer_bitwise() {
    let signed = TerlanPolarsSeries {
        inner: Series::new("signed".into(), [-1_i64, 2]),
    };
    let unsigned = TerlanPolarsSeries {
        inner: Series::new("unsigned".into(), [2_u64, u64::MAX]),
    };
    let result = series_add(&signed, &unsigned).expect("mixed integer Series addition");
    assert_eq!(result.inner.dtype(), &DataType::Int128);

    let booleans = TerlanPolarsSeries {
        inner: Series::new("flag".into(), [true, false]),
    };
    let integers = TerlanPolarsSeries {
        inner: Series::new("number".into(), [1_i32, 2]),
    };
    assert!(series_bit_and(&booleans, &integers).is_err());
}

#[test]
fn map_dtype_and_expression_namespace_round_trip() {
    use polars::lazy::dsl::{as_struct, col};

    let descriptor = data_type_map("Utf8", "Int64").expect("Map descriptor");
    assert!(data_types::data_type_is_map(&descriptor).expect("Map predicate"));
    assert_eq!(
        data_types::data_type_map_key(&descriptor).expect("Map key type"),
        "Utf8"
    );
    assert_eq!(
        data_types::data_type_map_value(&descriptor).expect("Map value type"),
        "Int64"
    );

    let frame = df!("key" => ["a", "b"], "value" => [1_i64, 2]).expect("Map fixture");
    let maps = frame
        .lazy()
        .select([as_struct(vec![col("key"), col("value")])
            .implode(true)
            .list()
            .to_map()
            .alias("m")])
        .collect()
        .expect("construct Map");
    assert_eq!(
        maps.column("m").expect("Map column").dtype(),
        &DataType::Map(Box::new(DataType::String), Box::new(DataType::Int64))
    );

    let map = expr_col("m").expect("Map expression");
    let key = expr_lit(&TerlanPolarsScalar::String("b".into())).expect("key literal");
    let expressions = [
        expr_alias(&expr_map_entries(&map).expect("entries"), "entries").expect("entries alias"),
        expr_alias(&expr_map_keys(&map).expect("keys"), "keys").expect("keys alias"),
        expr_alias(&expr_map_values(&map).expect("values"), "values").expect("values alias"),
        expr_alias(&expr_map_len(&map).expect("len"), "len").expect("len alias"),
        expr_alias(
            &expr_map_contains_key(&map, &key).expect("contains key"),
            "contains",
        )
        .expect("contains alias"),
        expr_alias(&expr_map_get(&map, &key).expect("get"), "get").expect("get alias"),
    ];
    let output = maps
        .lazy()
        .select(compile_many(&expressions).expect("compile Map expressions"))
        .collect()
        .expect("collect Map expressions");
    assert_eq!(
        output.column("len").unwrap().get(0).unwrap().to_string(),
        "2"
    );
    assert_eq!(
        output
            .column("contains")
            .unwrap()
            .get(0)
            .unwrap()
            .to_string(),
        "true"
    );
    assert_eq!(
        output.column("get").unwrap().get(0).unwrap().to_string(),
        "2"
    );

    // A non-Struct key is invalid for Map and must be rejected at construction.
    assert!(data_type_map("Null", "Int64").is_err());
}

#[test]
fn struct_arithmetic_resolves_each_field_and_rejects_non_numeric_fields() {
    let frame = DataFrame::new(
        1,
        vec![
            Series::new("i".into(), [1_i64]).into_column(),
            Series::new("f".into(), [2_f32]).into_column(),
            Series::new("s".into(), ["a"]).into_column(),
        ],
    )
    .expect("Struct arithmetic fixture");
    let numeric = expr_as_struct(&[
        expr_col("i").expect("integer field"),
        expr_col("f").expect("float field"),
    ])
    .expect("numeric Struct");
    let scalar = expr_lit(&TerlanPolarsScalar::Float(1.5)).expect("float scalar");
    let result = expr_alias(
        &expr_add(&numeric, &scalar).expect("Struct arithmetic"),
        "result",
    )
    .expect("Struct alias");
    let output = frame
        .clone()
        .lazy()
        .select(compile_many(&[result]).expect("compile Struct arithmetic"))
        .collect()
        .expect("collect Struct arithmetic");
    let DataType::Struct(fields) = output.column("result").unwrap().dtype() else {
        panic!("Struct arithmetic did not return a Struct")
    };
    assert_eq!(fields[0].dtype(), &DataType::Float64);
    assert_eq!(fields[1].dtype(), &DataType::Float32);

    let mixed = expr_as_struct(&[
        expr_col("i").expect("integer field"),
        expr_col("s").expect("String field"),
    ])
    .expect("mixed Struct");
    let result = expr_add(
        &mixed,
        &expr_lit(&TerlanPolarsScalar::Int(1)).expect("integer scalar"),
    )
    .expect("mixed Struct arithmetic");
    assert!(frame
        .lazy()
        .select(compile_many(&[result]).expect("compile mixed Struct arithmetic"))
        .collect()
        .is_err());
}

#[test]
fn decimal_rounding_widens_precision() {
    let decimal = Series::new("value".into(), [95_i128, -5_i128])
        .into_decimal(2, 1)
        .expect("Decimal fixture");
    let frame = DataFrame::new(2, vec![decimal.into_column()]).expect("Decimal frame");
    let value = expr_col("value").expect("Decimal expression");
    let rounded =
        expr_alias(&expr_round(&value, 0).expect("round"), "rounded").expect("round alias");
    let output = frame
        .lazy()
        .select(compile_many(&[rounded]).expect("compile Decimal expression"))
        .collect()
        .expect("collect Decimal expression");
    assert_eq!(
        output.column("rounded").unwrap().dtype(),
        &DataType::Decimal(3, 1)
    );

    let decimal = Series::new("value".into(), [5_i128, -5_i128])
        .into_decimal(1, 1)
        .expect("sign Decimal fixture");
    let frame = DataFrame::new(2, vec![decimal.into_column()]).expect("sign Decimal frame");
    let value = expr_col("value").expect("sign Decimal expression");
    let signed = expr_alias(&expr_sign(&value).expect("sign"), "signed").expect("sign alias");
    let output = frame
        .lazy()
        .select(compile_many(&[signed]).expect("compile Decimal sign"))
        .collect()
        .expect("collect Decimal sign");
    assert_eq!(
        output.column("signed").unwrap().dtype(),
        &DataType::Decimal(2, 1)
    );
}

#[test]
fn sql_resolution_is_deferred_and_numeric_literals_are_exact() {
    let mut context = sql_context_new();
    let deferred = sql_context_execute(&mut context, "SELECT * FROM derived")
        .expect("unresolved SQL produces a lazy plan");
    assert!(collect_lazy(&deferred).is_err());

    let exact = sql_context_execute(&mut context, "SELECT 1.5 AS value")
        .expect("exact numeric literal plan");
    let output = collect_lazy(&exact).expect("collect exact numeric literal");
    assert_eq!(
        output.inner.column("value").unwrap().dtype(),
        &DataType::Decimal(2, 1)
    );
}

#[test]
fn polars_2_bin_families_cover_intervals_quantiles_and_ranks() {
    let frame = df!("x" => [-2_i64, -1, 0, 1, 2]).expect("interval fixture");
    let x = expr_col("x").expect("interval input");
    let left_closed = expr_alias(
        &expr_bin_intervals_int(&x, &[-1, 1], false, false).expect("left-closed bins"),
        "left",
    )
    .expect("left alias");
    let right_closed = expr_alias(
        &expr_bin_intervals_int(&x, &[-1, 1], true, false).expect("right-closed bins"),
        "right",
    )
    .expect("right alias");
    let intervals = expr_alias(
        &expr_bin_intervals_int(&x, &[-1, 1], false, true).expect("interval Struct"),
        "intervals",
    )
    .expect("interval alias");
    let output = frame
        .lazy()
        .select(compile_many(&[left_closed, right_closed, intervals]).expect("compile intervals"))
        .collect()
        .expect("collect intervals");
    assert_eq!(output.column("left").unwrap().dtype(), &DataType::UInt32);
    assert_eq!(
        (0..5)
            .map(|index| output
                .column("left")
                .unwrap()
                .get(index)
                .unwrap()
                .to_string())
            .collect::<Vec<_>>(),
        ["0", "1", "1", "2", "2"]
    );
    assert_eq!(
        (0..5)
            .map(|index| output
                .column("right")
                .unwrap()
                .get(index)
                .unwrap()
                .to_string())
            .collect::<Vec<_>>(),
        ["0", "0", "1", "1", "2"]
    );
    let DataType::Struct(fields) = output.column("intervals").unwrap().dtype() else {
        panic!("include_intervals did not produce a Struct")
    };
    assert_eq!(
        fields
            .iter()
            .map(|field| field.name().as_str())
            .collect::<Vec<_>>(),
        ["bin", "left", "right"]
    );

    let ties = df!("x" => [1_i64, 1, 2, 2]).expect("tie fixture");
    let x = expr_col("x").expect("tie input");
    let labels = ["a".into(), "b".into(), "c".into(), "d".into()];
    let quantiles = expr_alias(
        &expr_bin_quantiles_labeled(&x, &[0.1, 0.25, 0.75], &labels, false, false)
            .expect("quantile bins"),
        "quantiles",
    )
    .expect("quantile alias");
    let ranks = expr_alias(
        &expr_bin_ranks_labeled(
            &x,
            &[0.25, 0.75],
            &["low".into(), "mid".into(), "high".into()],
            false,
            false,
        )
        .expect("rank bins"),
        "ranks",
    )
    .expect("rank alias");
    let output = ties
        .lazy()
        .select(compile_many(&[quantiles, ranks]).expect("compile tie bins"))
        .collect()
        .expect("collect tie bins");
    assert_eq!(
        (0..4)
            .map(|index| output
                .column("quantiles")
                .unwrap()
                .get(index)
                .unwrap()
                .to_string())
            .collect::<Vec<_>>(),
        ["\"c\"", "\"c\"", "\"d\"", "\"d\""]
    );
    assert_eq!(
        (0..4)
            .map(|index| output
                .column("ranks")
                .unwrap()
                .get(index)
                .unwrap()
                .to_string())
            .collect::<Vec<_>>(),
        ["\"low\"", "\"mid\"", "\"mid\"", "\"high\""]
    );

    let strings = df!("x" => ["a", "m", "z"]).expect("String interval fixture");
    let x = expr_col("x").expect("String input");
    let bins =
        expr_bin_intervals_string(&x, &["m".into()], false, false).expect("String interval bins");
    let output = strings
        .lazy()
        .select(compile_many(&[bins]).expect("compile String bins"))
        .collect()
        .expect("collect String bins");
    assert_eq!(
        (0..3)
            .map(|index| output.column("x").unwrap().get(index).unwrap().to_string())
            .collect::<Vec<_>>(),
        ["0", "1", "1"]
    );

    let dates = Series::new("x".into(), [0_i32, 10])
        .cast(&DataType::Date)
        .expect("Date interval values");
    let frame = DataFrame::new(2, vec![dates.into_column()]).expect("Date interval fixture");
    let breaks = TerlanPolarsSeries {
        inner: Series::new("breaks".into(), [5_i32])
            .cast(&DataType::Date)
            .expect("Date interval breakpoints"),
    };
    let x = expr_col("x").expect("Date interval input");
    let bins =
        expr_bin_intervals_series(&x, &breaks, false, false).expect("Date Series interval bins");
    let output = frame
        .lazy()
        .select(compile_many(&[bins]).expect("compile Date bins"))
        .collect()
        .expect("collect Date bins");
    assert_eq!(
        (0..2)
            .map(|index| output.column("x").unwrap().get(index).unwrap().to_string())
            .collect::<Vec<_>>(),
        ["0", "1"]
    );

    let eager = TerlanPolarsSeries {
        inner: Series::new("x".into(), [-2_i64, -1, 0, 1, 2]),
    };
    let intervals =
        series_bin_intervals_int(&eager, &[-1, 1], false, false).expect("eager interval bins");
    assert_eq!(intervals.inner.dtype(), &DataType::UInt32);
    assert_eq!(
        (0..5)
            .map(|index| intervals.inner.get(index).unwrap().to_string())
            .collect::<Vec<_>>(),
        ["0", "1", "1", "2", "2"]
    );

    let ties = TerlanPolarsSeries {
        inner: Series::new("x".into(), [1_i64, 1, 2, 2]),
    };
    let quantiles = series_bin_quantiles_labeled(
        &ties,
        &[0.1, 0.25, 0.75],
        &["a".into(), "b".into(), "c".into(), "d".into()],
        false,
        false,
    )
    .expect("eager quantile bins");
    let ranks = series_bin_ranks_labeled(
        &ties,
        &[0.25, 0.75],
        &["low".into(), "mid".into(), "high".into()],
        false,
        false,
    )
    .expect("eager rank bins");
    assert_eq!(
        (0..4)
            .map(|index| quantiles.inner.get(index).unwrap().to_string())
            .collect::<Vec<_>>(),
        ["\"c\"", "\"c\"", "\"d\"", "\"d\""]
    );
    assert_eq!(
        (0..4)
            .map(|index| ranks.inner.get(index).unwrap().to_string())
            .collect::<Vec<_>>(),
        ["\"low\"", "\"mid\"", "\"mid\"", "\"high\""]
    );
}

#[test]
fn polars_2_bin_descriptors_validate_specs_and_labels() {
    let input = expr_col("x").expect("bin input");
    assert!(expr_bin_intervals(&input, &[2.0, 1.0], false, false).is_err());
    assert!(expr_bin_intervals_uniform(&input, 0, false, false).is_err());
    assert!(expr_bin_quantiles(&input, &[0.75, 0.25], false, false).is_err());
    assert!(
        expr_bin_quantiles_labeled(&input, &[0.5], &["only-one".into()], false, false,).is_err()
    );
    assert!(expr_bin_quantiles(&input, &[0.25, 0.25], false, false).is_ok());
}

#[test]
fn parquet_enum_logical_type_loads_as_string() {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "terlan-polars-enum-{}-{suffix}.parquet",
        std::process::id()
    ));
    let frame = TerlanPolarsDataFrame {
        inner: df!("color" => ["RED", "GREEN", "BLUE", "RED", "BLUE"]).expect("ENUM fixture"),
    };
    write_parquet(&frame, path.to_str().unwrap()).expect("write String parquet");

    // Rewrite the String schema element to the legacy ENUM converted type,
    // following Polars' own regression fixture. Compact Thrift field headers
    // encode the field-id delta in the high nibble and the type in the low.
    let old = [
        0x18, 0x05, b'c', b'o', b'l', b'o', b'r', 0x25, 0x00, 0x4c, 0x1c, 0x00, 0x00,
    ];
    let new = [0x18, 0x05, b'c', b'o', b'l', b'o', b'r', 0x25, 0x08];
    let data = std::fs::read(&path).expect("read parquet fixture");
    let matches = data
        .windows(old.len())
        .enumerate()
        .filter_map(|(index, window)| (window == old).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "String schema element must be unique");
    let index = matches[0];
    let mut rewritten = Vec::with_capacity(data.len() - (old.len() - new.len()));
    rewritten.extend_from_slice(&data[..index]);
    rewritten.extend_from_slice(&new);
    rewritten.extend_from_slice(&data[index + old.len()..]);
    let footer_offset = rewritten.len() - 8;
    let old_footer = u32::from_le_bytes(
        rewritten[footer_offset..footer_offset + 4]
            .try_into()
            .expect("footer length"),
    );
    let removed = u32::try_from(old.len() - new.len()).expect("removed byte count");
    rewritten[footer_offset..footer_offset + 4]
        .copy_from_slice(&(old_footer - removed).to_le_bytes());
    std::fs::write(&path, rewritten).expect("write ENUM fixture");

    let loaded = read_parquet(path.to_str().unwrap()).expect("read ENUM parquet");
    assert_eq!(
        loaded.inner.column("color").unwrap().dtype(),
        &DataType::String
    );
    assert_eq!(loaded.inner.height(), 5);
    std::fs::remove_file(path).expect("remove ENUM fixture");
}
