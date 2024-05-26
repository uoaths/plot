pub mod grid;

use crate::trade::position::Position;
pub trait Plot {
    fn trap(&self) -> Vec<Position>;
}
