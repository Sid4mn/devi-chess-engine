pub mod array_board;
pub mod traits;

pub use array_board::ArrayBoard;
pub use traits::{BoardRepresentation, UndoMove};

pub type Board = ArrayBoard;
