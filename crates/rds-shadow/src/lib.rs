//! RDS Shadow — Windows support client for shadowing RDS sessions.

pub mod command;
pub mod config;
pub mod event;
pub mod filter;
pub mod writer;

#[cfg(windows)]
pub mod app;
#[cfg(windows)]
pub mod audit;
#[cfg(windows)]
pub mod cli;
#[cfg(windows)]
pub mod platform;
