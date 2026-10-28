// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::ids;
use qubit_validation_rules::regex_rule::RegexMatch;
use qubit_validation_rules::registrations;
use qubit_validation_rules::text::TextRuleError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;
use regex::Regex;

#[test]
fn test_extended_comment_pattern_preserves_full_string_matching() {
    let pattern = "(?x)abc # comment";
    assert!(Regex::new(pattern).is_ok(), "raw dependency pattern must be valid");
    let rule = RegexMatch::new(pattern).expect("valid pattern body must bind");
    assert_eq!(rule.validate("abc", &()), Ok(()));
    for value in ["xabc", "abcx", "abc\n"] {
        assert_eq!(rule.validate(value, &()), Err(TextRuleError::Pattern));
    }

    let registry = ValidatorRegistry::from_registrations(registrations()).unwrap();
    let arguments = [NamedValidationArgument::new(
        "pattern",
        ValidationArgument::String(pattern),
    )];
    let bound = registry.bind(ids::TEXT_REGEX, InputType::Text, &arguments).unwrap();
    let context = BoundValidationContext::new(&[]);
    assert_eq!(
        bound.validate(ValidationValue::Text("abc"), &context).unwrap(),
        ValidationOutcome::Valid
    );
    for value in ["xabc", "abcx", "abc\n"] {
        assert!(matches!(
            bound.validate(ValidationValue::Text(value), &context).unwrap(),
            ValidationOutcome::Invalid(_),
        ));
    }
}

#[test]
fn test_ordinary_patterns_and_invalid_bodies_keep_their_meaning() {
    let registry = ValidatorRegistry::from_registrations(registrations()).unwrap();
    for (pattern, accepted, rejected) in [
        ("abc", "abc", "abc\n"),
        ("a b", "a b", "ab"),
        ("a\nb", "a\nb", "ab"),
        ("(?x: a )#", "a#", "a"),
        ("(?x)abc(?-x)#", "abc#", "abc"),
        ("a|bc", "bc", "xbc"),
        ("^abc$", "abc", "abc\n"),
        ("(?x)# comment", "", "x"),
    ] {
        let rule = RegexMatch::new(pattern).unwrap_or_else(|error| panic!("valid pattern {pattern:?}: {error:?}"));
        assert_eq!(rule.validate(accepted, &()), Ok(()), "{pattern:?}");
        assert_eq!(rule.validate(rejected, &()), Err(TextRuleError::Pattern), "{pattern:?}");

        let arguments = [NamedValidationArgument::new(
            "pattern",
            ValidationArgument::String(pattern),
        )];
        let bound = registry.bind(ids::TEXT_REGEX, InputType::Text, &arguments).unwrap();
        let context = BoundValidationContext::new(&[]);
        assert_eq!(
            bound.validate(ValidationValue::Text(accepted), &context).unwrap(),
            ValidationOutcome::Valid,
            "{pattern:?}",
        );
        assert!(
            matches!(
                bound.validate(ValidationValue::Text(rejected), &context).unwrap(),
                ValidationOutcome::Invalid(_),
            ),
            "{pattern:?}",
        );
    }
    for pattern in ["[", "abc\\"] {
        assert!(matches!(RegexMatch::new(pattern), Err(error) if error.kind() == BindErrorKind::InvalidPattern));
        let arguments = [NamedValidationArgument::new(
            "pattern",
            ValidationArgument::String(pattern),
        )];
        assert!(matches!(
            registry.bind(ids::TEXT_REGEX, InputType::Text, &arguments),
            Err(error) if error.kind() == BindErrorKind::InvalidPattern,
        ));
    }

    let secret_pattern = "sensitive_invalid_pattern[";
    let error = RegexMatch::new(secret_pattern).err().expect("invalid pattern");
    assert!(!format!("{error:?}").contains(secret_pattern));
    let arguments = [NamedValidationArgument::new(
        "pattern",
        ValidationArgument::String(secret_pattern),
    )];
    let error = registry
        .bind(ids::TEXT_REGEX, InputType::Text, &arguments)
        .err()
        .expect("invalid pattern");
    assert!(!format!("{error:?}").contains(secret_pattern));
}
