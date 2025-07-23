pub mod traits;
pub mod array_board;

pub use traits::{BoardRepresentation, UndoMove};
pub use array_board::ArrayBoard;

pub type Board = ArrayBoard;