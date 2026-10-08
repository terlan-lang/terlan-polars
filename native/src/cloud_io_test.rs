//! Cloud-path feature wiring tests.

#[cfg(feature = "real-polars")]
use crate::lazy_scan_csv;

#[cfg(feature = "real-polars")]
#[test]
fn cloud_uris_build_deferred_csv_scan_plans() {
    for uri in [
        "https://example.invalid/data.csv",
        "s3://terlan-polars-test/data.csv",
        "gs://terlan-polars-test/data.csv",
        "az://terlan-polars-test/data.csv",
    ] {
        assert!(
            lazy_scan_csv(uri, false).is_ok(),
            "cloud URI should build a deferred plan: {uri}"
        );
    }
}
