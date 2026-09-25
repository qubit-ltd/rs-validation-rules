// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Dynamic registration adapters for typed rules.

mod collection;
#[cfg(feature = "regex")]
mod regex;
mod registrations;
mod text;

pub use registrations::registrations;
