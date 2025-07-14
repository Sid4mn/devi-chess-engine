use crate::types::*;

pub struct Board {
    squares: [Option<Piece>; 64],
    to_move: Color,
    castling_rights: u8,
    en_passant: Option<Square>,
    halfmove_clock: u8,
    fullmove_clock: u16,
}

impl Board {
    pub fn new() -> Self {
        Board {
            squares: [None; 64],
            to_move: Color::White,
            castling_rights: 0b111,
            en_passant: None,
            halfmove_clock: 0, 
            fullmove_clock: 1,
        }
    }

    pub fn get_piece(&self, square: Square) -> Option<Piece> {
        self.squares[square.0 as usize]
    }

    pub fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        self.squares[square.0 as usize] = piece;
    }

    pub fn is_empty(&self, square: Square) -> bool {
        self.squares[square.0 as usize].is_none()
    }
}