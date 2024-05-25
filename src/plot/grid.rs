use serde::{Deserialize, Serialize};

use crate::math::Range;
use crate::position::Position;
use crate::types::{Decimal, Price, QuoteQuantity};
use crate::Ploy;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    pub investment: QuoteQuantity,
    pub range: Range<Price>,
    pub copies: usize,
}

impl Grid {
    pub fn new(investment: QuoteQuantity, range: Range<Price>, copies: usize) -> Self {
        Self {
            investment,
            range,
            copies,
        }
    }
}

impl Ploy for Grid {
    fn trap(&self) -> Vec<Position> {
        let mut result = Vec::with_capacity(self.copies);
        let copies = Decimal::from(self.copies);
        let price_highest = self.range.max();
        let price_lowest = self.range.min();

        let interval = (price_highest - price_lowest) / (copies + Decimal::ONE);
        let interval_quote_quantity = self.investment / copies;

        let interval = interval.trunc_with_scale(6);
        let interval_quote_quantity = interval_quote_quantity.trunc_with_scale(6);

        for i in 0..self.copies {
            let buying = price_lowest + interval * Decimal::from(i);
            let selling = price_lowest + interval * Decimal::from(i + 2);
            let position = Position {
                buying_prices: vec![Range(buying, buying + (interval / Decimal::TWO))],
                selling_prices: vec![Range(
                    selling - (interval / Decimal::TWO),
                    price_highest.clone(),
                )],
                base_quantity: Decimal::ZERO,
                quote_quantity: interval_quote_quantity,
            };

            result.push(position)
        }

        result
    }
}

#[cfg(test)]
mod tests_grid {
    use super::*;

    fn dec(value: &str) -> Decimal {
        use std::str::FromStr;
        Decimal::from_str(value).unwrap()
    }

    #[test]
    fn test_trap() {
        let grid = Grid {
            investment: dec("30"),
            range: Range(dec("50"), dec("100")),
            copies: 1,
        };

        assert_eq!(
            grid.trap(),
            vec![Position {
                buying_prices: vec![Range(dec("50"), dec("62.5"))],
                selling_prices: vec![Range(dec("87.5"), dec("100"))],
                base_quantity: dec("0"),
                quote_quantity: dec("30.0")
            },]
        );

        let grid = Grid {
            investment: dec("30"),
            range: Range(dec("50"), dec("100")),
            copies: 2,
        };

        assert_eq!(
            grid.trap(),
            vec![
                Position {
                    buying_prices: vec![Range(dec("50"), dec("58.333333"))],
                    selling_prices: vec![Range(dec("74.999999"), dec("100"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("15.0")
                },
                Position {
                    buying_prices: vec![Range(dec("66.666666"), dec("74.999999"))],
                    selling_prices: vec![Range(dec("91.666665"), dec("100"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("15.0")
                },
            ]
        );

        let grid = Grid {
            investment: dec("30"),
            range: Range(dec("50"), dec("100")),
            copies: 3,
        };

        assert_eq!(
            grid.trap(),
            vec![
                Position {
                    buying_prices: vec![Range(dec("50"), dec("56.250000"))],
                    selling_prices: vec![Range(dec("68.750000"), dec("100"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("10.0")
                },
                Position {
                    buying_prices: vec![Range(dec("62.500000"), dec("68.750000"))],
                    selling_prices: vec![Range(dec("81.250000"), dec("100"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("10.0")
                },
                Position {
                    buying_prices: vec![Range(dec("75.000000"), dec("81.250000"))],
                    selling_prices: vec![Range(dec("93.750000"), dec("100"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("10.0")
                },
            ]
        );
    }
}
