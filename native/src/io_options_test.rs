//! Executable coverage for immutable I/O option descriptors.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::io_options::*;
use crate::{columns, from_rows, height, rows, schema};

fn path(extension: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "terlan-polars-io-{}-{}.{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        extension
    ))
}

fn frame() -> crate::TerlanPolarsDataFrame {
    from_rows(
        &["name".to_string(), "score".to_string()],
        &[
            vec!["Ada".to_string(), "10".to_string()],
            vec!["Lin".to_string(), "20".to_string()],
            vec!["Sam".to_string(), "30".to_string()],
        ],
    )
    .expect("test frame")
}

fn default_csv_read() -> String {
    csv_read_options(
        true,
        ",",
        "\"",
        "\n",
        "utf8",
        "none",
        &[],
        &[],
        true,
        false,
        "",
        false,
        false,
        false,
        -1,
        false,
        -1,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        262_144,
        0,
        0,
        0,
        100,
        true,
        false,
        "",
        0,
    )
    .expect("default CSV read options")
}

fn polars_2_csv_read(
    has_header: bool,
    schema_names: &[String],
    schema_types: &[String],
    raise_if_empty: bool,
) -> String {
    csv_read_options(
        has_header,
        ",",
        "\"",
        "\n",
        "utf8",
        "none",
        &[],
        &[],
        true,
        false,
        "",
        false,
        false,
        false,
        -1,
        false,
        -1,
        &[],
        &[],
        schema_names,
        schema_types,
        &[],
        &[],
        &[],
        262_144,
        0,
        0,
        0,
        100,
        raise_if_empty,
        false,
        "",
        0,
    )
    .expect("Polars 2 CSV read options")
}

fn default_csv_write() -> String {
    csv_write_options(
        false,
        true,
        1024,
        "",
        "",
        "",
        -1,
        -1,
        false,
        ",",
        "\"",
        "",
        "\n",
        "necessary",
        -1,
    )
    .expect("default CSV write options")
}

#[test]
fn csv_options_execute_non_default_parse_and_serialization_behavior() {
    let input = path("csv");
    fs::write(
        &input,
        "# generated\nname;score;date\nAda;10;2026-07-30\nLin;;2026-07-31\n",
    )
    .expect("write CSV fixture");
    let read = csv_read_options(
        true,
        ";",
        "\"",
        "\n",
        "utf8",
        "single",
        &["".to_string()],
        &[],
        true,
        false,
        "#",
        true,
        false,
        true,
        1,
        false,
        2,
        &["score".to_string(), "date".to_string()],
        &[],
        &[],
        &[],
        &["score".to_string()],
        &["Float64".to_string()],
        &[],
        16,
        0,
        0,
        0,
        2,
        true,
        false,
        "row",
        7,
    )
    .expect("custom CSV read options");
    let parsed =
        read_csv_with_options(input.to_str().expect("path"), &read).expect("custom CSV read");
    assert_eq!(height(&parsed), 2);
    assert_eq!(columns(&parsed), ["row", "score", "date"]);
    assert_eq!(schema(&parsed)[1].data_type, "Float64");

    let output = path("csv");
    let write = csv_write_options(
        true, false, 1, "", "", "", 0, 3, false, ";", "'", "NULL", "\r\n", "always", 1,
    )
    .expect("custom CSV write options");
    write_csv_with_options(&parsed, output.to_str().expect("path"), &write)
        .expect("custom CSV write");
    let bytes = fs::read(&output).expect("read CSV output");
    assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]));
    let text = String::from_utf8(bytes[3..].to_vec()).expect("UTF-8 output");
    assert!(!text.starts_with("row"));
    assert!(text.contains("'7';'10.000';'2026-07-30'"));
    assert!(text.contains("\r\n"));

    fs::remove_file(input).ok();
    fs::remove_file(output).ok();
}

#[test]
fn csv_2_options_replace_names_and_validate_file_inference_count() {
    let input = path("csv");
    fs::write(&input, "name,score\nAda,10\n").expect("write CSV fixture");
    let renamed = csv_read_options_with_column_names(
        &default_csv_read(),
        &["person".to_string(), "points".to_string()],
    )
    .expect("replacement column names");
    let parsed = read_csv_with_options(input.to_str().expect("path"), &renamed)
        .expect("CSV with replacement names");
    assert_eq!(columns(&parsed), ["person", "points"]);

    let inferred =
        csv_read_options_with_infer_schema_files(&renamed, 3).expect("multi-file inference count");
    assert!(!inferred.is_empty());
    assert!(csv_read_options_with_infer_schema_files(&renamed, 0).is_err());
    assert!(csv_read_options_with_column_names(&renamed, &[String::new()]).is_err());
    fs::remove_file(input).ok();
}

#[test]
fn csv_2_defaults_names_schema_matching_and_empty_policy_execute() {
    let descriptor: serde_json::Value =
        serde_json::from_str(&default_csv_read()).expect("CSV options JSON");
    assert_eq!(descriptor["infer_schema_files"], 10);

    let headerless = path("csv");
    fs::write(&headerless, "a,1\n").expect("write headerless CSV");
    let parsed = read_csv_with_options(
        headerless.to_str().expect("path"),
        &polars_2_csv_read(false, &[], &[], true),
    )
    .expect("read headerless CSV");
    assert_eq!(columns(&parsed), ["column_0", "column_1"]);

    let named = path("csv");
    fs::write(&named, "a,b\nA,B\n").expect("write named CSV");
    let parsed = read_csv_with_options(
        named.to_str().expect("path"),
        &polars_2_csv_read(
            true,
            &["b".into(), "a".into()],
            &["String".into(), "String".into()],
            true,
        ),
    )
    .expect("read CSV with reordered schema");
    assert_eq!(columns(&parsed), ["b", "a"]);
    assert_eq!(rows(&parsed, 1).expect("schema-matched row")[0], ["B", "A"]);

    let empty = path("csv");
    fs::write(&empty, "").expect("write empty CSV");
    let parsed = read_csv_with_options(
        empty.to_str().expect("path"),
        &polars_2_csv_read(false, &["a".into()], &["Int64".into()], false),
    )
    .expect("read empty headerless CSV with schema");
    assert_eq!(height(&parsed), 0);
    assert_eq!(columns(&parsed), ["a"]);
    assert_eq!(schema(&parsed)[0].data_type, "Int64");

    fs::remove_file(headerless).ok();
    fs::remove_file(named).ok();
    fs::remove_file(empty).ok();
}

#[test]
fn csv_2_extra_and_missing_column_policies_execute() {
    let extra = path("csv");
    fs::write(&extra, "a,b\n1,2\n").expect("write extra-column CSV");
    let base = polars_2_csv_read(true, &["a".into()], &["Int64".into()], true);
    assert!(read_csv_with_options(extra.to_str().unwrap(), &base).is_err());
    let ignore = csv_read_options_with_column_policies(&base, "ignore", "raise")
        .expect("ignore extra columns");
    let parsed = read_csv_with_options(extra.to_str().unwrap(), &ignore)
        .expect("read with ignored extra column");
    assert_eq!(columns(&parsed), ["a"]);
    assert_eq!(rows(&parsed, 1).unwrap()[0], ["1"]);

    let missing = path("csv");
    fs::write(&missing, "a\n1\n").expect("write missing-column CSV");
    let base = polars_2_csv_read(
        true,
        &["a".into(), "b".into()],
        &["Int64".into(), "Int64".into()],
        true,
    );
    assert!(read_csv_with_options(missing.to_str().unwrap(), &base).is_err());
    let insert = csv_read_options_with_column_policies(&base, "raise", "insert")
        .expect("insert missing columns");
    let parsed = read_csv_with_options(missing.to_str().unwrap(), &insert)
        .expect("read with inserted missing column");
    assert_eq!(columns(&parsed), ["a", "b"]);
    assert_eq!(rows(&parsed, 1).unwrap()[0], ["1", "null"]);

    assert!(csv_read_options_with_column_policies(&base, "drop", "raise").is_err());
    assert!(csv_read_options_with_column_policies(&base, "raise", "null").is_err());
    fs::remove_file(extra).ok();
    fs::remove_file(missing).ok();
}

#[test]
fn json_options_select_schema_and_error_policy() {
    let input = path("ndjson");
    fs::write(
        &input,
        "{\"name\":\"Ada\",\"score\":10}\n{\"name\":\"Lin\",\"score\":20}\n",
    )
    .expect("write NDJSON fixture");
    let options = json_read_options(
        true,
        1,
        1,
        false,
        &["score".to_string()],
        &[],
        &[],
        &["score".to_string()],
        &["Int64".to_string()],
        true,
    )
    .expect("JSON read options");
    let parsed =
        read_json_with_options(input.to_str().expect("path"), &options).expect("JSON read");
    assert_eq!(height(&parsed), 2);
    assert_eq!(columns(&parsed), ["score"]);
    assert_eq!(rows(&parsed, 2).expect("rows")[0][0], "10");
    fs::remove_file(input).ok();
}

#[test]
fn parquet_options_round_trip_slice_projection_metadata_and_row_index() {
    let output = path("parquet");
    let write = parquet_write_options(
        "gzip",
        6,
        "full",
        2,
        1024,
        false,
        &["producer".to_string()],
        &["terlan".to_string()],
    )
    .expect("Parquet write options");
    write_parquet_with_options(&frame(), output.to_str().expect("path"), &write)
        .expect("Parquet write");
    assert_eq!(
        parquet_row_count(output.to_str().expect("path")).expect("Parquet row count"),
        3
    );
    let metadata: serde_json::Value = serde_json::from_str(
        &parquet_metadata(output.to_str().expect("path")).expect("Parquet metadata"),
    )
    .expect("Parquet metadata JSON");
    assert_eq!(metadata["num_rows"], 3);
    assert_eq!(metadata["key_values"]["producer"], "terlan");

    let read = parquet_read_options(
        1,
        1,
        &["name".to_string()],
        &[],
        "row",
        4,
        true,
        "none",
        true,
        "source",
    )
    .expect("Parquet read options");
    let parsed =
        read_parquet_with_options(output.to_str().expect("path"), &read).expect("Parquet read");
    assert_eq!(height(&parsed), 1);
    assert_eq!(columns(&parsed), ["row", "name", "source"]);
    let materialized = rows(&parsed, 1).expect("rows");
    assert_eq!(materialized[0][0], "5");
    assert_eq!(materialized[0][1], "Lin");
    assert_eq!(materialized[0][2], output.to_str().expect("path"));
    fs::remove_file(output).ok();
}

#[test]
fn ipc_options_round_trip_compression_metadata_projection_and_limit() {
    let compressed = path("ipc");
    let write = ipc_write_options(
        "zstd",
        "newest",
        2,
        true,
        false,
        &["producer".to_string()],
        &["terlan".to_string()],
    )
    .expect("IPC write options");
    write_ipc_with_options(&frame(), compressed.to_str().expect("path"), &write)
        .expect("IPC write");
    let metadata: serde_json::Value = serde_json::from_str(
        &ipc_custom_metadata(compressed.to_str().expect("path")).expect("IPC metadata"),
    )
    .expect("IPC metadata JSON");
    assert_eq!(metadata["producer"], "terlan");

    let read = ipc_read_options(
        2,
        &["score".to_string()],
        &[],
        "row",
        9,
        false,
        true,
        "source",
    )
    .expect("IPC read options");
    let parsed =
        read_ipc_with_options(compressed.to_str().expect("path"), &read).expect("IPC read");
    assert_eq!(height(&parsed), 2);
    assert_eq!(columns(&parsed), ["row", "score", "source"]);

    let memory_mapped = ipc_read_options(2, &[], &[], "", 0, true, false, "")
        .expect("legacy memory-mapped descriptor");
    let error = read_ipc_with_options(compressed.to_str().expect("path"), &memory_mapped)
        .expect_err("safe adapter rejects unsafe memory mapping");
    assert_eq!(error.code(), "invalid_io_options");
    fs::remove_file(compressed).ok();
}

#[test]
fn option_validation_rejects_crossed_modes_ranges_and_forged_descriptors() {
    assert_eq!(
        csv_read_options(
            true,
            "::",
            "\"",
            "\n",
            "utf8",
            "none",
            &[],
            &[],
            true,
            false,
            "",
            false,
            false,
            false,
            -1,
            false,
            -1,
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            1024,
            0,
            0,
            0,
            100,
            true,
            false,
            "",
            0,
        )
        .expect_err("multi-byte separator")
        .code(),
        "invalid_io_options"
    );
    assert!(csv_read_options(
        true,
        ",",
        "\"",
        "\n",
        "utf8",
        "named",
        &["NA".to_string()],
        &[],
        true,
        false,
        "",
        false,
        false,
        false,
        -1,
        false,
        -1,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        1024,
        0,
        0,
        0,
        100,
        true,
        false,
        "",
        0,
    )
    .is_err());
    assert!(csv_write_options(
        false, true, 0, "", "", "", 2, -1, false, ",", "\"", "", "\n", "bad", -1,
    )
    .is_err());
    assert!(json_read_options(false, 100, 8192, true, &[], &[], &[], &[], &[], true).is_err());
    assert!(parquet_read_options(0, -1, &[], &[], "", 0, false, "auto", false, "",).is_err());
    assert!(parquet_write_options("snappy", 3, "chunk", -1, -1, true, &[], &[]).is_err());
    assert!(ipc_write_options("bad", "newest", -1, false, true, &[], &[]).is_err());
    assert!(read_csv_with_options("unused", "{}").is_err());

    assert!(!default_csv_read().is_empty());
    assert!(!default_csv_write().is_empty());
}
