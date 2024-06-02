pub mod grid;
pub mod grid_percent;

use crate::trade::position::Position;
pub trait Strategy {
    fn assign_position(&self) -> Vec<Position>;
}
