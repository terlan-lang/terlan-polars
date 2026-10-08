//! Behavioral tests for extended immutable DataFrame operations.

use crate::dataframe_extended::*;
use crate::{
    columns, frames_equal_missing, from_rows, height, series_from_ints, series_values, width,
};

/// Constructs a two-column String DataFrame fixture.
fn frame() -> crate::TerlanPolarsDataFrame {
    from_rows(
        &["name".to_string(), "value".to_string()],
        &[
            vec!["alpha".to_string(), "1".to_string()],
            vec!["beta".to_string(), "2".to_string()],
        ],
    )
    .expect("DataFrame fixture")
}

#[test]
fn schema_shape_chunk_and_row_metadata_execute() {
    let dataframe = frame();
    assert_eq!(dataframe_chunk_lengths(&dataframe), [2]);
    assert_eq!(dataframe_shape(&dataframe), [2, 2]);
    assert_eq!(dataframe_first_col_n_chunks(&dataframe), 1);
    assert_eq!(dataframe_max_chunk_count(&dataframe), 1);
    assert!(!dataframe_should_rechunk(&dataframe));
    assert!(!dataframe_shape_has_zero(&dataframe));
    assert_eq!(dataframe_column_index(&dataframe, "value"), Ok(1));
    assert_eq!(
        dataframe_column_index(&dataframe, "missing")
            .expect_err("missing column")
            .code(),
        "missing_column"
    );
    let column = dataframe_column_at_index(&dataframe, 1).expect("second column");
    assert_eq!(
        series_values(&column, 2).expect("column values"),
        ["1", "2"]
    );
    assert_eq!(
        dataframe_column_at_index(&dataframe, 2)
            .expect_err("column bounds")
            .code(),
        "row_index_out_of_bounds"
    );
    assert_eq!(
        dataframe_row(&dataframe, 1).expect("second row"),
        ["beta", "2"]
    );
    assert_eq!(
        dataframe_row(&dataframe, 2).expect_err("row bounds").code(),
        "row_index_out_of_bounds"
    );

    let empty_height = dataframe_empty_with_height(3).expect("empty height");
    assert_eq!(height(&empty_height), 3);
    assert_eq!(width(&empty_height), 0);
    assert_eq!(dataframe_shape(&empty_height), [3, 0]);
    assert_eq!(dataframe_first_col_n_chunks(&empty_height), 0);
    assert!(dataframe_shape_has_zero(&empty_height));

    let names = ["id".to_string(), "label".to_string()];
    let data_types = ["Int64".to_string(), "String".to_string()];
    let empty = dataframe_empty_with_schema(&names, &data_types).expect("empty typed DataFrame");
    let nulls = dataframe_full_null(&names, &data_types, 2).expect("null DataFrame");
    assert_eq!(height(&empty), 0);
    assert_eq!(height(&nulls), 2);
    assert!(dataframe_schema_equal(&empty, &nulls));
    assert_eq!(
        dataframe_full_null(&names, &data_types[..1], 1)
            .expect_err("schema width")
            .code(),
        "invalid_columns"
    );
}

#[test]
fn split_column_and_struct_operations_are_immutable() {
    let dataframe = frame();
    let before = dataframe_split_before(&dataframe, 1).expect("split before");
    let after = dataframe_split_after(&dataframe, 1).expect("split after");
    assert_eq!(height(&before), 1);
    assert_eq!(height(&after), 1);
    assert_eq!(height(&dataframe), 2);

    let ids = series_from_ints("id", &[10, 20]).expect("id Series");
    let inserted = dataframe_insert_column(&dataframe, 0, &ids).expect("insert column");
    assert_eq!(columns(&inserted), ["id", "name", "value"]);
    assert_eq!(columns(&dataframe), ["name", "value"]);

    let replacement = series_from_ints("replacement", &[30, 40]).expect("replacement Series");
    let replaced = dataframe_replace_column(&inserted, 0, &replacement).expect("replace column");
    assert_eq!(columns(&replaced), ["replacement", "name", "value"]);

    let extra = series_from_ints("extra", &[50, 60]).expect("extra Series");
    let extended = dataframe_with_column(&replaced, &extra).expect("with column");
    assert_eq!(
        columns(&extended),
        ["replacement", "name", "value", "extra"]
    );

    let packed = dataframe_to_struct(&extended, "row").expect("Struct Series");
    assert_eq!(series_values(&packed, 2).expect("Struct values").len(), 2);
    assert!(format!("{:?}", packed.inner.dtype()).starts_with("Struct("));

    let shrunk = dataframe_shrink_to_fit(&extended).expect("shrink DataFrame");
    assert!(frames_equal_missing(&extended, &shrunk));
}

#[test]
fn binary_serialization_round_trip_rejects_malformed_input() {
    let dataframe = frame();
    let bytes = dataframe_serialize(&dataframe).expect("serialize DataFrame");
    assert!(!bytes.is_empty());
    let restored = dataframe_deserialize(&bytes).expect("deserialize DataFrame");
    assert!(frames_equal_missing(&dataframe, &restored));
    assert!(dataframe_schema_equal(&dataframe, &restored));
    assert!(dataframe_deserialize(b"not a Polars DataFrame").is_err());
}

#[test]
fn partition_sets_preserve_group_order_and_independent_ownership() {
    let dataframe = from_rows(
        &["group".to_string(), "value".to_string()],
        &[
            vec!["b".to_string(), "1".to_string()],
            vec!["a".to_string(), "2".to_string()],
            vec!["b".to_string(), "3".to_string()],
        ],
    )
    .expect("partition fixture");
    let set = dataframe_partition_by(&dataframe, &["group".to_string()], true, false)
        .expect("stable partitions");
    assert_eq!(dataframe_set_len(&set), 2);

    let first = dataframe_set_get(&set, 0).expect("first partition");
    let second = dataframe_set_get(&set, 1).expect("second partition");
    assert_eq!(columns(&first), ["value"]);
    assert_eq!(height(&first), 2);
    assert_eq!(height(&second), 1);
    assert_eq!(
        dataframe_row(&first, 0).expect("first partition row"),
        ["1"]
    );

    assert_eq!(
        dataframe_set_get(&set, 2)
            .expect_err("partition bounds")
            .code(),
        "row_index_out_of_bounds"
    );
    assert_eq!(
        dataframe_partition_by(&dataframe, &[], true, true)
            .expect_err("empty partition keys")
            .code(),
        "invalid_columns"
    );
}

#[test]
fn chunk_sets_preserve_rows_and_reject_zero_partitions() {
    let dataframe = from_rows(
        &["value".to_string()],
        &[
            vec!["1".to_string()],
            vec!["2".to_string()],
            vec!["3".to_string()],
        ],
    )
    .expect("chunk fixture");
    let split = dataframe_split_chunks_by_n(&dataframe, 2, true).expect("two row partitions");
    assert_eq!(dataframe_set_len(&split), 2);
    assert_eq!(
        height(&dataframe_set_get(&split, 0).expect("first chunk"))
            + height(&dataframe_set_get(&split, 1).expect("second chunk")),
        3
    );

    let logical = dataframe_split_chunks(&dataframe, false).expect("logical chunks");
    let physical = dataframe_split_chunks(&dataframe, true).expect("physical chunks");
    assert_eq!(dataframe_set_len(&logical), 1);
    assert_eq!(dataframe_set_len(&physical), 1);
    assert_eq!(
        dataframe_split_chunks_by_n(&dataframe, 0, false)
            .expect_err("zero partitions")
            .code(),
        "invalid_row_limit"
    );
}
