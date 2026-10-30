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
/// Maximum AST nesting depth accepted for a caller-supplied pattern body.
const MAX_PATTERN_NESTING: u32 = 250;
/// Extra AST nesting budget for concatenation and the non-capturing wrapper.
const FULL_MATCH_NESTING_OVERHEAD: u32 = 2;

/// Compiles a pattern under the supplied per-rule engine limits.
///
/// Returns a parameter range error when the compiled program exceeds its
/// limit, or an invalid-pattern error when the syntax is rejected. The error
/// never retains the supplied pattern.
fn compile_pattern(pattern: &str, program_limit: usize, dfa_limit: usize, nest_limit: u32) -> Result<Regex, BindError> {
    RegexBuilder::new(pattern)
        .nest_limit(nest_limit)
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

/// Compiles a full-string pattern under the supplied per-rule engine limits.
///
/// Validates the body before adding full-string anchors with a separate nesting
/// allowance for the concatenation and non-capturing wrapper.
///
/// Invalid bodies are rejected before interpolation. If a valid extended-mode
/// trailing comment consumes the wrapper suffix, a newline terminates it before
/// retrying. Every compilation uses the supplied limits and redacted errors.
fn compile_with_limits(pattern: &str, program_limit: usize, dfa_limit: usize) -> Result<Regex, BindError> {
    compile_pattern(pattern, program_limit, dfa_limit, MAX_PATTERN_NESTING)?;
    let wrapped_limit = MAX_PATTERN_NESTING + FULL_MATCH_NESTING_OVERHEAD;
    let anchored = format!(r"\A(?:{pattern})\z");
    match compile_pattern(&anchored, program_limit, dfa_limit, wrapped_limit) {
        Err(error) if error.kind() == BindErrorKind::InvalidPattern => {
            let comment_terminated = format!("\\A(?:{pattern}\n)\\z");
            compile_pattern(&comment_terminated, program_limit, dfa_limit, wrapped_limit)
        }
        result => result,
    }
}

/// Matches the entire input against a compiled regular expression.
///
/// Anchors are added by the constructor, so callers provide only the pattern
/// body. The match is Unicode-aware according to the regex crate's defaults.
/// Pattern bodies are limited to 4,096 UTF-8 bytes, compiled programs to
/// approximately 8 MiB, lazy DFA caches to 2 MiB per rule, and AST nesting
/// depth to 250. The internal full-match wrapper uses a separate two-level
/// allowance for its concatenation and non-capturing group, so it does not
/// consume the body's budget. AST nesting depth is not the same as counting
/// parentheses in the pattern. Bodies exceeding the depth limit return
/// `InvalidPattern`. Input length is controlled by the caller.
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
    /// `InvalidPattern` for invalid regex syntax or an AST nesting depth above
    /// 250. AST depth is not equivalent to the number of parentheses in the
    /// pattern; the internal concatenation and non-capturing wrapper use a
    /// separate two-level allowance. The lazy DFA cache is limited to 2 MiB
    /// per rule.
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
