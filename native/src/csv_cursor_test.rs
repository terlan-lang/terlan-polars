use super::*;
use std::io::Write;

fn fixture(bytes: &[u8]) -> tempfile::NamedTempFile {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(bytes).unwrap();
    file
}

#[test]
fn snapshot_identity_batches_and_disposal_are_independent() {
    use sha2::{Digest, Sha256};
    let bytes = b"date,value\r\n2024-01-01,1\r\n2024-01-02,2\r\n2024-01-03,3";
    let file = fixture(bytes);
    let mut cursor = IdentifiedCsvCursor::open(file.path().to_str().unwrap(), 2).unwrap();
    assert_eq!(cursor.sha256, format!("{:x}", Sha256::digest(bytes)));
    std::fs::write(file.path(), b"replaced\n999\n").unwrap();
    let first = cursor.next_batch().unwrap().unwrap();
    let last = cursor.next_batch().unwrap().unwrap();
    assert_eq!(crate::height(&first), 2);
    assert_eq!(crate::height(&last), 1);
    assert_eq!(crate::schema(&first)[0].data_type, "Date");
    assert_eq!(crate::schema(&first), crate::schema(&last));
    assert!(cursor.next_batch().unwrap().is_none());
    assert!(cursor.next_batch().unwrap().is_none());
    drop(cursor);
    assert_eq!(crate::rows(&last, 1).unwrap()[0], ["2024-01-03", "3"]);
    assert_eq!(crate::height(&first), 2);
}

#[test]
fn quoted_newlines_escaped_quotes_and_utf8_cross_batches() {
    let file = fixture(b"name,value\n\"a\n\"\"b\"\",\xc4\x81\",1\nlast,2\n");
    let mut cursor = IdentifiedCsvCursor::open(file.path().to_str().unwrap(), 1).unwrap();
    assert_eq!(
        crate::rows(&cursor.next_batch().unwrap().unwrap(), 1).unwrap()[0][0],
        "a\n\"b\",ā"
    );
    assert_eq!(
        crate::rows(&cursor.next_batch().unwrap().unwrap(), 1).unwrap()[0][0],
        "last"
    );
    assert!(cursor.next_batch().unwrap().is_none());
}

#[test]
fn byte_cap_splits_batches_and_oversized_records_poison_cursor() {
    let mut file = fixture(b"value\n");
    let record = "x".repeat(MAX_BATCH_BYTES / 2);
    writeln!(file, "{record}\n{record}\n{record}").unwrap();
    let mut cursor = IdentifiedCsvCursor::open(file.path().to_str().unwrap(), 10000).unwrap();
    for _ in 0..3 {
        assert_eq!(crate::height(&cursor.next_batch().unwrap().unwrap()), 1);
        assert!(cursor
            .pending
            .as_ref()
            .map_or(true, |row| row.len() <= MAX_BATCH_BYTES));
    }
    assert!(cursor.next_batch().unwrap().is_none());
    let file = fixture(format!("value\n{}\n", "x".repeat(MAX_BATCH_BYTES)).as_bytes());
    let mut cursor = IdentifiedCsvCursor::open(file.path().to_str().unwrap(), 1).unwrap();
    assert_eq!(
        cursor.next_batch().err().unwrap().code(),
        "csv_record_limit"
    );
    assert!(cursor.reader.is_none());
    assert_eq!(
        cursor.next_batch().err().unwrap().code(),
        "csv_cursor_failed"
    );
}

#[test]
fn invalid_inputs_and_later_schema_changes_fail_closed() {
    let file = fixture(b"value\n1\ntext\n");
    assert!(IdentifiedCsvCursor::open(file.path().to_str().unwrap(), 0).is_err());
    assert!(IdentifiedCsvCursor::open(file.path().to_str().unwrap(), MAX_BATCH_ROWS + 1).is_err());
    let mut cursor = IdentifiedCsvCursor::open(file.path().to_str().unwrap(), 1).unwrap();
    assert!(cursor.next_batch().unwrap().is_some());
    assert!(cursor.next_batch().is_err());
    assert!(cursor.reader.is_none());
    let file = fixture(b"value\n\"unfinished");
    let mut cursor = IdentifiedCsvCursor::open(file.path().to_str().unwrap(), 1).unwrap();
    assert!(cursor.next_batch().is_err());
    let file = fixture(b"value\n");
    assert!(IdentifiedCsvCursor::open(file.path().to_str().unwrap(), 1)
        .unwrap()
        .next_batch()
        .unwrap()
        .is_none());
    let file = fixture(b"");
    assert!(IdentifiedCsvCursor::open(file.path().to_str().unwrap(), 1).is_err());
}

#[test]
fn full_history_keeps_only_current_batch_and_small_cursor_state() {
    let mut file = fixture(b"minute,value\n");
    for i in 0..100_001 {
        writeln!(file, "{i},{i}").unwrap();
    }
    let mut cursor = IdentifiedCsvCursor::open(file.path().to_str().unwrap(), 997).unwrap();
    let mut rows = 0;
    while let Some(frame) = cursor.next_batch().unwrap() {
        assert!(crate::height(&frame) <= 997);
        rows += crate::height(&frame);
        assert!(cursor.header.len() < 100);
        assert!(cursor.pending.is_none());
    }
    assert_eq!(rows, 100_001);
    assert!(cursor.reader.is_none());
}
