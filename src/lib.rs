pub mod app;
pub mod core;
pub mod io;
pub mod models;
pub mod parser;
pub mod ui;

// Re-export scan types for use by both main and io::terminal
pub use models::package::Package;

/// Message types for scan thread communication
pub enum ScanMessage {
    Progress(String),
    ScanWarning(String),
    Complete(Vec<Package>),
}

/// Scan failure markers for warning messages
pub const OFFICIAL_SCAN_FAILURE_MARKER: &str = "Official";
pub const AUR_SCAN_FAILURE_MARKER: &str = "AUR";
