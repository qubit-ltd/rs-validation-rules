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

#[cfg(feature = "regex")]
fn create_test_registry() -> ValidatorRegistry {
    ValidatorRegistry::from_registrations(registrations()).expect("standard registrations are valid")
}

#[cfg(feature = "regex")]
fn violation_code(outcome: ValidationOutcome) -> String {
    let ValidationOutcome::Invalid(violations) = outcome else {
        panic!("expected an invalid outcome, got {outcome:?}");
    };
    violations[0].code().as_str().to_owned()
}

#[cfg(feature = "regex")]
fn validate_text_rule(
    registry: &ValidatorRegistry,
    id: &str,
    arguments: &[NamedValidationArgument<'_>],
    value: &str,
) -> ValidationOutcome {
    registry
        .bind(id, InputType::Text, arguments)
        .expect("text rule should bind")
        .validate(ValidationValue::Text(value), &BoundValidationContext::new(&[]))
        .expect("text rule should execute")
}

#[test]
fn test_invalid_pattern_bodies_cannot_escape_full_string_wrapper() {
    let registry = ValidatorRegistry::from_registrations(registrations()).unwrap();
    for pattern in [")a|b(?:", ")a|(?x)#"] {
        assert!(Regex::new(pattern).is_err());
        let error = RegexMatch::new(pattern).err().expect("invalid body must fail");
        assert_eq!(error.kind(), BindErrorKind::InvalidPattern);
        assert!(!format!("{error:?}").contains(pattern));

        let arguments = [NamedValidationArgument::new(
            "pattern",
            ValidationArgument::String(pattern),
        )];
        let error = registry
            .bind(ids::TEXT_REGEX, InputType::Text, &arguments)
            .expect_err("invalid body must not bind");
        assert_eq!(error.kind(), BindErrorKind::InvalidPattern);
        assert!(!format!("{error:?}").contains(pattern));
    }
}

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
fn test_regex_body_nesting_budget_survives_full_match_wrapper() {
    let registry = ValidatorRegistry::from_registrations(registrations()).unwrap();
    for depth in [248, 249, 250] {
        let body = format!("{}a{}", "(?:".repeat(depth), ")".repeat(depth));
        assert!(Regex::new(&body).is_ok());
        let patterns = if depth < 250 {
            vec![body.clone(), format!("(?x){body} # trailing comment")]
        } else {
            vec![body.clone()]
        };
        for pattern in patterns {
            assert!(Regex::new(&pattern).is_ok(), "raw body should fit at depth {depth}");
            let typed = RegexMatch::new(&pattern).expect("body fits nesting budget");
            let args = [NamedValidationArgument::new(
                "pattern",
                ValidationArgument::String(&pattern),
            )];
            let bound = registry.bind(ids::TEXT_REGEX, InputType::Text, &args).unwrap();
            for (text, accepted) in [("a", true), ("xa", false), ("ax", false), ("a\n", false)] {
                assert_eq!(typed.validate(text, &()).is_ok(), accepted, "depth={depth}");
                let outcome = bound
                    .validate(ValidationValue::Text(text), &BoundValidationContext::new(&[]))
                    .unwrap();
                assert_eq!(matches!(outcome, ValidationOutcome::Valid), accepted);
            }
        }
    }
    let too_deep = format!("{}a{}", "(?:".repeat(251), ")".repeat(251));
    let error = RegexMatch::new(&too_deep).err().expect("body exceeds nesting budget");
    assert_eq!(error.kind(), BindErrorKind::InvalidPattern);
    assert!(!format!("{error:?} {error}").contains(&too_deep));
    let args = [NamedValidationArgument::new(
        "pattern",
        ValidationArgument::String(&too_deep),
    )];
    let error = registry.bind(ids::TEXT_REGEX, InputType::Text, &args).unwrap_err();
    assert_eq!(error.kind(), BindErrorKind::InvalidPattern);
    assert!(!format!("{error:?} {error}").contains(&too_deep));
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
        .expect_err("invalid pattern");
    assert!(!format!("{error:?}").contains(secret_pattern));
}

#[cfg(feature = "regex")]
#[test]
fn test_regex_rule_compiles_and_matches_full_input() {
    let rule = RegexMatch::new(r"[a-z]+@[a-z]+\.com").expect("valid regex");
    assert_eq!(rule.validate("user@example.com", &()), Ok(()));
    assert_eq!(
        rule.validate("prefix user@example.com", &()),
        Err(TextRuleError::Pattern)
    );
    assert!(matches!(RegexMatch::new("["), Err(error) if error.kind() == BindErrorKind::InvalidPattern));

    let registry = create_test_registry();
    let arguments = [NamedValidationArgument::new(
        "pattern",
        ValidationArgument::String("[0-9]+"),
    )];
    assert_eq!(
        validate_text_rule(&registry, "qubit.rules.text.regex", &arguments, "123"),
        ValidationOutcome::Valid
    );
    assert_eq!(
        violation_code(validate_text_rule(
            &registry,
            "qubit.rules.text.regex",
            &arguments,
            "12a"
        )),
        "text.pattern"
    );
}
#[cfg(feature = "regex")]
/// Typed and registered regex construction share the pattern byte limit.
#[test]
fn test_regex_rejects_pattern_above_limit() {
    let allowed = "a".repeat(4_096);
    assert!(
        RegexMatch::new(&allowed).is_ok(),
        "a simple pattern at the limit must compile"
    );

    let too_long = "a".repeat(4_097);
    let error = match RegexMatch::new(&too_long) {
        Ok(_) => panic!("typed constructor accepted an oversized pattern"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
    assert_eq!(error.parameter(), Some("pattern"));

    let registry = create_test_registry();
    let args = [NamedValidationArgument::new(
        "pattern",
        ValidationArgument::String(&too_long),
    )];
    let error = match registry.bind(ids::TEXT_REGEX, InputType::Text, &args) {
        Ok(_) => panic!("registry accepted an oversized pattern"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
    assert_eq!(error.parameter(), Some("pattern"));
    assert!(matches!(RegexMatch::new("["), Err(error) if error.kind() == BindErrorKind::InvalidPattern));
}

#[cfg(feature = "regex")]
#[test]
fn test_regex_public_compiled_program_limit() {
    let pattern = r"\w{10000}";
    assert!(pattern.len() < 4_096, "exercise compiled size, not body size");
    let error = RegexMatch::new(pattern)
        .err()
        .expect("Unicode repetition must exceed the compiled program limit");
    assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
    assert_eq!(error.parameter(), Some("pattern"));
    assert!(!format!("{error}").contains(pattern));
    assert!(!format!("{error:?}").contains(pattern));

    let registry = create_test_registry();
    let arguments = [NamedValidationArgument::new(
        "pattern",
        ValidationArgument::String(pattern),
    )];
    let error = registry
        .bind(ids::TEXT_REGEX, InputType::Text, &arguments)
        .expect_err("the registry must enforce the compiled program limit");
    assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
    assert_eq!(error.parameter(), Some("pattern"));
    assert!(!format!("{error}").contains(pattern));
    assert!(!format!("{error:?}").contains(pattern));

    let accepted = "abcdefghijklmnop";
    let control = r"[a-z]{16}";
    let typed = RegexMatch::new(control).expect("small program must compile");
    assert_eq!(typed.validate(accepted, &()), Ok(()));
    assert_eq!(typed.validate("short", &()), Err(TextRuleError::Pattern));
    let arguments = [NamedValidationArgument::new(
        "pattern",
        ValidationArgument::String(control),
    )];
    let bound = registry
        .bind(ids::TEXT_REGEX, InputType::Text, &arguments)
        .expect("small program must bind");
    let context = BoundValidationContext::new(&[]);
    assert_eq!(
        bound
            .validate(ValidationValue::Text(accepted), &context)
            .expect("execute"),
        ValidationOutcome::Valid,
    );
    assert!(matches!(
        bound
            .validate(ValidationValue::Text("short"), &context)
            .expect("execute"),
        ValidationOutcome::Invalid(_),
    ));
}
