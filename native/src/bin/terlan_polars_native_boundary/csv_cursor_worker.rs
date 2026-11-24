//! Ownership boundary for identified incremental CSV cursors.
use super::*;
use terlan_polars_native::csv_cursor::IdentifiedCsvCursor;

const CURSOR_TYPE: &str = "polars.DataFrame.CsvBatchCursor";

impl Worker {
    pub(super) fn csv_cursor_open(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(path), NativeArg::Int(rows)] = args.as_slice() else {
            return protocol_error("native_bad_args", "CSV cursor expects path and batch rows");
        };
        let rows = usize::try_from(*rows).unwrap_or(0);
        match IdentifiedCsvCursor::open(path, rows) {
            Err(error) => adapter_result_error(error.code(), error.message()),
            Ok(cursor) => {
                self.next_id = self.next_id.saturating_add(1).max(1);
                let id = self.next_id;
                self.csv_cursors.insert(id, cursor);
                format!(
                    "result_ok_handle {} {id} 1 {}",
                    STANDARD.encode(&self.owner),
                    STANDARD.encode(CURSOR_TYPE)
                )
            }
        }
    }

    fn csv_cursor_id(&self, args: &[NativeArg]) -> Result<u64, String> {
        let [NativeArg::Handle(handle)] = args else {
            return Err(protocol_error(
                "native_bad_args",
                "expected a CSV cursor handle",
            ));
        };
        if handle.owner != self.owner
            || handle.type_name != CURSOR_TYPE
            || handle.generation != 1
            || !self.csv_cursors.contains_key(&handle.id)
        {
            return Err(protocol_error(
                "stale_native_handle",
                "CSV cursor is not live in this helper",
            ));
        }
        Ok(handle.id)
    }

    pub(super) fn csv_cursor_next(&mut self, args: Vec<NativeArg>) -> String {
        let id = match self.csv_cursor_id(&args) {
            Ok(id) => id,
            Err(error) => return error,
        };
        match self.csv_cursors.get_mut(&id).unwrap().next_batch() {
            Ok(Some(frame)) => self.store_result_dataframe(frame),
            Ok(None) => adapter_result_error("csv_end_of_stream", "CSV cursor is exhausted"),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn csv_cursor_sha256(&self, args: Vec<NativeArg>) -> String {
        let id = match self.csv_cursor_id(&args) {
            Ok(id) => id,
            Err(error) => return error,
        };
        format!(
            "ok_string {}",
            STANDARD.encode(&self.csv_cursors[&id].sha256)
        )
    }

    pub(super) fn csv_cursor_dispose(&mut self, args: Vec<NativeArg>) -> String {
        let id = match self.csv_cursor_id(&args) {
            Ok(id) => id,
            Err(error) => return error,
        };
        self.csv_cursors.remove(&id);
        "ok_unit".to_owned()
    }
}
