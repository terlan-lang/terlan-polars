use super::*;
use std::io::Cursor;

#[test]
fn large_string_rows_use_correlated_bounded_frames_and_round_trip() {
    let rows = vec![vec!["ā,\"line\n".repeat(200)]; 2000];
    let reply = format!(
        "reply 42 1 result_ok_string_rows {}",
        encode_string_rows(&rows)
    );
    assert!(reply.len() > TRANSPORT_LIMITS.frame);
    let mut wire = Vec::new();
    transport::write_message(
        &mut wire,
        &reply,
        42,
        transport::Direction::Reply,
        TRANSPORT_LIMITS,
    )
    .unwrap();
    for frame in wire.split_inclusive(|byte| *byte == b'\n') {
        assert!(frame.len() <= TRANSPORT_LIMITS.frame);
        assert!(frame.starts_with(b"reply_chunk 42 "));
    }
    let decoded = transport::read_message(
        &mut Cursor::new(&wire),
        transport::Direction::Reply,
        Some(42),
        TRANSPORT_LIMITS,
    )
    .unwrap()
    .unwrap();
    assert_eq!(decoded, reply);
    assert_eq!(
        decode_string_rows(decoded.split_whitespace().nth(4).unwrap()).unwrap(),
        rows
    );
    assert!(transport::read_message(
        &mut Cursor::new(&wire),
        transport::Direction::Reply,
        Some(43),
        TRANSPORT_LIMITS
    )
    .is_err());
    let first_end = wire.iter().position(|byte| *byte == b'\n').unwrap() + 1;
    assert!(transport::read_message(
        &mut Cursor::new(&wire[..first_end]),
        transport::Direction::Reply,
        Some(42),
        TRANSPORT_LIMITS
    )
    .is_err());
}

#[test]
fn worker_loop_accepts_chunked_requests_without_desynchronizing() {
    let request = format!(
        "call 9 {} s:{}",
        STANDARD.encode("polars.show_versions"),
        STANDARD.encode("x".repeat(2 * TRANSPORT_LIMITS.frame))
    );
    let mut wire = Vec::new();
    transport::write_message(
        &mut wire,
        &request,
        9,
        transport::Direction::Call,
        TRANSPORT_LIMITS,
    )
    .unwrap();
    writeln!(wire, "call 10 {}", STANDARD.encode("polars.show_versions")).unwrap();
    let mut output = Vec::new();
    assert_eq!(run_loop(Cursor::new(wire), &mut output), ExitCode::SUCCESS);
    let output = String::from_utf8(output).unwrap();
    assert!(output.lines().next().unwrap().starts_with("reply 9 1 err "));
    assert!(output
        .lines()
        .nth(1)
        .unwrap()
        .starts_with("reply 10 1 ok_string "));
}

#[cfg(feature = "real-polars")]
#[test]
fn cursor_handles_reject_alias_disposal_foreign_owners_and_wrong_types() {
    let mut worker = Worker::default();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test/fixtures/identified_dates.csv");
    let reply = worker.csv_cursor_open(vec![
        NativeArg::Text(path.to_string_lossy().into_owned()),
        NativeArg::Int(1),
    ]);
    assert!(reply.starts_with("result_ok_handle "), "{reply}");
    let fields: Vec<_> = reply.split_whitespace().collect();
    let handle = NativeHandle {
        owner: decode_text(fields[1]).unwrap(),
        id: fields[2].parse().unwrap(),
        generation: 1,
        type_name: decode_text(fields[4]).unwrap(),
    };
    assert!(Worker::default()
        .csv_cursor_next(vec![NativeArg::Handle(handle.clone())])
        .starts_with("err "));
    let mut wrong = handle.clone();
    wrong.type_name = DATAFRAME_TYPE.into();
    assert!(worker
        .csv_cursor_next(vec![NativeArg::Handle(wrong)])
        .starts_with("err "));
    assert!(worker
        .csv_cursor_next(vec![NativeArg::Handle(handle.clone())])
        .starts_with("result_ok_handle "));
    assert_eq!(worker.frames.len(), 1);
    assert_eq!(
        worker.csv_cursor_dispose(vec![NativeArg::Handle(handle.clone())]),
        "ok_unit"
    );
    assert_eq!(
        worker.frames.len(),
        1,
        "cursor disposal must not dispose returned frames"
    );
    assert!(worker
        .csv_cursor_next(vec![NativeArg::Handle(handle.clone())])
        .starts_with("err "));
    assert!(worker
        .csv_cursor_dispose(vec![NativeArg::Handle(handle)])
        .starts_with("err "));
}
