#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    eprintln!("rds-shadow: this binary is Windows-only (mstsc /shadow and WTS API)");
    std::process::exit(2);
}

#[cfg(windows)]
fn main() {
    if let Err(e) = rds_shadow::app::run() {
        rds_shadow::platform::fatal(&e.to_string());
        std::process::exit(1);
    }
}
