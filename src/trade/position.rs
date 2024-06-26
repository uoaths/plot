use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::math::Range;
use crate::types::{BaseQuantity, Price, QuoteQuantity};

use super::{Executor, Trade, Trader};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub buying_prices: Vec<Range<Price>>,
    pub selling_prices: Vec<Range<Price>>,
    pub base_quantity: BaseQuantity,
    pub quote_quantity: QuoteQuantity,
}

impl Position {
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

    pub async fn min_profit_trades(
        &mut self,
        agent: &impl Trader,
    ) -> Result<Vec<Trade>, Box<dyn Error>> {
        let buying_price = self.max_buying_price();
        let selling_price = self.min_selling_price();

        let prices = vec![*selling_price, *buying_price, *selling_price];

        let mut trades = Vec::new();
        for price in prices.iter() {
            trades.extend(self.trap(agent, &price).await?);
        }

        Ok(trades)
    }
}

impl Executor for Position {
    async fn trap(
        &mut self,
        agent: &impl Trader,
        price: &Price,
    ) -> Result<Vec<Trade>, Box<dyn Error>> {
        let mut trades = Vec::new();

        if self.is_within_selling_price(price) && !self.base_quantity.is_zero() {
            trades.extend(agent.sell(price, &self.base_quantity).await?);

            for trade in trades.iter() {
                self.base_quantity -= trade.base_quantity;
                self.quote_quantity += trade.quote_quantity;
            }
        }

        if self.is_within_buying_price(price) && !self.quote_quantity.is_zero() {
            trades.extend(agent.buy(price, &self.quote_quantity).await?);

            for trade in trades.iter() {
                self.base_quantity += trade.base_quantity;
                self.quote_quantity -= trade.quote_quantity;
            }
        }

        Ok(trades)
    }
}

impl Executor for Vec<Position> {
    async fn trap(
        &mut self,
        agent: &impl Trader,
        price: &Price,
    ) -> Result<Vec<Trade>, Box<dyn Error>> {
        let mut trades = Vec::new();

        for position in self.iter_mut() {
            trades.extend(position.trap(agent, price).await?);
        }

        Ok(trades)
    }
}

#[cfg(test)]
mod tests_position {
    use std::error::Error;

    use crate::math::Range;
    use crate::trade::{Executor, Trader};
    use crate::types::{BaseQuantity, Decimal, Price, QuoteQuantity};

    use super::Position;
    use super::Trade;

    struct TradeAgent {
        commission: Decimal,
    }

    impl TradeAgent {
        fn with_commission(value: &str) -> Self {
            Self {
                commission: dec(value),
            }
        }
    }

    impl Default for TradeAgent {
        fn default() -> Self {
            Self {
                commission: dec("0"),
            }
        }
    }

    impl Trader for TradeAgent {
        async fn buy(
            &self,
            price: &Price,
            quote_quantity: &QuoteQuantity,
        ) -> Result<Vec<Trade>, Box<dyn Error>> {
            if price > &Decimal::ZERO {
                let base_quantity = (quote_quantity / price) * (Decimal::ONE - self.commission);

                return Ok(vec![Trade::with_buy(
                    price.clone(),
                    base_quantity,
                    quote_quantity.clone(),
                )]);
            };

            Err("Buy Trade Error")?
        }

        async fn sell(
            &self,
            price: &Price,
            base_quantity: &BaseQuantity,
        ) -> Result<Vec<Trade>, Box<dyn Error>> {
            if price > &Decimal::ZERO {
                let quote_quantity = (base_quantity * price) * (Decimal::ONE - self.commission);

                return Ok(vec![Trade::with_sell(
                    price.clone(),
                    base_quantity.clone(),
                    quote_quantity.clone(),
                )]);
            };

            Err("Sell Trade Error")?
        }
    }

    fn dec(value: &str) -> Decimal {
        use std::str::FromStr;
        Decimal::from_str(value).unwrap()
    }

    #[tokio::test]
    async fn test_min_profit_trades_with_short() {
        let position = Position {
            buying_prices: vec![Range(dec("30"), dec("50"))],
            selling_prices: vec![Range(dec("200"), dec("250"))],
            base_quantity: dec("0.0"),
            quote_quantity: dec("20.0"),
        };

        let agent = TradeAgent::default();

        let trades = position.clone().min_profit_trades(&agent).await.unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_buy(dec("50"), dec("0.4"), dec("20.0")),
                Trade::with_sell(dec("200"), dec("0.4"), dec("80.0"))
            ]
        );

        let agent = TradeAgent::with_commission("0.001");
        let trades = position.clone().min_profit_trades(&agent).await.unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_buy(dec("50"), dec("0.3996"), dec("20.0")),
                Trade::with_sell(dec("200"), dec("0.3996"), dec("79.8400800"))
            ]
        );
    }

    #[tokio::test]
    async fn test_min_profit_trades() {
        let mut position = Position {
            buying_prices: vec![Range(dec("30"), dec("80"))],
            selling_prices: vec![Range(dec("210"), dec("250"))],
            base_quantity: dec("5.0"),
            quote_quantity: dec("20.0"),
        };

        let trades = position
            .min_profit_trades(&TradeAgent::with_commission("0"))
            .await
            .unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_sell(dec("210"), dec("5"), dec("1050")),
                Trade::with_buy(dec("80"), dec("13.375"), dec("1070")),
                Trade::with_sell(dec("210"), dec("13.375"), dec("2808.75"))
            ]
        );
    }

    #[tokio::test]
    async fn test_min_profit_trades_with_mulit_prices() {
        let mut position = Position {
            buying_prices: vec![Range(dec("30"), dec("80")), Range(dec("90"), dec("100"))],
            selling_prices: vec![Range(dec("210"), dec("250")), Range(dec("205"), dec("200"))],
            base_quantity: dec("5.0"),
            quote_quantity: dec("20.0"),
        };

        let trades = position
            .min_profit_trades(&TradeAgent::with_commission("0"))
            .await
            .unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_sell(dec("200"), dec("5"), dec("1000")),
                Trade::with_buy(dec("100"), dec("10.2"), dec("1020")),
                Trade::with_sell(dec("200"), dec("10.2"), dec("2040"))
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

    #[tokio::test]
    async fn test_trap_same_price() {
        let mut position = Position {
            buying_prices: vec![Range(dec("30"), dec("80"))],
            selling_prices: vec![Range(dec("70"), dec("80"))],
            base_quantity: dec("5.0"),
            quote_quantity: dec("20.0"),
        };

        let trades = position
            .trap(&TradeAgent::with_commission("0"), &dec("80"))
            .await
            .unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_sell(dec("80"), dec("5"), dec("400")),
                Trade::with_buy(dec("80"), dec("5.250"), dec("420.0"))
            ]
        );
    }

    #[tokio::test]
    async fn test_trap() {
        let mut position = Position {
            buying_prices: vec![Range(dec("10"), dec("20"))],
            selling_prices: vec![Range(dec("50"), dec("80"))],
            base_quantity: dec("0"),
            quote_quantity: dec("20.0"),
        };

        let trades = position
            .trap(&TradeAgent::with_commission("0"), &dec("80"))
            .await
            .unwrap();
        assert_eq!(trades, vec![]);

        let trades = position
            .trap(&TradeAgent::with_commission("0"), &dec("20"))
            .await
            .unwrap();
        assert_eq!(trades, vec![
            Trade::with_buy(dec("20"), dec("1"), dec("20"))
        ]);
    }
}
