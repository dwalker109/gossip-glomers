//! An enum which can hold a known optional value, or a signal
//! that a generated/inferred value should be set at some point.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[serde(from = "Option<T>")]
#[serde(into = "Option<T>")]
pub enum Id<T: Clone> {
    Known(Option<T>),
    Defer,
}

impl<T: Clone> From<Option<T>> for Id<T> {
    fn from(value: Option<T>) -> Self {
        Id::Known(value)
    }
}

impl<T: Clone> From<Id<T>> for Option<T> {
    fn from(value: Id<T>) -> Self {
        match value {
            Id::Known(value) => value,
            Id::Defer => None,
        }
    }
}

impl<T: Clone> Id<T> {
    /// When self is `Defer`, substitute the provided value.
    pub fn coalesce(self, val: Id<T>) -> Self {
        match self {
            Id::Defer => val,
            _ => self,
        }
    }

    /// When self is `Defer`, substitute the provided value by calling `coalesce_fn`.
    pub fn else_coalesce(self, coalesce_fn: impl Fn() -> Id<T>) -> Self {
        match self {
            Id::Defer => coalesce_fn(),
            _ => self,
        }
    }
}
