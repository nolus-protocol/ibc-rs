//! ICS-24: Host defines the minimal set of interfaces that a state machine
//! hosting an IBC-enabled chain must implement.
#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types,))]
#![cfg_attr(not(feature = "parity-scale-codec"), deny(trivial_numeric_casts))]
#![deny(
    warnings,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod error;
pub mod identifiers;
pub mod path;
pub(crate) mod validate;
