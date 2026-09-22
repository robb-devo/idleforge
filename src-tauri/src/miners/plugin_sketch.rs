//! Plugin sketch: how to add another GPU/CPU miner later.
//!
//! ```ignore
//! use crate::miners::adapter::{MinerAdapter, MinerKind, MinerStats, StartRequest};
//!
//! pub struct ExampleAdapter { /* child process, api port, … */ }
//!
//! impl MinerAdapter for ExampleAdapter {
//!     fn id(&self) -> &'static str { "example" }
//!     fn kind(&self) -> MinerKind { MinerKind::Gpu }
//!     fn start(&mut self, req: &StartRequest) -> Result<(), String> { /* spawn */ Ok(()) }
//!     fn stop(&mut self) -> Result<(), String> { /* kill */ Ok(()) }
//!     fn is_running(&self) -> bool { false }
//!     fn poll_stats(&mut self) -> Result<MinerStats, String> { /* HTTP/API */ todo!() }
//!     fn last_error(&self) -> Option<String> { None }
//! }
//! ```
//!
//! Wire it in `state.rs` based on `config.gpu.adapter` / `config.cpu.adapter`.
//! Never embed mining algorithms — only orchestrate external binaries.
