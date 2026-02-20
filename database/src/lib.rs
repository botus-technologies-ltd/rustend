//! Database crate
//!
//! Provides database utilities including flexible ID types for MongoDB, PostgreSQL, MySQL, and SQLite.

pub mod utils;
pub mod init;

#[cfg(feature = "mongodb")]
pub mod mongo;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "sqlite")]
pub mod sqlite;
