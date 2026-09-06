use chrono::NaiveDate;
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ChinaIdentityError {
    #[error("invalid length")]
    InvalidLength,
    #[error("invalid body digit")]
    InvalidBodyDigit { position: u8 },
    #[error("non-ascii body digit")]
    NonAsciiBodyDigit { position: u8 },
    #[error("invalid birth date")]
    InvalidBirthDate,
    #[error("invalid checksum")]
    InvalidChecksum,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChinaIdentityFacts {
    birth_date: NaiveDate,
}
impl ChinaIdentityFacts {
    pub const fn birth_date(self) -> NaiveDate {
        self.birth_date
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChinaIdentity18;
impl ChinaIdentity18 {
    pub fn parse(value: &str) -> Result<ChinaIdentityFacts, ChinaIdentityError> {
        let b = value.as_bytes();
        if b.len() != 18 {
            return Err(ChinaIdentityError::InvalidLength);
        };
        for i in 0..17 {
            if !b[i].is_ascii() {
                return Err(ChinaIdentityError::NonAsciiBodyDigit { position: i as u8 });
            }
            if !b[i].is_ascii_digit() {
                return Err(ChinaIdentityError::InvalidBodyDigit { position: i as u8 });
            }
        }
        let date = std::str::from_utf8(&b[6..14])
            .ok()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok())
            .ok_or(ChinaIdentityError::InvalidBirthDate)?;
        let sum = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2]
            .iter()
            .zip(&b[..17])
            .map(|(w, c)| u32::from(*w) * u32::from(c - b'0'))
            .sum::<u32>();
        let expected = b"10X98765432"[(sum % 11) as usize];
        let actual = b[17].to_ascii_uppercase();
        if actual != expected {
            return Err(ChinaIdentityError::InvalidChecksum);
        }
        Ok(ChinaIdentityFacts { birth_date: date })
    }
}
