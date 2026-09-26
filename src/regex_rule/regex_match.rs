// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::Validator;
#[cfg(feature = "regex")]
use regex::Regex;
#[cfg(feature = "regex")]
use regex::RegexBuilder;

use crate::text::TextRuleError;

/// Maximum number of UTF-8 bytes in an unanchored pattern body.
const MAX_PATTERN_BYTES: usize = 4_096;
/// Approximate maximum compiled regex program size in bytes.
const MAX_COMPILED_BYTES: usize = 8 * 1024 * 1024;
/// Maximum lazy DFA transition cache size in bytes.
const MAX_DFA_CACHE_BYTES: usize = 2 * 1024 * 1024;

/// Compiles a full-string pattern under the supplied per-rule engine limits.
///
/// Returns a parameter range error when the compiled program exceeds its
/// limit, or an invalid-pattern error when the syntax is rejected. The error
/// never retains the supplied pattern.
fn compile_with_limits(pattern: &str, program_limit: usize, dfa_limit: usize) -> Result<Regex, BindError> {
    let anchored = format!(r"\A(?:{pattern})\z");
    RegexBuilder::new(&anchored)
        .size_limit(program_limit)
        .dfa_size_limit(dfa_limit)
        .build()
        .map_err(|error| match error {
            regex::Error::CompiledTooBig(_) => {
                BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter("pattern")
            }
            _ => BindError::new(BindErrorKind::InvalidPattern),
        })
}

/// Matches the entire input against a compiled regular expression.
///
/// Anchors are added by the constructor, so callers provide only the pattern
/// body. The match is Unicode-aware according to the regex crate's defaults.
/// Pattern bodies are limited to 4,096 UTF-8 bytes, compiled programs to
/// approximately 8 MiB, and lazy DFA caches to 2 MiB per rule. Input length is
/// controlled by the caller.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::regex_rule::RegexMatch;
/// use qubit_validator::Validator;
///
/// let rule = RegexMatch::new(r"[a-z]+@[a-z]+\.com").expect("valid pattern");
/// assert!(rule.validate("user@example.com", &()).is_ok());
/// assert!(rule.validate("prefix user@example.com", &()).is_err());
/// ```
#[cfg(feature = "regex")]
pub struct RegexMatch(
    /// Compiled pattern anchored to match the entire input.
    Regex,
);

#[cfg(feature = "regex")]
impl RegexMatch {
    /// Compiles `pattern` as a full-string match.
    ///
    /// # Parameters
    /// - `pattern`: Regular-expression body to match against the complete
    ///   input.
    ///
    /// # Returns
    /// A compiled rule that succeeds only when the entire input matches.
    ///
    /// # Errors
    /// Returns `ParameterOutOfRange` for a pattern over 4,096 UTF-8 bytes or
    /// a compiled program exceeding the approximate 8 MiB size limit. Returns
    /// `InvalidPattern` for invalid regex syntax. The lazy DFA cache is
    /// limited to 2 MiB per rule.
    pub fn new(pattern: &str) -> Result<Self, BindError> {
        if pattern.len() > MAX_PATTERN_BYTES {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter("pattern"));
        }
        compile_with_limits(pattern, MAX_COMPILED_BYTES, MAX_DFA_CACHE_BYTES).map(Self)
    }
}

#[cfg(feature = "regex")]
impl Validator<str, ()> for RegexMatch {
    /// Pattern mismatch emitted when the complete input does not match.
    type Error = TextRuleError;

    /// Tests whether the compiled pattern matches the complete input.
    ///
    /// # Parameters
    /// - `value`: Text to match.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when the entire input matches the compiled pattern.
    ///
    /// # Errors
    /// Returns [`TextRuleError::Pattern`] when the pattern does not match.
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        self.0.is_match(value).then_some(()).ok_or(TextRuleError::Pattern)
    }
}

#[cfg(test)]
mod tests {
    use qubit_validator::BindErrorKind;

    use super::compile_with_limits;

    /// Classifies a compiled-program limit as a pattern parameter failure.
    #[test]
    fn test_compiled_program_limit_has_structured_bind_error() {
        let error = match compile_with_limits("abc", 1, 2 * 1024 * 1024) {
            Ok(_) => panic!("one byte cannot hold the compiled program"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
        assert_eq!(error.parameter(), Some("pattern"));
    }
}
