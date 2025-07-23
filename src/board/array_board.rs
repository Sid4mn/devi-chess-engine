use crate::types::*;
use super::traits::{BoardRepresentation, UndoMove};

pub struct ArrayBoard {
    squares: [Option<Piece>; 64],
    to_move: Color,
    castling_rights: u8,
    en_passant: Option<Square>,
    halfmove_clock: u8,
    fullmove_clock: u16,
}

impl ArrayBoard {
    pub fn new() -> Self {
        ArrayBoard {
            squares: [None; 64],
            to_move: Color::White,
            castling_rights: 0b1111,
            en_passant: None,
            halfmove_clock: 0, 
            fullmove_clock: 1,
        }
    }
}

impl BoardRepresentation for ArrayBoard {
    fn get_piece(&self, square: Square) -> Option<Piece> {
        self.squares[square.0 as usize]
    }

    fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        self.squares[square.0 as usize] = piece;
    }

    fn is_empty(&self, square: Square) -> bool {
        self.squares[square.0 as usize].is_none()
    }

    fn to_move(&self) -> Color {
        self.to_move
    }
    
    fn set_to_move(&mut self, color: Color) {
        self.to_move = color;
    }
    
    fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }
    
    fn set_en_passant(&mut self, square: Option<Square>) {
        self.en_passant = square;
    }
    
    fn castling_rights(&self) -> u8 {
        self.castling_rights
    }
    
    fn set_castling_rights(&mut self, rights: u8) {
        self.castling_rights = rights;
    }
    
    fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }
    
    fn set_halfmove_clock(&mut self, clock: u8) {
        self.halfmove_clock = clock;
    }
    
    fn fullmove_clock(&self) -> u16 {
        self.fullmove_clock
    }
    
    fn set_fullmove_clock(&mut self, clock: u16) {
        self.fullmove_clock = clock;
    }

    fn setup_starting_position(&mut self) {
        self.set_piece(Square(0), Some(Piece::new(PieceType::Rook, Color::White))); //a1
        self.set_piece(Square(1), Some(Piece::new(PieceType::Knight, Color::White))); //b1
        self.set_piece(Square(2), Some(Piece::new(PieceType::Bishop, Color::White))); //c1
        self.set_piece(Square(3), Some(Piece::new(PieceType::Queen, Color::White))); //d1
        self.set_piece(Square(4), Some(Piece::new(PieceType::King, Color::White))); //e1
        self.set_piece(Square(5), Some(Piece::new(PieceType::Bishop, Color::White))); //f1
        self.set_piece(Square(6), Some(Piece::new(PieceType::Knight, Color::White))); //g1
        self.set_piece(Square(7), Some(Piece::new(PieceType::Rook, Color::White))); //h1

        for i in 8..16 { // 2
            self.set_piece(Square(i), Some(Piece::new(PieceType::Pawn, Color::White))); 
        }

        for i in 48..56 { // 7
            self.set_piece(Square(i), Some(Piece::new(PieceType::Pawn, Color::Black))); 
        }

        self.set_piece(Square(56), Some(Piece::new(PieceType::Rook, Color::Black))); //a8
        self.set_piece(Square(57), Some(Piece::new(PieceType::Knight, Color::Black))); //b8
        self.set_piece(Square(58), Some(Piece::new(PieceType::Bishop, Color::Black))); //c8
        self.set_piece(Square(59), Some(Piece::new(PieceType::Queen, Color::Black))); //d8
        self.set_piece(Square(60), Some(Piece::new(PieceType::King, Color::Black))); //e8
        self.set_piece(Square(61), Some(Piece::new(PieceType::Bishop, Color::Black))); //f8
        self.set_piece(Square(62), Some(Piece::new(PieceType::Knight, Color::Black))); //g8
        self.set_piece(Square(63), Some(Piece::new(PieceType::Rook, Color::Black))); //h8

    }

    fn clear(&mut self) {
        self.squares = [None; 64];
    }
    
    //Stub the methods TBD
    fn make_move(&mut self, _mv: &Move) -> UndoMove {
        todo!("Implement make_move")
    }
    
    fn unmake_move(&mut self, _mv: &Move, _undo: UndoMove) {
        todo!("Implement unmake_move")
    }
    
    fn find_king(&self, _color: Color) -> Option<Square> {
        todo!("Implement find_king")
    }
    
    fn is_in_check(&self, _color: Color) -> bool {
        todo!("Implement is_in_check")
    }
    
    fn is_square_attacked(&self, _square: Square, _by_color: Color) -> bool {
        todo!("Implement is_square_attacked")
    }

    fn to_fen(&self) -> String { 
        todo!("Implement to_fen")
    }
    fn from_fen(fen: &str) -> Result<Self, String> where Self: Sized {
        todo!("Implement from_fen")
    }
}