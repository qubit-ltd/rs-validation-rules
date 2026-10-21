// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Text format validation rules.

#[path = "china_mobile_structure.rs"]
mod china_mobile_structure;
#[path = "email_ascii.rs"]
mod email_ascii;
#[path = "uri.rs"]
mod uri;
#[path = "uuid_text.rs"]
mod uuid_text;

pub use china_mobile_structure::ChinaMobileStructure;
pub use email_ascii::EmailAscii;
pub use uri::Uri;
pub use uuid_text::UuidText;
