use serde::{Deserialize, Serialize};

use crate::math::Range;
use crate::types::{Decimal, Price, QuoteQuantity};

use super::{Position, Strategy};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridPercent {
    pub investment: QuoteQuantity,
    pub range: Range<Price>,
    pub percent: Decimal,
    pub percent_lost: Decimal,
}

impl GridPercent {
    pub fn new(investment: QuoteQuantity, range: Range<Price>, percent: Decimal, percent_lost: Decimal) -> Self {
        Self {
            investment,
            range,
            percent,
            percent_lost
        }
    }
}

impl Strategy for GridPercent {
    fn assign_position(&self) -> Vec<Position> {
        let initial_price = self.range.min().clone();
        let termination_price = self.range.max().clone();
        let percentage_increase = Decimal::ONE + self.percent;
        let percentage_lost = Decimal::ONE - self.percent_lost;

        let mut prices = vec![initial_price];
        loop {
            let new_price = prices.last().unwrap() * percentage_increase;
            let new_price = new_price.trunc_with_scale(12);
            if new_price >= termination_price {
                break;
            }

            prices.push(new_price);
        }

        let mut positions = Vec::with_capacity(prices.len());
        let mut num = 0;
        for i in 0..prices.len() {
            let index = i + num;

            let buy_0 = match prices.get(index) {
                Some(v) => *v,
                None => return positions,
            };

            let buy_1 = match prices.get(index + 1) {
                Some(v) => *v,
                None => return positions,
            };

            let sell_0 = match prices.get(index + 2) {
                Some(v) => *v,
                None => return positions,
            };

            if let None = prices.get(index + 3) {
                return positions;
            }

            let selling_prices = {
                if Decimal::ZERO < percentage_lost && percentage_lost < Decimal::ONE {
                    vec![Range(sell_0, termination_price.clone()), Range(Decimal::ZERO, sell_0 * percentage_lost)]
                } else {
                    vec![Range(sell_0, termination_price.clone())]
                }
            };

            positions.push(Position {
                buying_prices: vec![Range(buy_0, buy_1)],
                selling_prices: selling_prices,
                base_quantity: Decimal::ZERO,
                quote_quantity: self.investment.clone(),
            });

            num += 3;
        }

        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(value: &str) -> Decimal {
        use std::str::FromStr;
        Decimal::from_str(value).unwrap()
    }

    #[test]
    fn test_positions() {
        let grid = GridPercent::new(dec("100"), Range(dec("50"), dec("60")), dec("0.01"), dec("0"));
        let positions = grid.assign_position();

        assert_eq!(
            positions,
            vec![
                Position {
                    buying_prices: vec![Range(dec("50"), dec("50.5"))],
                    selling_prices: vec![Range(dec("51.005"), dec("60"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("100")
                },
                Position {
                    buying_prices: vec![Range(dec("52.0302005"), dec("52.550502505"))],
                    selling_prices: vec![Range(dec("53.07600753005"), dec("60"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("100")
                },
                Position {
                    buying_prices: vec![Range(dec("54.142835281403"), dec("54.684263634217"))],
                    selling_prices: vec![Range(dec("55.231106270559"), dec("60"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("100")
                },
                Position {
                    buying_prices: vec![Range(dec("56.341251506596"), dec("56.904664021661"))],
                    selling_prices: vec![Range(dec("57.473710661877"), dec("60"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("100")
                }
            ]
        );

        let grid = GridPercent::new(dec("100"), Range(dec("100"), dec("200")), dec("0.05"), dec("0"));
        let positions = grid.assign_position();

        assert_eq!(
            positions,
            vec![
                Position {
                    buying_prices: vec![Range(dec("100"), dec("105"))],
                    selling_prices: vec![Range(dec("110.25"), dec("200"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("100")
                },
                Position {
                    buying_prices: vec![Range(dec("121.550625"), dec("127.62815625"))],
                    selling_prices: vec![Range(dec("134.0095640625"), dec("200"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("100")
                },
                Position {
                    buying_prices: vec![Range(dec("147.745544378906"), dec("155.132821597851"))],
                    selling_prices: vec![Range(dec("162.889462677743"), dec("200"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("100")
                }
            ]
        );
    }

    #[test]
    fn test_positions_stop_lost() {
        let grid = GridPercent::new(dec("100"), Range(dec("100"), dec("200")), dec("0.05"), dec("0.1"));
        let positions = grid.assign_position();

        assert_eq!(
            positions,
            vec![
                Position {
                    buying_prices: vec![Range(dec("100"), dec("105"))],
                    selling_prices: vec![Range(dec("110.25"), dec("200")), Range(dec("0"), dec("99.225"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("100")
                },
                Position {
                    buying_prices: vec![Range(dec("121.550625"), dec("127.62815625"))],
                    selling_prices: vec![Range(dec("134.0095640625"), dec("200")), Range(dec("0"), dec("120.60860765625"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("100")
                },
                Position {
                    buying_prices: vec![Range(dec("147.745544378906"), dec("155.132821597851"))],
                    selling_prices: vec![Range(dec("162.889462677743"), dec("200")), Range(dec("0"), dec("146.6005164099687"))],
                    base_quantity: dec("0"),
                    quote_quantity: dec("100")
                }
            ]
        );
    }
}
