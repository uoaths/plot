use serde::{Deserialize, Serialize};

use crate::math::Range;
use crate::time;
use crate::types::{BaseQuantity, Price, QuoteQuantity};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub buying_prices: Vec<Range<Price>>,
    pub selling_prices: Vec<Range<Price>>,
    pub base_quantity: BaseQuantity,
    pub quote_quantity: QuoteQuantity,
}

impl Position {
    pub fn min_profit_trades<B, S>(&self, buy: &B, sell: &S) -> Option<Vec<Trade>>
    where
        B: Fn(&Price, &QuoteQuantity) -> Option<BaseQuantity>,
        S: Fn(&Price, &BaseQuantity) -> Option<QuoteQuantity>,
    {
        let mut result = Vec::with_capacity(3);
        let buying_price = self.max_buying_price();
        let selling_price = self.min_selling_price();

        let quote_quantity = if self.is_short() {
            self.quote_quantity
        } else {
            let trade = Trade::with_sell(selling_price, &self.base_quantity, sell)?;
            let quantity = trade.quote_quantity + self.quote_quantity;
            result.push(trade);

            quantity
        };

        let base_quantity = {
            let trade = Trade::with_buy(buying_price, &quote_quantity, buy)?;
            let quantity = trade.base_quantity.clone();
            result.push(trade);

            quantity
        };

        let _quote_quantity = {
            let trade = Trade::with_sell(selling_price, &base_quantity, sell)?;
            result.push(trade);
        };

        return Some(result);
    }

    pub fn is_short(&self) -> bool {
        self.base_quantity.is_zero()
    }

    pub fn max_buying_price(&self) -> &Price {
        let mut max_buy_price = &Price::ZERO;
        for range in self.buying_prices.iter() {
            if range.max() > max_buy_price {
                max_buy_price = range.max();
            }
        }

        max_buy_price
    }

    pub fn min_selling_price(&self) -> &Price {
        let mut min_sell_price = &Price::MAX;
        for range in self.selling_prices.iter() {
            if range.min() < min_sell_price {
                min_sell_price = range.min();
            }
        }

        min_sell_price
    }

    pub fn is_within_buying_price(&self, value: &Price) -> bool {
        if self.buying_prices.is_empty() {
            return false;
        }
    
        for range in self.buying_prices.iter() {
            if range.is_within(value) {
                return true;
            }
    
            continue;
        }
    
        false
    }

    pub fn is_within_selling_price(&self, value: &Price) -> bool {
        if self.selling_prices.is_empty() {
            return false;
        }
    
        for range in self.selling_prices.iter() {
            if range.is_within(value) {
                return true;
            }
    
            continue;
        }
    
        false
    }
}

// Buy:  base  -> quote
// Sell: quote -> base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub side: TradeSide,
    pub price: Price,
    pub base_quantity: BaseQuantity, // Actual transaction base quantity
    pub quote_quantity: QuoteQuantity, // Actual transaction quote quantity
    pub timestamp: u128,             // Actual transaction timestamp
}

impl Trade {
    pub fn new(
        side: TradeSide,
        price: Price,
        base_quantity: BaseQuantity,
        quote_quantity: QuoteQuantity,
        timestamp: u128,
    ) -> Self {
        Self {
            side,
            price,
            base_quantity,
            quote_quantity,
            timestamp,
        }
    }

    pub fn with_buy_side(
        price: Price,
        base_quantity: BaseQuantity,
        quote_quantity: QuoteQuantity,
    ) -> Self {
        Self::new(
            TradeSide::Buy,
            price,
            base_quantity,
            quote_quantity,
            time::timestamp().as_millis(),
        )
    }

    pub fn with_sell_side(
        price: Price,
        base_quantity: BaseQuantity,
        quote_quantity: QuoteQuantity,
    ) -> Self {
        Self::new(
            TradeSide::Sell,
            price,
            base_quantity,
            quote_quantity,
            time::timestamp().as_millis(),
        )
    }

    pub fn with_sell<T>(price: &Price, quantity: &BaseQuantity, sell: &T) -> Option<Self>
    where
        T: Fn(&Price, &BaseQuantity) -> Option<QuoteQuantity>,
    {
        if price < &Price::ZERO || quantity < &BaseQuantity::ZERO {
            return None;
        };

        let quote_quantity = sell(price, quantity)?;

        Some(Self::with_sell_side(
            price.clone(),
            quantity.clone(),
            quote_quantity,
        ))
    }

    pub fn with_buy<T>(price: &Price, quantity: &QuoteQuantity, buy: &T) -> Option<Self>
    where
        T: Fn(&Price, &QuoteQuantity) -> Option<BaseQuantity>,
    {
        if price < &Price::ZERO || quantity < &QuoteQuantity::ZERO {
            return None;
        };

        let act_base_quantity = buy(price, quantity)?;

        Some(Self::with_buy_side(
            price.clone(),
            act_base_quantity,
            quantity.clone(),
        ))
    }

    pub fn profit(value: &Vec<Self>) -> (BaseQuantity, QuoteQuantity) {
        let mut base_quantity = BaseQuantity::ZERO;
        let mut quote_quantity = QuoteQuantity::ZERO;

        for trade in value.iter() {
            match trade.side {
                TradeSide::Buy => {
                    base_quantity += trade.base_quantity;
                    quote_quantity -= trade.quote_quantity;
                }
                TradeSide::Sell => {
                    base_quantity -= trade.base_quantity;
                    quote_quantity += trade.quote_quantity;
                }
            }
        }

        (base_quantity, quote_quantity)
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Deserialize)]
pub enum TradeSide {
    #[serde(rename = "BUY")]
    Buy,

    #[serde(rename = "SELL")]
    Sell,
}

// ===== TESTS =====
#[cfg(test)]
mod tests_trade {
    use crate::types::Decimal;

    use super::Trade;

    fn dec(value: &str) -> Decimal {
        use std::str::FromStr;
        Decimal::from_str(value).unwrap()
    }

    #[test]
    fn test_profit() {
        let trades = vec![
            Trade::with_sell_side(dec("210"), dec("5"), dec("1050")),
            Trade::with_buy_side(dec("80"), dec("13.375"), dec("1070")),
            Trade::with_sell_side(dec("210"), dec("13.375"), dec("2808.75")),
        ];

        assert_eq!(Trade::profit(&trades), (dec("-5"), dec("2788.75")));

        let trades = vec![
            Trade::with_buy_side(dec("50"), dec("0.3996"), dec("20.0")),
            Trade::with_sell_side(dec("200"), dec("0.3996"), dec("79.8400800")),
        ];

        assert_eq!(Trade::profit(&trades), (dec("0"), dec("59.8400800")));

        let trades = vec![
            Trade::with_buy_side(dec("50"), dec("0.3996"), dec("20.0")),
            Trade::with_sell_side(dec("200"), dec("0.3996"), dec("79.8400800")),
            Trade::with_buy_side(dec("50"), dec("9.99"), dec("500.0")),
        ];

        assert_eq!(Trade::profit(&trades), (dec("9.99"), dec("-440.1599200")));

        let trades = vec![
            Trade::with_buy_side(dec("50"), dec("0.3996"), dec("20.0")),
            Trade::with_sell_side(dec("200"), dec("0.3996"), dec("79.8400800")),
            Trade::with_buy_side(dec("50"), dec("9.99"), dec("500.0")),
            Trade::with_sell_side(dec("200"), dec("9.99"), dec("1996.002")),
        ];

        assert_eq!(Trade::profit(&trades), (dec("0"), dec("1555.8420800")));

        let trades = vec![
            Trade::with_buy_side(dec("50"), dec("0.3996"), dec("20.0")),
            Trade::with_sell_side(dec("200"), dec("0.3996"), dec("79.8400800")),
            Trade::with_buy_side(dec("50"), dec("0.3996"), dec("20.0")),
        ];

        assert_eq!(Trade::profit(&trades), (dec("0.3996"), dec("39.8400800")));
    }
}

#[cfg(test)]
mod tests_position {
    use crate::math::Range;
    use crate::types::{BaseQuantity, Decimal, Price, QuoteQuantity};

    use super::Position;
    use super::Trade;

    fn buy(commission: Decimal) -> impl Fn(&Price, &QuoteQuantity) -> Option<BaseQuantity> {
        move |price: &Price, quantity: &QuoteQuantity| {
            if price > &Decimal::ZERO {
                return Some((quantity / price) * (Decimal::ONE - commission));
            }

            None
        }
    }

    fn sell(commission: Decimal) -> impl Fn(&Price, &BaseQuantity) -> Option<QuoteQuantity> {
        move |price: &Price, quantity: &BaseQuantity| {
            if price > &Decimal::ZERO {
                return Some((quantity * price) * (Decimal::ONE - commission));
            }

            None
        }
    }

    fn dec(value: &str) -> Decimal {
        use std::str::FromStr;
        Decimal::from_str(value).unwrap()
    }

    #[test]
    fn test_min_profit_trades_with_short() {
        let position = Position {
            buying_prices: vec![Range(dec("30"), dec("50"))],
            selling_prices: vec![Range(dec("200"), dec("250"))],
            base_quantity: dec("0.0"),
            quote_quantity: dec("20.0"),
        };

        let trades = position
            .min_profit_trades(&buy(dec("0")), &sell(dec("0")))
            .unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_buy_side(dec("50"), dec("0.4"), dec("20.0")),
                Trade::with_sell_side(dec("200"), dec("0.4"), dec("80.0"))
            ]
        );

        let trades = position
            .min_profit_trades(&buy(dec("0.001")), &sell(dec("0.001")))
            .unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_buy_side(dec("50"), dec("0.3996"), dec("20.0")),
                Trade::with_sell_side(dec("200"), dec("0.3996"), dec("79.8400800"))
            ]
        );
    }

    #[test]
    fn test_min_profit_trades() {
        let position = Position {
            buying_prices: vec![Range(dec("30"), dec("80"))],
            selling_prices: vec![Range(dec("210"), dec("250"))],
            base_quantity: dec("5.0"),
            quote_quantity: dec("20.0"),
        };

        let trades = position
            .min_profit_trades(&buy(dec("0")), &sell(dec("0")))
            .unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_sell_side(dec("210"), dec("5"), dec("1050")),
                Trade::with_buy_side(dec("80"), dec("13.375"), dec("1070")),
                Trade::with_sell_side(dec("210"), dec("13.375"), dec("2808.75"))
            ]
        );
    }

    #[test]
    fn test_min_profit_trades_with_mulit_prices() {
        let position = Position {
            buying_prices: vec![Range(dec("30"), dec("80")), Range(dec("90"), dec("100"))],
            selling_prices: vec![Range(dec("210"), dec("250")), Range(dec("205"), dec("200"))],
            base_quantity: dec("5.0"),
            quote_quantity: dec("20.0"),
        };

        let trades = position
            .min_profit_trades(&buy(dec("0")), &sell(dec("0")))
            .unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_sell_side(dec("200"), dec("5"), dec("1000")),
                Trade::with_buy_side(dec("100"), dec("10.2"), dec("1020")),
                Trade::with_sell_side(dec("200"), dec("10.2"), dec("2040"))
            ]
        );
    }

    impl PartialEq for Trade {
        fn eq(&self, other: &Self) -> bool {
            self.side == other.side
                && self.price == other.price
                && self.base_quantity == other.base_quantity
                && self.quote_quantity == other.quote_quantity
        }
    }
}
