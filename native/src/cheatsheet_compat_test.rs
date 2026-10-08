use std::fs;
use std::path::Path;

use polars::df;

use crate::dataframe_extended::{dataframe_set_get, dataframe_set_len};
use crate::expressions::{
    expr_alias, expr_col, expr_date_components, expr_gt, expr_lit, expr_log10, expr_lt,
    expr_when_chain,
};
use crate::*;

fn owned(inner: polars::prelude::DataFrame) -> TerlanPolarsDataFrame {
    TerlanPolarsDataFrame { inner }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn inspection_and_plan_diagnostics_are_behavioral() {
    let frame = owned(
        df!(
            "name" => ["Ada", "Grace", "Linus"],
            "score" => [Some(1_i64), None, Some(3)]
        )
        .unwrap(),
    );

    assert!(show_versions().contains("polars py-2.0.0"));
    let overview = glimpse(&frame).unwrap();
    assert!(overview.contains("Rows: 3"));
    assert!(overview.contains("$ score <"));

    let summary = describe(&frame).unwrap();
    assert_eq!(height(&summary), 9);
    assert_eq!(columns(&summary), ["statistic", "name", "score"]);

    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("test")
        .join("fixtures")
        .join("people.csv");
    let scanned = lazy_scan_csv(fixture.to_str().unwrap(), false).unwrap();
    let selected = lazy_select(&scanned, &["age".to_string()]);
    let plan = lazy_sort(&selected, "age", true);
    let dot = lazy_describe_plan_dot(&plan, true).unwrap();
    assert!(dot.contains("digraph"));
    let profiled = lazy_profile(&plan).unwrap();
    assert_eq!(dataframe_set_len(&profiled), 2);
    assert_eq!(height(&dataframe_set_get(&profiled, 0).unwrap()), 2);
    assert!(width(&dataframe_set_get(&profiled, 1).unwrap()) >= 2);
}

#[test]
fn direct_dataframe_conveniences_and_temporal_numeric_builders_execute() {
    let frame = owned(
        df!(
            "a" => [1_i64, 2, 3, 4, 5],
            "b" => [10_i64, 20, 30, 40, 50],
            "year" => [2024_i32, 2024, 2025, 2025, 2026],
            "month" => [1_i8, 2, 3, 4, 5],
            "day" => [2_i8, 3, 4, 5, 6],
            "power" => [1.0_f64, 10.0, 100.0, 1000.0, 10_000.0]
        )
        .unwrap(),
    );

    assert_eq!(
        data_types(&frame),
        ["Int64", "Int64", "Int32", "Int8", "Int8", "Float64"]
    );
    assert_eq!(
        estimated_size_with_unit(&frame, "b").unwrap(),
        estimated_size(&frame) as f64
    );
    assert_eq!(
        estimated_size_with_unit(&frame, "mb").unwrap(),
        estimated_size(&frame) as f64 / 1024.0_f64.powi(2)
    );
    assert_eq!(
        estimated_size_with_unit(&frame, "milkshake")
            .unwrap_err()
            .code(),
        "invalid_size_unit"
    );

    let dropped = drop_columns_with_options(&frame, &strings(&["b", "absent"]), false).unwrap();
    assert!(!columns(&dropped).contains(&"b".to_string()));
    assert!(drop_columns_with_options(&frame, &strings(&["absent"]), true).is_err());

    let gathered = gather_rows_every(&frame, 2, 1).unwrap();
    assert_eq!(rows(&gathered, 2).unwrap()[0][0], "2");
    assert_eq!(rows(&gathered, 2).unwrap()[1][0], "4");
    assert_eq!(
        gather_rows_every(&frame, 0, 0).unwrap_err().code(),
        "invalid_row_step"
    );

    let one = expr_lit(&TerlanPolarsScalar::Int(1)).unwrap();
    let five = expr_lit(&TerlanPolarsScalar::Int(5)).unwrap();
    let filtered = filter_all(
        &frame,
        &[
            expr_gt(&expr_col("a").unwrap(), &one).unwrap(),
            expr_lt(&expr_col("a").unwrap(), &five).unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(height(&filtered), 3);
    assert_eq!(
        filter_all(&frame, &[]).unwrap_err().code(),
        "invalid_expression"
    );

    let constrained = filter_constraints(
        &frame,
        &strings(&["year", "month"]),
        &[
            expr_lit(&TerlanPolarsScalar::Int(2025)).unwrap(),
            expr_lit(&TerlanPolarsScalar::Int(3)).unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(height(&constrained), 1);
    assert_eq!(rows(&constrained, 1).unwrap()[0][0], "3");
    assert_eq!(
        filter_constraints(&frame, &strings(&["year"]), &[])
            .unwrap_err()
            .code(),
        "invalid_expression"
    );

    let date = expr_date_components(
        &expr_col("year").unwrap(),
        &expr_col("month").unwrap(),
        &expr_col("day").unwrap(),
    )
    .unwrap();
    let logarithm = expr_alias(&expr_log10(&expr_col("power").unwrap()).unwrap(), "log10").unwrap();
    let projected = select_exprs(&frame, &[date, logarithm]).unwrap();
    assert_eq!(columns(&projected), ["year", "log10"]);
    assert_eq!(rows(&projected, 1).unwrap()[0], ["2024-01-02", "0.0"]);

    let two = expr_lit(&TerlanPolarsScalar::Int(2)).unwrap();
    let four = expr_lit(&TerlanPolarsScalar::Int(4)).unwrap();
    let chain = expr_when_chain(
        &[
            expr_lt(&expr_col("a").unwrap(), &two).unwrap(),
            expr_lt(&expr_col("a").unwrap(), &four).unwrap(),
        ],
        &[
            expr_lit(&TerlanPolarsScalar::String("small".to_string())).unwrap(),
            expr_lit(&TerlanPolarsScalar::String("medium".to_string())).unwrap(),
        ],
        &expr_lit(&TerlanPolarsScalar::String("large".to_string())).unwrap(),
    )
    .unwrap();
    let classified = select_exprs(&frame, &[expr_alias(&chain, "size").unwrap()]).unwrap();
    assert_eq!(
        rows(&classified, 5).unwrap(),
        [["small"], ["medium"], ["medium"], ["large"], ["large"],]
    );
    assert_eq!(
        expr_when_chain(&[], &[], &two).unwrap_err().code(),
        "invalid_expression"
    );
    assert_eq!(
        expr_when_chain(&[expr_col("a").unwrap()], &[], &two)
            .unwrap_err()
            .code(),
        "invalid_expression"
    );
}

#[test]
fn relaxed_concat_join_update_and_group_map_execute() {
    let integers = owned(df!("value" => [1_i32, 2]).unwrap());
    let floats = owned(df!("value" => [3.5_f64]).unwrap());
    let concatenated = concat_vertical_relaxed(&integers, &floats).unwrap();
    assert_eq!(height(&concatenated), 3);
    assert_eq!(schema(&concatenated)[0].data_type, "Float64");

    let left = owned(df!("id" => [1_i64, 2], "value" => ["old", "keep"]).unwrap());
    let right = owned(df!("key" => [1_i64, 3], "value" => ["new", "added"]).unwrap());
    let joined = join_with_coalesce(
        &left,
        &right,
        &strings(&["id"]),
        &strings(&["key"]),
        "full",
        "_right",
        false,
        true,
    )
    .unwrap();
    assert!(columns(&joined).contains(&"id".to_string()));
    assert!(!columns(&joined).contains(&"key".to_string()));

    let updates = owned(df!("id" => [1_i64], "value" => ["replacement"]).unwrap());
    let updated = update(&left, &updates, &strings(&["id"]), "left", false).unwrap();
    assert_eq!(
        rows(&updated, 2).unwrap(),
        [
            vec!["1".to_string(), "replacement".to_string()],
            vec!["2".to_string(), "keep".to_string()]
        ]
    );

    let grouped = owned(df!("group" => ["a", "a", "b"], "value" => [1_i64, 2, 3]).unwrap());
    let value = expr_alias(&expr_col("value").unwrap(), "mapped").unwrap();
    let mapped = map_groups(&grouped, &strings(&["group"]), &[value]).unwrap();
    assert_eq!(height(&mapped), 3);
    assert_eq!(columns(&mapped), ["mapped"]);
}

#[test]
fn upsample_and_storage_option_validation_execute() {
    let frame = owned(df!("time" => [1_i64, 3], "value" => [10_i64, 30]).unwrap());
    let sampled = upsample(&frame, "time", "1i", &[], true).unwrap();
    assert_eq!(height(&sampled), 3);

    let error = match lazy_scan_parquet_with_storage_options(
        "s3://example/data.parquet",
        &strings(&["aws_region"]),
        &[],
    ) {
        Ok(_) => panic!("mismatched option lists must fail before I/O"),
        Err(error) => error,
    };
    assert_eq!(error.code(), "invalid_storage_options");
}

fn contains_file(path: &Path) -> bool {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| {
            let path = entry.path();
            path.is_file() || (path.is_dir() && contains_file(&path))
        })
}

#[test]
fn keyed_partition_sink_writes_files() {
    let frame = owned(df!("group" => ["a", "a", "b"], "value" => [1_i64, 2, 3]).unwrap());
    let plan = to_lazy(&frame);
    let key = expr_col("group").unwrap();
    let base = std::env::temp_dir().join(format!(
        "terlan-polars-partition-test-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("worker")
    ));
    let _ = fs::remove_dir_all(&base);
    lazy_write_parquet_partitioned(&plan, base.to_str().unwrap(), &[key], true, 10, 0).unwrap();
    assert!(contains_file(&base));
    fs::remove_dir_all(base).unwrap();
}
