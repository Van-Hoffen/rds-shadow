#[cfg(not(windows))]
fn main() {
    eprintln!("rds-shadow-cli: this binary is Windows-only (mstsc /shadow and WTS API)");
    std::process::exit(2);
}

#[cfg(windows)]
fn main() {
    if let Err(e) = rds_shadow::cli::run() {
        eprintln!("rds-shadow-cli: {e}");
        std::process::exit(1);
    }
}
