// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::registrations;
use qubit_validation_rules::text::ChinaMobileStructure;
use qubit_validation_rules::text::EmailAscii;
use qubit_validation_rules::text::TextRuleError;
use qubit_validation_rules::text::Uri;
use qubit_validation_rules::text::UuidText;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;

#[test]
fn test_uri_accepts_absolute_rfc3986_examples() {
    for value in [
        "mailto:user@example.com",
        "a:b",
        "https://example.com",
        "urn:isbn:0451450523",
    ] {
        assert_eq!(Uri.validate(value, &()), Ok(()), "{value}");
    }
}

#[test]
fn test_uri_rejects_invalid_syntax_and_relative_references() {
    for value in [
        "https://example.com/%GG",
        "a:b\0",
        "https://例子.example/",
        "relative/path",
    ] {
        assert_eq!(Uri.validate(value, &()), Err(TextRuleError::Uri), "{value:?}");
    }
}

#[test]
fn test_registered_uri_rejects_invalid_percent_escape() {
    let registry = ValidatorRegistry::from_registrations(registrations()).expect("standard registrations are valid");
    let bound = registry
        .bind("qubit.rules.text.uri", InputType::Text, &[])
        .expect("URI rule binds without arguments");
    let outcome = bound
        .validate(
            ValidationValue::Text("https://example.com/%GG"),
            &BoundValidationContext::new(&[]),
        )
        .expect("URI rule executes");
    let ValidationOutcome::Invalid(violations) = outcome else {
        panic!("invalid percent escape must produce a violation: {outcome:?}");
    };
    assert_eq!(violations[0].code().as_str(), "text.uri");
}

#[test]
fn test_strict_text_profiles_accept_valid_and_reject_invalid_inputs() {
    assert_eq!(EmailAscii.validate("a.b+tag@example-domain.com", &()), Ok(()));
    for invalid in [
        "missing-at",
        "a@b@c",
        "@example.com",
        "a@",
        "a..b@example.com",
        ".a@example.com",
        "a.@example.com",
        "a b@example.com",
        "a@-example.com",
        "a@example-.com",
        "a@example..com",
        "a@例子.测试",
    ] {
        assert_eq!(
            EmailAscii.validate(invalid, &()),
            Err(TextRuleError::Email),
            "{invalid}"
        );
    }

    assert_eq!(Uri.validate("https://example.com/path", &()), Ok(()));
    assert_eq!(Uri.validate("a:", &()), Ok(()));
    for invalid in ["", "relative", "1bad:rest", "a:b c"] {
        assert_eq!(Uri.validate(invalid, &()), Err(TextRuleError::Uri), "{invalid}");
    }

    assert_eq!(UuidText.validate("550e8400-e29b-41d4-a716-446655440000", &()), Ok(()));
    assert_eq!(UuidText.validate("not-a-uuid", &()), Err(TextRuleError::Uuid));

    assert_eq!(ChinaMobileStructure.validate("13800138000", &()), Ok(()));
    for invalid in ["23800138000", "12800138000", "1380013800", "1380013800x"] {
        assert_eq!(
            ChinaMobileStructure.validate(invalid, &()),
            Err(TextRuleError::Mobile),
            "{invalid}"
        );
    }
}

#[test]
fn test_email_ascii_profile_boundaries() {
    for text in ["a@b", "first.last+tag@example.com", "a@xn--fiqs8s"] {
        assert_eq!(EmailAscii.validate(text, &()), Ok(()));
    }
    for text in [
        "\"a\"@example.com",
        "a..b@example.com",
        ".a@example.com",
        "a.@example.com",
        "a@-b",
        "a@b-",
        "a@b.",
        "a@b_c",
        "a@[127.0.0.1]",
        "a@例子",
    ] {
        assert_eq!(EmailAscii.validate(text, &()), Err(TextRuleError::Email));
    }
    assert!(EmailAscii.validate(&format!("{}@b", "a".repeat(64)), &()).is_ok());
    assert!(EmailAscii.validate(&format!("{}@b", "a".repeat(65)), &()).is_err());
    assert!(EmailAscii.validate(&format!("a@{}", "b".repeat(63)), &()).is_ok());
    assert!(EmailAscii.validate(&format!("a@{}", "b".repeat(64)), &()).is_err());
    let domain = format!("{}.{}.{}", "b".repeat(63), "b".repeat(63), "b".repeat(61));
    let limit = format!("{}@{}", "a".repeat(64), domain);
    assert_eq!(limit.len(), 254);
    assert!(EmailAscii.validate(&limit, &()).is_ok());
    assert!(EmailAscii.validate(&(limit + "b"), &()).is_err());
}
