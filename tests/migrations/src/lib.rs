//! Storage layout migration tests.
//!
//! This crate contains no production code. Its integration tests in
//! `tests/storage_layout.rs` drive `get_storage_layout`,
//! `begin_layout_migration`, and `complete_layout_migration` on all three
//! contracts, including legacy deployments that predate the layout key. The
//! rules under test are documented in `docs/storage-layout-versions.md`.

#![no_std]
