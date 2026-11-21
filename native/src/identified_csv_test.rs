//! Snapshot identity, source mutation, and bounded capture regressions.

use super::*;
use std::io::{Read, Write};

const CSV: &[u8] = b"date,price\n2024-01-01,1.5\n2024-01-02,2.5\n";
const DIGEST: &str = "a4d10fc37f0cb152b1b5ab0d423c3b1544e5d7bb33d9b7e35a9791e65a6b3521";

#[test]
fn minute_timestamps_match_the_existing_date_inference_reader() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(b"timestamp,close\n2024-01-01T00:00:00,1.5\n2024-01-01T00:01:00,2.5\n")
        .unwrap();
    let path = file.path().to_str().unwrap();
    let identified = read_csv_dates_identified(path).unwrap();
    let ordinary = crate::read_csv_dates(path).unwrap();
    let dtype = &crate::schema(&identified.dataframe)[0].data_type;
    assert!(dtype.starts_with("Datetime"), "{dtype}");
    assert_eq!(dtype, &crate::schema(&ordinary)[0].data_type);
    assert_eq!(
        crate::rows(&identified.dataframe, 2).unwrap(),
        crate::rows(&ordinary, 2).unwrap()
    );
}

#[test]
fn captured_bytes_survive_both_in_place_writes_and_path_replacement() {
    for replace_path in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("history.csv");
        std::fs::write(&path, CSV).unwrap();
        let snapshot = capture(std::fs::File::open(&path).unwrap()).unwrap();
        if replace_path {
            std::fs::rename(&path, directory.path().join("previous.csv")).unwrap();
        }
        std::fs::write(&path, b"date,price\n2025-01-01,999.0\n").unwrap();
        let identified = parse_snapshot(snapshot).unwrap();
        assert_eq!(identified.sha256, DIGEST);
        assert_eq!(crate::height(&identified.dataframe), 2);
        assert_eq!(crate::schema(&identified.dataframe)[0].data_type, "Date");
        assert_eq!(
            crate::rows(&identified.dataframe, 2).unwrap()[0],
            ["2024-01-01", "1.5"]
        );
    }
}

#[test]
fn identity_covers_raw_line_endings_not_normalized_rows() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(CSV).unwrap();
    let first = read_csv_dates_identified(file.path().to_str().unwrap()).unwrap();
    let crlf = String::from_utf8(CSV.to_vec())
        .unwrap()
        .replace('\n', "\r\n");
    std::fs::write(file.path(), crlf.as_bytes()).unwrap();
    let second = read_csv_dates_identified(file.path().to_str().unwrap()).unwrap();
    assert_eq!(first.sha256, DIGEST);
    assert_ne!(first.sha256, second.sha256);
    assert_eq!(
        crate::rows(&first.dataframe, 2).unwrap(),
        crate::rows(&second.dataframe, 2).unwrap()
    );
}

#[test]
fn capture_is_chunk_bounded_and_hashes_every_byte_across_short_reads() {
    struct Input {
        remaining: usize,
        interrupted: bool,
    }
    impl Read for Input {
        fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
            assert!(output.len() <= CAPTURE_BUFFER_BYTES);
            if !self.interrupted {
                self.interrupted = true;
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            let count = self.remaining.min(output.len()).min(7919);
            output[..count].fill(b'x');
            self.remaining -= count;
            Ok(count)
        }
    }
    let size = 257 * CAPTURE_BUFFER_BYTES + 7;
    let snapshot = capture(Input {
        remaining: size,
        interrupted: false,
    })
    .unwrap();
    assert_eq!(snapshot.file.metadata().unwrap().len(), size as u64);
    assert_eq!(
        snapshot.sha256,
        "761c71ebc0396453383d227e908b9867ac53c3202f7b529c12c1775f435d3793"
    );
}

#[test]
fn failed_reads_and_csv_errors_never_return_an_identified_frame() {
    struct FailingInput;
    impl Read for FailingInput {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("injected read failure"))
        }
    }
    assert!(capture(FailingInput).is_err());
    let directory = tempfile::tempdir().unwrap();
    let missing = read_csv_dates_identified(directory.path().join("missing.csv").to_str().unwrap());
    assert_eq!(missing.err().unwrap().code(), "csv_read_error");
    let malformed = directory.path().join("malformed.csv");
    std::fs::write(&malformed, b"a,b\n1,2\n3,4,5\n").unwrap();
    assert!(read_csv_dates_identified(malformed.to_str().unwrap()).is_err());
}
