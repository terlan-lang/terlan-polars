//! Tests for Apache Arrow IPC DataFrame interchange.

use crate::{
    dataframe_from_arrow_ipc, dataframe_to_arrow_ipc, frames_equal, from_rows,
    series_from_arrow_ipc, series_from_ints, series_integer_values, series_to_arrow_ipc,
};

#[test]
fn arrow_ipc_round_trip_preserves_schema_and_values() {
    let source = from_rows(
        &["name".to_string(), "score".to_string()],
        &[
            vec!["Ada".to_string(), "10".to_string()],
            vec!["Grace".to_string(), "20".to_string()],
        ],
    )
    .expect("source DataFrame");

    let bytes = dataframe_to_arrow_ipc(&source).expect("serialize Arrow IPC");
    let restored = dataframe_from_arrow_ipc(&bytes).expect("read Arrow IPC");

    assert!(frames_equal(&source, &restored));
}

#[test]
fn arrow_ipc_rejects_empty_and_malformed_payloads() {
    for payload in [&[][..], &[0, 1, 2, 3][..]] {
        assert_eq!(
            dataframe_from_arrow_ipc(payload)
                .expect_err("invalid Arrow IPC must fail")
                .code,
            "arrow_ipc_error"
        );
    }
}

#[test]
fn series_arrow_ipc_round_trip_preserves_name_type_and_values() {
    let source = series_from_ints("score", &[10, 20, 30]).expect("source Series");

    let bytes = series_to_arrow_ipc(&source).expect("serialize Series Arrow IPC");
    let restored = series_from_arrow_ipc(&bytes).expect("read Series Arrow IPC");

    assert_eq!(restored.inner.name().as_str(), "score");
    assert_eq!(
        series_integer_values(&restored).expect("integer values"),
        [10, 20, 30]
    );
}

#[test]
fn series_arrow_ipc_rejects_payloads_with_multiple_columns() {
    let dataframe = from_rows(
        &["left".to_string(), "right".to_string()],
        &[vec!["1".to_string(), "2".to_string()]],
    )
    .expect("two-column DataFrame");
    let bytes = dataframe_to_arrow_ipc(&dataframe).expect("serialize Arrow IPC");

    let error = series_from_arrow_ipc(&bytes).expect_err("multiple columns must fail");
    assert_eq!(error.code, "arrow_ipc_error");
    assert!(error.message.contains("exactly one column"));
}
