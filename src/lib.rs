mod time;

pub mod math;
pub mod plot;
pub mod trade;

pub mod types {
    pub use rust_decimal::Decimal;

    pub type Price = Decimal;
    pub type Quantity = Decimal;
    pub type BaseQuantity = Quantity;
    pub type QuoteQuantity = Quantity;
}

pub mod error {
    pub use rust_decimal::Error;
}

pub mod prelude {
    pub use super::plot;
    pub use super::trade;
}
