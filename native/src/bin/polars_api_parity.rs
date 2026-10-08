//! Reports the committed upstream-to-Terlan Polars API inventory mapping.

use std::process::ExitCode;

use terlan_polars_native::parity::{
    data_type_api_parity_report, dataframe_api_parity_report, expression_api_parity_report,
    io_api_parity_report, lazyframe_api_parity_report, series_api_parity_report,
    sql_api_parity_report, ApiParityReport,
};

/// Prints one family report and returns whether it has pending identities.
fn print_report(name: &str, report: &ApiParityReport) -> bool {
    println!("{name}");
    println!("  upstream: {}", report.upstream_count);
    println!("  mapped: {}", report.mapped_count);
    println!("  exact native-name matches: {}", report.exact_count);
    println!("  explicit mappings: {}", report.explicit_mapping_count);
    println!("  pending: {}", report.pending.len());
    for method in &report.pending {
        println!("    {method}");
    }
    !report.pending.is_empty()
}

/// Prints inventory reports and optionally requires zero unmapped identities.
fn main() -> ExitCode {
    let require_complete = std::env::args().any(|argument| argument == "--require-complete");
    let reports = [
        ("DataFrame", dataframe_api_parity_report()),
        ("Series", series_api_parity_report()),
        ("LazyFrame", lazyframe_api_parity_report()),
        ("Expr", expression_api_parity_report()),
        ("DataType", data_type_api_parity_report()),
        ("I/O", io_api_parity_report()),
        ("SQL", sql_api_parity_report()),
    ];
    println!("Published Rust Polars 0.55.2 inventory baseline coverage");
    let mut pending = false;
    for (name, report) in reports {
        let report = match report {
            Ok(report) => report,
            Err(error) => {
                eprintln!("polars API inventory error: {error}");
                return ExitCode::FAILURE;
            }
        };
        pending |= print_report(name, &report);
    }

    if require_complete && pending {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
