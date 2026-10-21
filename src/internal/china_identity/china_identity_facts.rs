// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use chrono::NaiveDate;

/// Facts extracted from a structurally valid identity number.
///
/// These facts do not establish that the identity number was issued or belongs
/// to a particular person.
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
pub struct ChinaIdentityFacts {
    /// Calendar date parsed from the identity number's birth-date digits.
    birth_date: NaiveDate,
}

impl ChinaIdentityFacts {
    /// Creates facts from a date validated by the identity parser.
    #[must_use]
    #[inline]
    pub(super) const fn from_birth_date(birth_date: NaiveDate) -> Self {
        Self { birth_date }
    }

    /// Returns the calendar date encoded by the eight birth-date digits.
    ///
    /// # Returns
    /// The encoded date; it does not attest to issuance or identity.
    #[must_use]
    #[inline]
    pub const fn birth_date(self) -> NaiveDate {
        self.birth_date
    }
}
