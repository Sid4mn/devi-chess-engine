use crate::types::*;

pub struct Board {
    squares: [Option<Piece>; 64],
    to_move: Color,
    castling_rights: u8,
    pub en_passant: Option<Square>,
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

    pub fn setup_starting_position(&mut self) {
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
}