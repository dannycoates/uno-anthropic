//! uno-anthropic
//!
//! An unofficial, idiomatic Rust SDK for the Anthropic API covering the Messages API
//! (create, streaming, count_tokens), Batch API, Models API, Beta features,
//! and Bedrock/Vertex AI integrations.
//!
//! # Quick Start
//!
//! ```ignore
//! use uno_anthropic::{Client, Model, MessageCreateParams, MessageParam};
//!
//! let client = Client::new(); // reads ANTHROPIC_API_KEY from env
//! let message = client.messages().create(
//!     MessageCreateParams::builder()
//!         .model(Model::ClaudeOpus4_6)
//!         .max_tokens(1024)
//!         .messages(vec![MessageParam::user("Hello, Claude!")])
//!         .build()
//! ).await?;
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod middleware;
pub mod retry;
pub mod types;

pub mod messages;
pub mod streaming;

pub mod batches;
pub mod models;

pub mod beta;

#[cfg(feature = "bedrock")]
pub mod bedrock;

#[cfg(feature = "vertex")]
pub mod vertex;

// Re-export key types at crate root for ergonomic imports.
pub use client::Client;
pub use error::Error;
pub use messages::params::{CountTokensParams, MessageCreateParams};
pub use types::*;
