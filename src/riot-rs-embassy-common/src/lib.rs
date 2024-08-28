//! Architecture-agnostic types shared between architectures.

#![no_std]
#![feature(doc_auto_cfg)]
#![deny(missing_docs)]

pub mod gpio;

#[cfg(context = "cortex-m")]
pub mod executor_swi;
