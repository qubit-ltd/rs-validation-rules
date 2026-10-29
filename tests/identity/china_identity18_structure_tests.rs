// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::Validator;

#[cfg(feature = "china-identity")]
#[test]
fn test_china_identity_structure_accepts_zero_region_code() {
    use qubit_validation_rules::identity::ChinaIdentity18Structure;

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
    use qubit_validation_rules::identity::ChinaIdentity18Structure;
    use qubit_validation_rules::identity::ChinaIdentityError;

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
