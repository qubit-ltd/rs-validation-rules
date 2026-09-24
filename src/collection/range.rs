use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::Validator;

/// A comparable inclusive/exclusive range rule.
#[derive(Clone, Debug)]
pub struct Range<T> {
    lower: std::ops::Bound<T>,
    upper: std::ops::Bound<T>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
/// Range validation errors.
pub enum RangeError {
    #[error("value is outside the range")]
    /// The value falls outside an inclusive or exclusive bound.
    OutOfRange,
    #[error("value cannot be ordered")]
    /// The value cannot be ordered, even when both bounds are absent.
    Unordered,
}

impl<T: PartialOrd> Range<T> {
    /// Creates a range from `lower` and `upper`.
    /// Returns `InvalidBounds` for reversed, empty, or unordered bounds,
    /// including a single bound that cannot be compared with itself.
    pub fn new(lower: std::ops::Bound<T>, upper: std::ops::Bound<T>) -> Result<Self, BindError> {
        use std::ops::Bound::Excluded;
        use std::ops::Bound::Included;
        use std::ops::Bound::Unbounded;
        let unordered_bound = [&lower, &upper].into_iter().any(|bound| match bound {
            Included(value) | Excluded(value) => value.partial_cmp(value).is_none(),
            Unbounded => false,
        });
        if unordered_bound {
            return Err(BindError::new(BindErrorKind::InvalidBounds));
        }
        let invalid = match (&lower, &upper) {
            (Unbounded, _) | (_, Unbounded) => false,
            (Included(a), Included(b))
            | (Included(a), Excluded(b))
            | (Excluded(a), Included(b))
            | (Excluded(a), Excluded(b)) => match a.partial_cmp(b) {
                Some(std::cmp::Ordering::Greater) => true,
                Some(std::cmp::Ordering::Equal) => {
                    matches!((&lower, &upper), (Excluded(_), _) | (_, Excluded(_)))
                }
                None => true,
                _ => false,
            },
        };
        if invalid {
            return Err(BindError::new(BindErrorKind::InvalidBounds));
        }
        Ok(Self { lower, upper })
    }
}

impl<T: PartialOrd> Validator<T, ()> for Range<T> {
    type Error = RangeError;
    fn validate(&self, value: &T, _: &()) -> Result<(), Self::Error> {
        use std::ops::Bound::Excluded;
        use std::ops::Bound::Included;
        use std::ops::Bound::Unbounded;
        if value.partial_cmp(value).is_none() {
            return Err(RangeError::Unordered);
        }
        let lower_ok = match &self.lower {
            Unbounded => true,
            Included(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_ge(),
            Excluded(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_gt(),
        };
        let upper_ok = match &self.upper {
            Unbounded => true,
            Included(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_le(),
            Excluded(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_lt(),
        };
        if lower_ok && upper_ok {
            Ok(())
        } else {
            Err(RangeError::OutOfRange)
        }
    }
}
