// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Reasons an 18-character identity number fails structural validation.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::identity::ChinaIdentity18Structure;
/// use qubit_validation_rules::identity::ChinaIdentityError;
///
/// assert_eq!(ChinaIdentity18Structure::parse("short"), Err(ChinaIdentityError::InvalidLength));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
pub enum ChinaIdentityError {
    /// The input is not exactly 18 bytes long.
    #[error("invalid length")]
    InvalidLength,
    /// A body character at the zero-based byte position is ASCII but not a
    /// digit.
    #[error("invalid body digit")]
    InvalidBodyDigit {
        /// Zero-based byte position of the invalid character.
        position: u8,
    },
    /// A body character at the zero-based byte position is not ASCII.
    #[error("non-ascii body digit")]
    NonAsciiBodyDigit {
        /// Zero-based byte position of the invalid character.
        position: u8,
    },
    /// The eight birth-date digits do not encode a calendar date.
    #[error("invalid birth date")]
    InvalidBirthDate,
    /// The final character disagrees with the checksum computed from the body.
    #[error("invalid checksum")]
    InvalidChecksum,
    /// The final character is neither an ASCII digit nor `X` or `x`.
    #[error("invalid checksum character")]
    InvalidChecksumCharacter,
}
