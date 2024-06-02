pub mod grid;

use crate::trade::position::Position;
pub trait Strategy {
    fn assign_position(&self) -> Vec<Position>;
}
