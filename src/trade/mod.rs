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

pub trait Executor {
    fn trap(
        &mut self,
        agent: &impl Trader,
        price: &Price,
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



#[cfg(test)]
mod tests {
    use crate::types::Decimal;

    use super::Trade;

    fn dec(value: &str) -> Decimal {
        use std::str::FromStr;
        Decimal::from_str(value).unwrap()
    }
    
    #[tokio::test]
    async fn test_costs() {
        let trade = Trade::with_buy(dec("10"), dec("5"), dec("50"));
        assert_eq!(trade.costs(), dec("0"));

        let trade = Trade::with_buy(dec("50"), dec("0.3996"), dec("20.0"));
        assert_eq!(trade.costs(), dec("0.02"));

        let trade = Trade::with_buy(dec("507.545135202621"), dec("0.09841489"), dec("50"));
        assert_eq!(trade.costs(), dec("0.0500013489989265733099999825"));

        let trade = Trade::with_sell(dec("509.067770608228"), dec("0.098"), dec("49.8387528781"));
        assert_eq!(trade.costs(), dec("0.049888641506344"));

        let trade = Trade::with_sell(dec("200"), dec("0.3996"), dec("79.84008"));
        assert_eq!(trade.costs(), dec("0.07992"));
    }
}