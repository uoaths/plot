use serde::{Deserialize, Serialize};

use crate::types::{BaseQuantity, Price, QuoteQuantity};

use super::{Trade, TradeSide};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evaluate {
    pub volume_base_quantity: BaseQuantity,
    pub volume_quote_quantity: QuoteQuantity,
    pub leave_base_quantity: BaseQuantity,
    pub leave_quote_quantity: QuoteQuantity,
    pub buy_count: usize,
    pub sell_count: usize,
    pub max_price: Price,
    pub min_price: Price,
    pub costs: QuoteQuantity,
}

impl Default for Evaluate {
    fn default() -> Self {
        Self {
            volume_base_quantity: BaseQuantity::ZERO,
            volume_quote_quantity: QuoteQuantity::ZERO,
            leave_base_quantity: BaseQuantity::ZERO,
            leave_quote_quantity: QuoteQuantity::ZERO,
            buy_count: 0,
            sell_count: 0,
            max_price: Price::ZERO,
            min_price: Price::MAX,
            costs: QuoteQuantity::ZERO,
        }
    }
}

pub trait Evaluater {
    fn evaluate(&self) -> impl std::future::Future<Output = Evaluate> + Send;
}

impl Evaluater for Vec<Trade> {
    async fn evaluate(&self) -> Evaluate {
        let mut report = Evaluate::default();

        if self.is_empty() {
            return report;
        }

        for trade in self.iter() {
            if trade.price > report.max_price {
                report.max_price = trade.price
            }

            if trade.price < report.min_price {
                report.min_price = trade.price
            }

            report.costs += trade.costs();
            report.volume_base_quantity += trade.base_quantity;
            report.volume_quote_quantity += trade.quote_quantity;

            match trade.side {
                TradeSide::Buy => {
                    report.buy_count += 1;
                    report.leave_base_quantity += trade.base_quantity;
                    report.leave_quote_quantity -= trade.quote_quantity;
                }
                TradeSide::Sell => {
                    report.sell_count += 1;
                    report.leave_base_quantity -= trade.base_quantity;
                    report.leave_quote_quantity += trade.quote_quantity;
                }
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use crate::trade::evaluate::{Evaluate, Evaluater};
    use crate::trade::Trade;
    use crate::types::Decimal;

    fn dec(value: &str) -> Decimal {
        use std::str::FromStr;
        Decimal::from_str(value).unwrap()
    }

    #[tokio::test]
    async fn test_evaluate() {
        let trades = vec![
            Trade::with_sell(dec("210"), dec("5"), dec("1050")),
            Trade::with_buy(dec("80"), dec("13.375"), dec("1070")),
            Trade::with_sell(dec("210"), dec("13.375"), dec("2808.75")),
        ];

        assert_eq!(
            trades.evaluate().await,
            Evaluate {
                volume_base_quantity: dec("31.750"),
                volume_quote_quantity: dec("4928.75"),
                leave_base_quantity: dec("-5"),
                leave_quote_quantity: dec("2788.75"),
                buy_count: 1,
                sell_count: 2,
                max_price: dec("210"),
                min_price: dec("80"),
                costs: dec("0")
            }
        );

        let trades = vec![
            Trade::with_buy(dec("50"), dec("0.3996"), dec("20.0")),
            Trade::with_sell(dec("200"), dec("0.3996"), dec("79.8400800")),
        ];

        assert_eq!(
            trades.evaluate().await,
            Evaluate {
                volume_base_quantity: dec("0.7992"),
                volume_quote_quantity: dec("99.8400800"),
                leave_base_quantity: dec("0"),
                leave_quote_quantity: dec("59.8400800"),
                buy_count: 1,
                sell_count: 1,
                max_price: dec("200"),
                min_price: dec("50"),
                costs: dec("0.0999200")
            }
        );

        let trades = vec![
            Trade::with_buy(dec("50"), dec("0.3996"), dec("20.0")),
            Trade::with_sell(dec("200"), dec("0.3996"), dec("79.8400800")),
            Trade::with_buy(dec("50"), dec("9.99"), dec("500.0")),
        ];

        assert_eq!(
            trades.evaluate().await,
            Evaluate {
                volume_base_quantity: dec("10.7892"),
                volume_quote_quantity: dec("599.8400800"),
                leave_base_quantity: dec("9.99"),
                leave_quote_quantity: dec("-440.1599200"),
                buy_count: 2,
                sell_count: 1,
                max_price: dec("200"),
                min_price: dec("50"),
                costs: dec("0.5999200")
            }
        );

        let trades = vec![
            Trade::with_buy(dec("507.545135202621"), dec("0.09841489"), dec("50")),
            Trade::with_sell(dec("509.067770608228"), dec("0.098"), dec("49.8387528781")),
        ];

        assert_eq!(
            trades.evaluate().await,
            Evaluate {
                volume_base_quantity: dec("0.19641489"),
                volume_quote_quantity: dec("99.8387528781"),
                leave_base_quantity: dec("0.00041489"),
                leave_quote_quantity: dec("-0.1612471219"),
                buy_count: 1,
                sell_count: 1,
                max_price: dec("509.067770608228"),
                min_price: dec("507.545135202621"),
                costs: dec("0.0998899905052705733099999825")
            }
        );
    }
}
