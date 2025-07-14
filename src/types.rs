#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialMove {
    Castle,
    Promotion,
    EnPassant
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    pub fn new( p_type: PieceType, color: Color) -> Piece {
        Piece {piece_type: p_type, color: color, }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square (pub u8); // 0-63


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub special_move: Option<SpecialMove>,
}