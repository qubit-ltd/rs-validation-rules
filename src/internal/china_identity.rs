// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structural parsing for 18-character mainland China identity numbers.

mod china_identity18_structure;
mod china_identity_error;
mod china_identity_facts;

pub use china_identity_error::ChinaIdentityError;
pub use china_identity_facts::ChinaIdentityFacts;
pub use china_identity18_structure::ChinaIdentity18Structure;
