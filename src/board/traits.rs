use crate::types::*;

#[derive(Debug, Clone)]
pub struct UndoMove {
    pub captured_piece: Option<Piece>,
    pub previous_en_passant: Option<Square>,
    pub previous_castling_rights: u8,
    pub previous_halfmove_clock: u8,
    pub previous_to_move: Color,
}
pub trait BoardRepresentation {
    fn get_piece(&self, square: Square) -> Option<Piece>;
    fn set_piece(&mut self, square: Square, piece: Option<Piece>);
    fn is_empty(&self, square: Square) -> bool;

    //Game State
    fn to_move(&self) -> Color;
    fn set_to_move(&mut self, color: Color);
    fn en_passant(&self) -> Option<Square>;
    fn set_en_passant(&mut self, square: Option<Square>);
    fn castling_rights(&self) -> u8;
    fn set_castling_rights(&mut self, rights: u8);
    fn halfmove_clock(&self) -> u8;
    fn set_halfmove_clock(&mut self, clock: u8);
    fn fullmove_clock(&self) -> u16;
    fn set_fullmove_clock(&mut self, clock: u16);

    // Move execution
    fn make_move(&mut self, mv: &Move) -> UndoMove;
    fn unmake_move(&mut self, mv: &Move, undo: UndoMove);
    
    // Position queries
    fn find_king(&self, color: Color) -> Option<Square>;
    fn is_in_check(&self, color: Color) -> bool;
    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool;
    
    // Board setup/manipulation
    fn setup_starting_position(&mut self);
    fn clear(&mut self);
    
    // Utility methods
    fn count_pieces(&self, piece_type: PieceType, color: Color) -> u8 {
        let mut count = 0;
        for i in 0..64 {
            if let Some(piece) = self.get_piece(Square(i)) {
                if piece.piece_type == piece_type && piece.color == color {
                    count += 1;
                }
            }
        }
        count
    }
    
    fn to_fen(&self) -> String;
    fn from_fen(fen: &str) -> Result<Self, String> where Self: Sized;
}
