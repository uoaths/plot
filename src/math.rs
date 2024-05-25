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
        self.min() < value && value < self.max()
    }
}

pub fn is_within_ranges(value: &Decimal, ranges: &Vec<Range<Decimal>>) -> bool {
    if ranges.is_empty() {
        return false;
    }

    for range in ranges.iter() {
        if range.is_within(value) {
            return true;
        }

        continue;
    }

    false
}
