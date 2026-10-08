//! Behavioral tests for extended LazyFrame operations.

use crate::*;

#[test]
fn lazy_casts_change_resolved_schema_without_collecting() {
    let frame = from_rows(
        &["value".to_string(), "other".to_string()],
        &[
            vec!["1".to_string(), "2".to_string()],
            vec!["3".to_string(), "4".to_string()],
        ],
    )
    .expect("source frame");
    let lazy = to_lazy(&frame);
    let named = lazy_cast_columns(
        &lazy,
        &["value".to_string()],
        &["Float64".to_string()],
        true,
    )
    .expect("named cast");
    let schema = lazy_schema(&named).expect("named schema");
    assert_eq!(schema[0].data_type, "Float64");
    assert_eq!(schema[1].data_type, "String");

    let all = lazy_cast_all(&lazy, "Float64", false).expect("whole-frame cast");
    assert!(lazy_schema(&all)
        .expect("whole-frame schema")
        .iter()
        .all(|column| column.data_type == "Float64"));

    let mismatch = match lazy_cast_columns(&lazy, &["value".to_string()], &[], true) {
        Ok(_) => panic!("mismatched cast vectors must fail"),
        Err(error) => error,
    };
    assert_eq!(mismatch.code(), "invalid_columns");
    assert!(lazy_cast_all(&lazy, "not-a-dtype", true).is_err());
}

#[test]
fn lazy_optimizer_inventory_tracks_immutable_toggles() {
    let frame = from_rows(&["value".to_string()], &[vec!["1".to_string()]]).expect("source frame");
    let original = to_lazy(&frame);
    let original_flags = lazy_current_optimizations(&original);
    assert!(original_flags.contains(&"projection_pushdown".to_string()));

    let disabled = lazy_set_optimization(&original, "projection_pushdown", false)
        .expect("disable projection pushdown");
    assert!(!lazy_current_optimizations(&disabled).contains(&"projection_pushdown".to_string()));
    assert!(lazy_current_optimizations(&original).contains(&"projection_pushdown".to_string()));
}

#[test]
fn lazy_pivot_uses_explicit_output_columns_and_validates_options() {
    let source = from_rows(
        &["index".to_string(), "on".to_string(), "value".to_string()],
        &[
            vec!["x".to_string(), "a".to_string(), "1".to_string()],
            vec!["x".to_string(), "b".to_string(), "2".to_string()],
        ],
    )
    .expect("pivot source");
    let on_columns = select(&source, &["on".to_string()]).expect("pivot output columns");
    let pivoted = lazy_pivot(
        &to_lazy(&source),
        &on_columns,
        &["on".to_string()],
        &["index".to_string()],
        &["value".to_string()],
        "first",
        true,
        "_",
        "auto",
    )
    .expect("lazy pivot");
    let output = collect_lazy(&pivoted).expect("collect pivot");
    assert_eq!(height(&output), 1);
    assert_eq!(width(&output), 3);

    assert!(lazy_pivot(
        &to_lazy(&source),
        &on_columns,
        &[],
        &["index".to_string()],
        &["value".to_string()],
        "first",
        true,
        "_",
        "auto",
    )
    .is_err());
    assert!(lazy_pivot(
        &to_lazy(&source),
        &on_columns,
        &["on".to_string()],
        &["index".to_string()],
        &["value".to_string()],
        "first",
        true,
        "_",
        "unknown",
    )
    .is_err());
}

#[test]
fn lazy_plan_sets_collect_multiple_results_and_provide_context() {
    let first = from_rows(&["first".to_string()], &[vec!["1".to_string()]]).expect("first frame");
    let second = from_rows(
        &["second".to_string()],
        &[vec!["2".to_string()], vec!["3".to_string()]],
    )
    .expect("second frame");
    let first_plan = to_lazy(&first);
    let second_plan = to_lazy(&second);

    let one = lazyframe_set_new(&first_plan);
    let both = lazyframe_set_append(&one, &second_plan);
    assert_eq!(lazyframe_set_len(&one), 1);
    assert_eq!(lazyframe_set_len(&both), 2);

    let results = lazy_collect_all(&both, false).expect("collect all plans");
    assert_eq!(dataframe_set_len(&results), 2);
    assert_eq!(
        height(&dataframe_set_get(&results, 0).expect("first result")),
        1
    );
    assert_eq!(
        height(&dataframe_set_get(&results, 1).expect("second result")),
        2
    );
    let in_memory = lazy_collect_with_engine(&first_plan, "in_memory").expect("in-memory collect");
    assert_eq!(height(&in_memory), 1);
    let automatic = lazy_collect_with_engine(&first_plan, "auto").expect("automatic collect");
    assert_eq!(height(&automatic), 1);
    let streaming =
        lazy_collect_all_with_engine(&both, "streaming").expect("streaming collect all");
    assert_eq!(dataframe_set_len(&streaming), 2);
    assert_eq!(
        lazy_collect_with_engine(&first_plan, "unknown")
            .expect_err("invalid engine")
            .code(),
        "invalid_engine"
    );
    assert_eq!(
        lazy_collect_with_engine(&first_plan, "gpu")
            .expect_err("disabled GPU engine")
            .code(),
        "unsupported_engine"
    );

    let contexts = lazyframe_set_new(&second_plan);
    let contextual = lazy_with_context(&first_plan, &contexts);
    assert!(lazy_describe_plan(&contextual, false)
        .expect("contextual plan")
        .contains("HCONCAT"));
}
