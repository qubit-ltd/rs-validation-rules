use chrono::NaiveDate;
use qubit_validator::Validator;

/// Reasons an 18-character identity number fails structural validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
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

/// Facts extracted from a structurally valid identity number.
///
/// These facts do not establish that the identity number was issued or belongs
/// to a particular person.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChinaIdentityFacts {
    birth_date: NaiveDate,
}
impl ChinaIdentityFacts {
    /// Returns the calendar date encoded by the eight birth-date digits.
    pub const fn birth_date(self) -> NaiveDate {
        self.birth_date
    }
}

/// Validates the structure of an 18-character mainland China identity number.
///
/// This rule checks the length, ASCII digits in the body, final checksum
/// character, calendar birth date, and checksum. It does not check whether the
/// region code is assigned, the number was issued, or the holder's identity.
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
    pub fn parse(value: &str) -> Result<ChinaIdentityFacts, ChinaIdentityError> {
        let b = value.as_bytes();
        if b.len() != 18 {
            return Err(ChinaIdentityError::InvalidLength);
        };
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
        let date = std::str::from_utf8(&b[6..14])
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
        Ok(ChinaIdentityFacts { birth_date: date })
    }
}

impl Validator<str, ()> for ChinaIdentity18Structure {
    type Error = ChinaIdentityError;

    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        Self::parse(value).map(|_| ())
    }
}
