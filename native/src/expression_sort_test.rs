//! Generic expression sorting parity tests.

use crate::expressions::{
    compile_many, expr_alias, expr_col, expr_hash, expr_is_sorted, expr_sort_with,
    expr_sort_with_stability,
};
use polars::df;
use polars::prelude::IntoLazy;

#[test]
fn stable_sort_is_encoded_and_executes() {
    let input = expr_col("value").expect("column expression");
    let unstable = expr_sort_with(&input, false, false).expect("ordinary sort");
    let stable =
        expr_sort_with_stability(&input, false, false, true).expect("stable sort expression");

    let unstable_json: serde_json::Value =
        serde_json::from_str(&unstable).expect("ordinary sort JSON");
    let stable_json: serde_json::Value = serde_json::from_str(&stable).expect("stable sort JSON");
    assert_eq!(unstable_json["maintain_order"], false);
    assert_eq!(stable_json["maintain_order"], true);

    let stable = expr_alias(&stable, "sorted").expect("stable sort alias");
    let output = df!("value" => [2_i64, 1, 2, 1])
        .expect("sort fixture")
        .lazy()
        .select(compile_many(&[stable]).expect("compiled stable sort"))
        .collect()
        .expect("stable sort should execute");
    assert_eq!(
        output
            .column("sorted")
            .expect("sorted column")
            .i64()
            .expect("integer column")
            .into_no_null_iter()
            .collect::<Vec<_>>(),
        vec![1, 1, 2, 2]
    );
}

#[test]
fn expression_is_sorted_checks_requested_direction() {
    let input = expr_col("value").expect("column expression");
    let ascending = expr_alias(
        &expr_is_sorted(&input, "ascending", "unknown").expect("ascending check"),
        "ascending",
    )
    .expect("ascending alias");
    let descending = expr_alias(
        &expr_is_sorted(&input, "descending", "unknown").expect("descending check"),
        "descending",
    )
    .expect("descending alias");
    let output = df!("value" => [1_i64, 2, 3])
        .expect("sorted fixture")
        .lazy()
        .select(compile_many(&[ascending, descending]).expect("compiled sorted checks"))
        .collect()
        .expect("sorted checks should execute");
    assert!(output
        .column("ascending")
        .unwrap()
        .bool()
        .unwrap()
        .get(0)
        .unwrap());
    assert!(!output
        .column("descending")
        .unwrap()
        .bool()
        .unwrap()
        .get(0)
        .unwrap());
    assert!(expr_is_sorted(&input, "sideways", "unknown").is_err());
    assert!(expr_is_sorted(&input, "ascending", "middle").is_err());
}

#[test]
fn expression_hash_is_seeded_and_validated() {
    let input = expr_col("value").expect("column expression");
    let first = expr_alias(
        &expr_hash(&input, 42).expect("first hash expression"),
        "first",
    )
    .expect("first alias");
    let repeated = expr_alias(
        &expr_hash(&input, 42).expect("repeated hash expression"),
        "repeated",
    )
    .expect("repeated alias");
    let changed = expr_alias(
        &expr_hash(&input, 43).expect("changed hash expression"),
        "changed",
    )
    .expect("changed alias");

    let output = df!("value" => [10_i64, 20, 30])
        .expect("hash fixture")
        .lazy()
        .select(compile_many(&[first, repeated, changed]).expect("compiled hashes"))
        .collect()
        .expect("hash expressions should execute");
    let first = output.column("first").expect("first hash");
    let repeated = output.column("repeated").expect("repeated hash");
    let changed = output.column("changed").expect("changed hash");
    assert!(first.equals(repeated));
    assert!(!first.equals(changed));

    assert_eq!(
        expr_hash(&input, -1).expect_err("negative seed").code(),
        "invalid_expression"
    );
}
