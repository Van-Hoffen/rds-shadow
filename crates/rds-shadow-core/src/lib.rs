//! Shared domain types for RDS Shadow.
//!
//! This crate is intentionally platform-neutral so CI can test it on Linux.
//! Windows-only calls live in the `rds-shadow` binary.

pub mod command;
pub mod config;
pub mod event;
pub mod filter;
pub mod writer;

pub use command::{mstsc_args, mstsc_shadow_args, rdp_file};
pub use config::{Config, ConfigError, ConnectionMode};
pub use event::{AuditEvent, EventKind, ResultKind};
pub use filter::EventFilter;
pub use writer::JsonlWriter;
