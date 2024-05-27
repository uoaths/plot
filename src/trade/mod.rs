pub mod evaluate;
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

    pub fn with_buy(
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

    pub fn with_sell(
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

    pub fn costs(&self) -> QuoteQuantity {
        match self.side {
            TradeSide::Buy => {
                let orgin_base = self.quote_quantity / self.price;
                if self.base_quantity == orgin_base {
                    QuoteQuantity::ZERO
                } else {
                    (orgin_base - self.base_quantity) * self.price
                }
            }
            TradeSide::Sell => {
                let orgin_quote = self.base_quantity * self.price;
                if self.quote_quantity == orgin_quote {
                    QuoteQuantity::ZERO
                } else {
                    orgin_quote - self.quote_quantity
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Deserialize)]
pub enum TradeSide {
    #[serde(rename = "BUY")]
    Buy,

    #[serde(rename = "SELL")]
    Sell,
}
