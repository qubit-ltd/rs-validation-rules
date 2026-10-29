// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[cfg(feature = "china-identity")]
use std::str::from_utf8;

#[cfg(feature = "china-identity")]
use qubit_validation_rules::identity::ChinaIdentity18Structure;
#[cfg(feature = "china-identity")]
use qubit_validation_rules::identity::ChinaIdentityError;
use qubit_validator::Validator;

#[cfg(feature = "china-identity")]
#[test]
fn test_china_identity_structure_accepts_zero_region_code() {
    // This satisfies the structural checks; it does not establish a valid issued
    // credential.
    let structurally_valid = "000000194912310027";
    let facts = ChinaIdentity18Structure::parse(structurally_valid)
        .expect("zero region code does not invalidate the structure");
    assert_eq!(facts.birth_date().to_string(), "1949-12-31");
    assert_eq!(ChinaIdentity18Structure.validate(structurally_valid, &()), Ok(()));
}
#[cfg(feature = "china-identity")]
#[test]
fn test_china_identity_parser_reports_each_validation_failure() {
    assert_eq!(
        ChinaIdentity18Structure::parse("short"),
        Err(ChinaIdentityError::InvalidLength)
    );
    assert_eq!(
        ChinaIdentity18Structure::parse("é010519491231002X"),
        Err(ChinaIdentityError::NonAsciiBodyDigit { position: 0 })
    );
    assert_eq!(
        ChinaIdentity18Structure::parse("1101051949123100A0"),
        Err(ChinaIdentityError::InvalidBodyDigit { position: 16 })
    );
    assert_eq!(
        ChinaIdentity18Structure::parse("11010519491231002!"),
        Err(ChinaIdentityError::InvalidChecksumCharacter)
    );
    assert_eq!(
        ChinaIdentity18Structure::parse("11010520230230002X"),
        Err(ChinaIdentityError::InvalidBirthDate)
    );
    assert_eq!(
        ChinaIdentity18Structure::parse("110105194912310020"),
        Err(ChinaIdentityError::InvalidChecksum)
    );

    let valid = "11010519491231002X";
    let facts = ChinaIdentity18Structure::parse(valid).expect("known structurally valid identity number");
    assert_eq!(facts.birth_date().to_string(), "1949-12-31");
    assert_eq!(ChinaIdentity18Structure.validate(valid, &()), Ok(()));
}

#[cfg(feature = "china-identity")]
#[test]
fn test_china_identity_checksum_remainder_fixtures() {
    // These fixed fixtures cover every checksum remainder without generating
    // expected check characters through the implementation under test.
    let cases = [
        "110105194912310003",
        "110105194912310011",
        "11010519491231002X",
        "110105194912310038",
        "110105194912310046",
        "110105194912310054",
        "110105194912310062",
        "110105194912310070",
        "110105194912310089",
        "110105194912310097",
        "110105194912310185",
    ];
    for value in cases {
        let facts = ChinaIdentity18Structure::parse(value).expect("independently checked structural fixture");
        assert_eq!(facts.birth_date().to_string(), "1949-12-31", "{value}");
        assert_eq!(ChinaIdentity18Structure.validate(value, &()), Ok(()), "{value}");
    }
    assert_eq!(
        ChinaIdentity18Structure::parse("11010519491231002x"),
        ChinaIdentity18Structure::parse("11010519491231002X"),
    );
    assert_eq!(ChinaIdentity18Structure.validate("11010519491231002x", &()), Ok(()));
}

#[cfg(feature = "china-identity")]
#[test]
fn test_china_identity_century_leap_day_boundary() {
    // Gregorian century rules accept 2000 as a leap year but reject 1900.
    let valid = "110105200002290021";
    let facts = ChinaIdentity18Structure::parse(valid).expect("year 2000 has a leap day");
    assert_eq!(facts.birth_date().to_string(), "2000-02-29");
    assert_eq!(ChinaIdentity18Structure.validate(valid, &()), Ok(()));
    let invalid = "110105190002290025";
    assert_eq!(
        ChinaIdentity18Structure::parse(invalid),
        Err(ChinaIdentityError::InvalidBirthDate)
    );
    assert_eq!(
        ChinaIdentity18Structure.validate(invalid, &()),
        Err(ChinaIdentityError::InvalidBirthDate)
    );
}

#[cfg(feature = "china-identity")]
#[test]
fn test_china_identity_rejects_each_single_digit_mutation() {
    // Every body position must affect a structural check; the final digit must
    // report the checksum error specifically.
    let original = b"11010519491231002X";
    for position in 0..17 {
        let mut candidate = *original;
        candidate[position] = b'0' + (candidate[position] - b'0' + 1) % 10;
        let text = from_utf8(&candidate).expect("ASCII mutation");
        assert!(ChinaIdentity18Structure::parse(text).is_err(), "position {position}");
        assert!(
            ChinaIdentity18Structure.validate(text, &()).is_err(),
            "position {position}"
        );
    }
    let invalid_checksum = "110105194912310020";
    assert_eq!(
        ChinaIdentity18Structure::parse(invalid_checksum),
        Err(ChinaIdentityError::InvalidChecksum)
    );
    assert_eq!(
        ChinaIdentity18Structure.validate(invalid_checksum, &()),
        Err(ChinaIdentityError::InvalidChecksum)
    );
}
