use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum Float {
    Nan,
    PositiveInfinity,
    NegativeInfinity,
    Normal(Normal),
}

impl Float {
    pub fn new(value: f64) -> Self {
        use std::num::FpCategory as E;
        match value.classify() {
            E::Nan => Self::Nan,
            E::Infinite => match () {
                () if value.is_sign_positive() => Self::PositiveInfinity,
                () if value.is_sign_negative() => Self::NegativeInfinity,
                () => unreachable!(),
            },
            _ => Self::Normal(Normal::new(value)),
        }
    }

    pub fn as_f64(self) -> f64 {
        match self {
            Self::Nan => f64::NAN,
            Self::PositiveInfinity => f64::INFINITY,
            Self::NegativeInfinity => f64::NEG_INFINITY,
            Self::Normal(n) => n.as_f64(),
        }
    }
}

// new type is required for privacy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub struct Normal(OrderedFloat<f64>);

impl Normal {
    pub fn new(value: f64) -> Self {
        Self(OrderedFloat(value))
    }

    pub fn as_f64(self) -> f64 {
        self.0.into_inner()
    }
}

