//! Tests for declarative expression UDF construction and application.

use crate::expressions::{compile_one, expr_add, expr_alias, expr_col, expr_lit};
use crate::udf::{apply_expression_udf, define_expression_udf, expression_udf_parameter};
use crate::TerlanPolarsScalar;

#[test]
fn udf_substitutes_nested_parameters_and_compiles() {
    let left = expression_udf_parameter(0).expect("left parameter");
    let right = expression_udf_parameter(1).expect("right parameter");
    let sum = expr_add(&left, &right).expect("sum template");
    let template = expr_alias(&sum, "total").expect("alias template");
    let udf = define_expression_udf(2, "Int64", &template).expect("define UDF");
    let first = expr_col("amount").expect("column argument");
    let second = expr_lit(&TerlanPolarsScalar::Int(5)).expect("literal argument");

    let applied = apply_expression_udf(&udf, &[first, second]).expect("apply expression UDF");

    let _ = compile_one(&applied).expect("substituted expression compiles");
    assert!(applied.contains("\"name\":\"total\""));
    assert!(!applied.contains("\"op\":\"parameter\""));
}

#[test]
fn udf_rejects_invalid_arities_and_parameter_indexes() {
    let parameter = expression_udf_parameter(0).expect("parameter");
    assert_eq!(
        define_expression_udf(-1, "Int64", &parameter)
            .expect_err("negative arity must fail")
            .code,
        "invalid_udf_arity"
    );
    assert_eq!(
        define_expression_udf(65, "Int64", &parameter)
            .expect_err("oversized arity must fail")
            .code,
        "invalid_udf_arity"
    );
    assert_eq!(
        expression_udf_parameter(-1)
            .expect_err("negative parameter must fail")
            .code,
        "invalid_udf_parameter"
    );
    assert_eq!(
        expression_udf_parameter(64)
            .expect_err("oversized parameter must fail")
            .code,
        "invalid_udf_parameter"
    );
    assert_eq!(
        define_expression_udf(0, "Int64", &parameter)
            .expect_err("out-of-range parameter must fail")
            .code,
        "invalid_udf_parameter"
    );
}

#[test]
fn udf_rejects_wrong_argument_count_and_forged_payloads() {
    let parameter = expression_udf_parameter(0).expect("parameter");
    let udf = define_expression_udf(1, "Int64", &parameter).expect("define UDF");
    assert_eq!(
        apply_expression_udf(&udf, &[])
            .expect_err("missing argument must fail")
            .code,
        "udf_arity_mismatch"
    );
    assert_eq!(
        apply_expression_udf(&udf, &[parameter.clone(), parameter])
            .expect_err("extra argument must fail")
            .code,
        "udf_arity_mismatch"
    );
    assert_eq!(
        apply_expression_udf("{}", &[])
            .expect_err("forged UDF must fail")
            .code,
        "invalid_udf"
    );
    assert_eq!(
        apply_expression_udf(
            r#"{"version":2,"arity":0,"template":{"op":"all"},"output_type":"Int64"}"#,
            &[],
        )
        .expect_err("unknown version must fail")
        .code,
        "invalid_udf"
    );
}

#[test]
fn udf_rejects_missing_or_invalid_output_types() {
    let parameter = expression_udf_parameter(0).expect("parameter");
    for output_type in ["", "not-a-polars-data-type"] {
        assert_eq!(
            define_expression_udf(1, output_type, &parameter)
                .expect_err("invalid output type must fail")
                .code,
            "invalid_udf_output_type"
        );
    }
}

#[test]
fn unresolved_parameter_never_reaches_polars() {
    let parameter = expression_udf_parameter(0).expect("parameter");
    assert_eq!(
        compile_one(&parameter)
            .expect_err("unresolved parameter must fail")
            .code,
        "unresolved_udf_parameter"
    );
}
