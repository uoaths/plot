pub mod position;

use std::error::Error;
use std::future::Future;

use serde::{Deserialize, Serialize};

use crate::time;
use crate::types::{BaseQuantity, Price, QuoteQuantity};

pub trait Trader {
    fn buy(
        &self,
        price: &Price,
        quantity: &QuoteQuantity,
    ) -> impl Future<Output = Result<Vec<Trade>, Box<dyn Error>>> + Send;

    fn sell(
        &self,
        price: &Price,
        quantity: &BaseQuantity,
    ) -> impl Future<Output = Result<Vec<Trade>, Box<dyn Error>>> + Send;
}

pub trait Tracker {
    fn track(
        &mut self,
        trader: &impl Trader,
        prices: &Vec<Price>,
    ) -> impl Future<Output = Result<Vec<Trade>, Box<dyn Error>>>;
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
