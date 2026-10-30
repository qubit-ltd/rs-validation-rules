// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::str::from_utf8;

use chrono::NaiveDate;
use qubit_validator::Validator;

use super::super::china_identity_error::ChinaIdentityError;
use super::super::china_identity_facts::ChinaIdentityFacts;

/// Validates the structure of an 18-character mainland China identity number.
///
/// This rule checks the length, ASCII digits in the body, final checksum
/// character, calendar birth date, and checksum. It does not check whether the
/// region code is assigned, the number was issued, or the holder's identity.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::identity::ChinaIdentity18Structure;
///
/// let facts = ChinaIdentity18Structure::parse("11010519491231002X")
///     .expect("valid structural example");
/// assert_eq!(facts.birth_date().to_string(), "1949-12-31");
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChinaIdentity18Structure;

impl ChinaIdentity18Structure {
    /// Parses `value` and returns its encoded birth date when the structure is
    /// valid.
    ///
    /// Returns [`ChinaIdentityError`] for a wrong length, non-ASCII or
    /// non-digit body character, invalid calendar date, invalid final
    /// character, or incorrect checksum. A successful result does not prove
    /// issuance or a valid region code.
    ///
    /// # Parameters
    /// - `value`: Candidate 18-byte identity number.
    ///
    /// # Returns
    /// Parsed facts containing the encoded birth date.
    ///
    /// # Errors
    /// Returns the first structural failure detected in the input.
    pub fn parse(value: &str) -> Result<ChinaIdentityFacts, ChinaIdentityError> {
        let b = value.as_bytes();
        if b.len() != 18 {
            return Err(ChinaIdentityError::InvalidLength);
        }
        for (i, digit) in b.iter().enumerate().take(17) {
            if !digit.is_ascii() {
                return Err(ChinaIdentityError::NonAsciiBodyDigit { position: i as u8 });
            }
            if !digit.is_ascii_digit() {
                return Err(ChinaIdentityError::InvalidBodyDigit { position: i as u8 });
            }
        }
        if !(b[17].is_ascii_digit() || matches!(b[17], b'X' | b'x')) {
            return Err(ChinaIdentityError::InvalidChecksumCharacter);
        }
        let date = from_utf8(&b[6..14])
            .ok()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok())
            .ok_or(ChinaIdentityError::InvalidBirthDate)?;
        let sum = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2]
            .iter()
            .zip(&b[..17])
            .map(|(w, c)| (*w as u32) * u32::from(*c - b'0'))
            .sum::<u32>();
        let expected = b"10X98765432"[(sum % 11) as usize];
        let actual = b[17].to_ascii_uppercase();
        if actual != expected {
            return Err(ChinaIdentityError::InvalidChecksum);
        }
        Ok(ChinaIdentityFacts::from_birth_date(date))
    }
}

impl Validator<str, ()> for ChinaIdentity18Structure {
    /// Structural failure emitted for an invalid candidate.
    type Error = ChinaIdentityError;

    /// Accepts values that pass all identity-number structure checks.
    ///
    /// # Parameters
    /// - `value`: Candidate identity number.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when the candidate passes every structural check.
    ///
    /// # Errors
    /// Returns the structural failure from [`Self::parse`].
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        Self::parse(value).map(|_| ())
    }
}
