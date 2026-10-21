// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Text validation rules.

mod allowed_chars;
mod allowed_chars_error;
mod character_set;
mod format;
mod length;
mod matches_dependency;
mod matches_dependency_error;
mod non_blank;
mod non_blank_error;
mod text_rule_error;

pub use allowed_chars::AllowedChars;
pub use allowed_chars_error::AllowedCharsError;
pub use character_set::CharacterSet;
pub use format::ChinaMobileStructure;
pub use format::EmailAscii;
pub use format::Uri;
pub use format::UuidText;
pub use length::ByteLength;
pub use length::ByteLengthError;
pub use length::CharLength;
pub use length::TextLengthError;
pub use matches_dependency::MatchesDependency;
pub use matches_dependency_error::MatchesDependencyError;
pub use non_blank::NonBlank;
pub use non_blank_error::NonBlankError;
pub use text_rule_error::TextRuleError;
