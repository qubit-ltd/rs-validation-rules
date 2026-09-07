// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(missing_docs)]

pub use TextRuleError as CharLengthError;
pub use TextRuleError as EmailAsciiError;
pub use TextRuleError as UriError;
pub use TextRuleError as UuidTextError;
pub use TextRuleError as ChinaMobileStructureError;

pub use crate::internal::legacy::AllowedChars;
pub use crate::internal::legacy::AllowedCharsError;
pub use crate::internal::legacy::ByteLength;
pub use crate::internal::legacy::ByteLengthError;
pub use crate::internal::legacy::CharLength;
pub use crate::internal::legacy::CharacterSet;
pub use crate::internal::legacy::ChinaMobileStructure;
pub use crate::internal::legacy::EmailAscii;
pub use crate::internal::legacy::NonBlank;
pub use crate::internal::legacy::NonBlankError;
pub use crate::internal::legacy::TextLengthError;
pub use crate::internal::legacy::TextRuleError;
pub use crate::internal::legacy::Uri;
pub use crate::internal::legacy::UuidText;

pub mod non_blank {
    pub use super::NonBlank;
    pub use super::NonBlankError;
}
pub mod char_length {
    pub use super::CharLength;
    pub use super::CharLengthError;
}
pub mod byte_length {
    pub use super::ByteLength;
    pub use super::ByteLengthError;
}
pub mod character_set {
    pub use super::AllowedChars;
    pub use super::AllowedCharsError;
    pub use super::CharacterSet;
}
pub mod allowed_chars {
    pub use super::AllowedChars;
    pub use super::AllowedCharsError;
    pub use super::CharacterSet;
}
pub mod email_ascii {
    pub use super::EmailAscii;
    pub use super::EmailAsciiError;
}
pub mod uri {
    pub use super::Uri;
    pub use super::UriError;
}
pub mod uuid_text {
    pub use super::UuidText;
    pub use super::UuidTextError;
}
pub mod china_mobile_structure {
    pub use super::ChinaMobileStructure;
    pub use super::ChinaMobileStructureError;
}
