mod time;

pub mod math;
pub mod position;

pub mod types {
    pub use rust_decimal::Decimal;

    pub type Price = Decimal;
    pub type Quantity = Decimal;
    pub type BaseQuantity = Quantity;
    pub type QuoteQuantity = Quantity;
}

pub trait Ploy {
    fn trap(&self) -> Vec<position::Position>;
}

pub mod prelude {
    pub use super::position::{Position, Trade, TradeSide};
    pub use super::Ploy;
}
