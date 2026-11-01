//! Pure, storage-independent domain contracts for the Flourish kernel.
//!
//! This crate deliberately contains no persistence, network, file-system, or
//! desktop-shell code. Rust owns the domain rules exposed here; presentation
//! layers consume projections of these types rather than reimplementing them.

#![forbid(unsafe_code)]

pub mod authorization;
pub mod capture;
pub mod composer;
pub mod entities;
pub mod evidence;
pub mod identity;
pub mod method_execution;
pub mod method_outcome;
pub mod method_runtime;
pub mod ownership;
pub mod personal_context;
pub mod predicate;
pub mod product_v2;
pub mod protocol;
pub mod registry;
pub mod retired_policy;
pub mod sync;
pub mod value;

pub use authorization::*;
pub use capture::*;
pub use composer::*;
pub use entities::*;
pub use evidence::*;
pub use identity::*;
pub use ownership::*;
pub use personal_context::*;
pub use predicate::*;
pub use product_v2::*;
pub use protocol::*;
pub use registry::*;
pub use retired_policy::*;
pub use sync::*;
pub use value::*;
