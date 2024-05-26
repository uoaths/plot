use serde::{Deserialize, Serialize};

use crate::types::Decimal;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Range<T>(pub T, pub T);

impl Range<Decimal> {
    pub fn min(&self) -> &Decimal {
        if self.0 < self.1 {
            return &self.0;
        }

        &self.1
    }

    pub fn max(&self) -> &Decimal {
        if self.0 < self.1 {
            return &self.1;
        }

        &self.0
    }

    pub fn is_within(&self, value: &Decimal) -> bool {
        self.min() <= value && value <= self.max()
    }
}
