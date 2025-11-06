use super::traits::{BoardRepresentation, UndoMove};
use crate::types::*;
use crate::types::{BK, BQ, WK, WQ};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ArrayBoard {
    #[serde(with = "BigArray")]
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
        self.squares = [None; 64];
        self.to_move = Color::White;
        self.castling_rights = 0b1111;
        self.en_passant = None;
        self.halfmove_clock = 0;
        self.fullmove_clock = 1;

        self.set_piece(Square(0), Some(Piece::new(PieceType::Rook, Color::White))); //a1
        self.set_piece(Square(1), Some(Piece::new(PieceType::Knight, Color::White))); //b1
        self.set_piece(Square(2), Some(Piece::new(PieceType::Bishop, Color::White))); //c1
        self.set_piece(Square(3), Some(Piece::new(PieceType::Queen, Color::White))); //d1
        self.set_piece(Square(4), Some(Piece::new(PieceType::King, Color::White))); //e1
        self.set_piece(Square(5), Some(Piece::new(PieceType::Bishop, Color::White))); //f1
        self.set_piece(Square(6), Some(Piece::new(PieceType::Knight, Color::White))); //g1
        self.set_piece(Square(7), Some(Piece::new(PieceType::Rook, Color::White))); //h1

        for i in 8..16 {
            // 2
            self.set_piece(Square(i), Some(Piece::new(PieceType::Pawn, Color::White)));
        }

        for i in 48..56 {
            // 7
            self.set_piece(Square(i), Some(Piece::new(PieceType::Pawn, Color::Black)));
        }

        self.set_piece(Square(56), Some(Piece::new(PieceType::Rook, Color::Black))); //a8
        self.set_piece(
            Square(57),
            Some(Piece::new(PieceType::Knight, Color::Black)),
        ); //b8
        self.set_piece(
            Square(58),
            Some(Piece::new(PieceType::Bishop, Color::Black)),
        ); //c8
        self.set_piece(Square(59), Some(Piece::new(PieceType::Queen, Color::Black))); //d8
        self.set_piece(Square(60), Some(Piece::new(PieceType::King, Color::Black))); //e8
        self.set_piece(
            Square(61),
            Some(Piece::new(PieceType::Bishop, Color::Black)),
        ); //f8
        self.set_piece(
            Square(62),
            Some(Piece::new(PieceType::Knight, Color::Black)),
        ); //g8
        self.set_piece(Square(63), Some(Piece::new(PieceType::Rook, Color::Black)));
        //h8
    }

    fn clear(&mut self) {
        self.squares = [None; 64];
    }

    fn make_move(&mut self, _mv: &Move) -> UndoMove {
        let undo = UndoMove {
            captured_piece: self.get_piece(_mv.to),
            previous_en_passant: self.en_passant,
            previous_castling_rights: self.castling_rights,
            previous_halfmove_clock: self.halfmove_clock,
            previous_fullmove_clock: self.fullmove_clock,
            previous_to_move: self.to_move,
        };

        let moving_piece = self.get_piece(_mv.from).expect("No piece at square");

        self.set_piece(_mv.from, None);

        let piece_to_place = if let Some(SpecialMove::Promotion) = &_mv.special_move {
            if let Some(promoted_piece_type) = _mv.promotion {
                Piece::new(promoted_piece_type, moving_piece.color)
            } else {
                panic!("Promotion move without promotion piece type!");
            }
        } else {
            moving_piece
        };

        self.set_piece(_mv.to, Some(piece_to_place));

        // Clear en passant square (will be set later if applicable)
        self.en_passant = None;

        if let Some(special) = &_mv.special_move {
            match special {
                SpecialMove::EnPassant => {
                    let captured_pawn_square = match moving_piece.color {
                        Color::White => Square(_mv.to.0 - 8),
                        Color::Black => Square(_mv.to.0 + 8),
                    };
                    self.set_piece(captured_pawn_square, None);
                }

                SpecialMove::Castle => {
                    let king_side = _mv.to.0 > _mv.from.0;
                    let (rook_from, rook_to) = match (moving_piece.color, king_side) {
                        (Color::White, true) => (Square(7), Square(5)),
                        (Color::White, false) => (Square(0), Square(3)),
                        (Color::Black, true) => (Square(63), Square(61)),
                        (Color::Black, false) => (Square(56), Square(59)),
                    };
                    if let Some(rook) = self.get_piece(rook_from) {
                        self.set_piece(rook_from, None);
                        self.set_piece(rook_to, Some(rook));
                    } else {
                        debug_assert!(false, "rook missing during castling");
                    }
                }

                SpecialMove::Promotion => {
                    // Already handled above
                }
            }
        }

        // Update castling rights
        // Clear if capturing on rook squares
        if undo.captured_piece.is_some() {
            match _mv.to {
                Square(0) => self.castling_rights &= !(WQ),
                Square(7) => self.castling_rights &= !(WK),
                Square(56) => self.castling_rights &= !(BQ),
                Square(63) => self.castling_rights &= !(BK),
                _ => {}
            }
        }

        // Clear if moving from rook squares
        match _mv.from {
            Square(0) => self.castling_rights &= !(WQ),
            Square(7) => self.castling_rights &= !(WK),
            Square(56) => self.castling_rights &= !(BQ),
            Square(63) => self.castling_rights &= !(BK),
            _ => {}
        }

        // Clear king castling rights when king moves
        if moving_piece.piece_type == PieceType::King {
            match moving_piece.color {
                Color::White => self.castling_rights &= !(WK | WQ),
                Color::Black => self.castling_rights &= !(BK | BQ),
            }
        }

        // Set en passant square if pawn moved two squares
        if moving_piece.piece_type == PieceType::Pawn {
            let move_distance = (_mv.to.0 as i8 - _mv.from.0 as i8).abs();
            if move_distance == 16 {
                // ALWAYS set en passant square after double pawn push
                let en_passant_square = Square((_mv.from.0 + _mv.to.0) / 2);
                self.en_passant = Some(en_passant_square);
            }
        }

        // Update halfmove clock
        if moving_piece.piece_type == PieceType::Pawn || undo.captured_piece.is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // Toggle side to move
        self.to_move = match self.to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        // Update fullmove clock
        if self.to_move == Color::White {
            self.fullmove_clock += 1;
        }

        undo
    }

    fn unmake_move(&mut self, _mv: &Move, _undo: UndoMove) {
        let piece_at_dest = self.get_piece(_mv.to).expect("No piece at destination");

        let piece_to_restore = if let Some(SpecialMove::Promotion) = &_mv.special_move {
            Piece::new(PieceType::Pawn, piece_at_dest.color)
        } else {
            piece_at_dest
        };

        self.set_piece(_mv.from, Some(piece_to_restore));
        self.set_piece(_mv.to, _undo.captured_piece);

        if let Some(special) = &_mv.special_move {
            match special {
                SpecialMove::EnPassant => {
                    let captured_pawn_square = match piece_to_restore.color {
                        Color::White => Square(_mv.to.0 - 8),
                        Color::Black => Square(_mv.to.0 + 8),
                    };
                    let captured_pawn = Piece {
                        piece_type: PieceType::Pawn,
                        color: match piece_to_restore.color {
                            Color::White => Color::Black,
                            Color::Black => Color::White,
                        },
                    };
                    self.set_piece(captured_pawn_square, Some(captured_pawn));
                }
                SpecialMove::Castle => {
                    let king_side = _mv.to.0 > _mv.from.0;
                    let (rook_from, rook_to) = match (piece_to_restore.color, king_side) {
                        (Color::White, true) => (Square(7), Square(5)),
                        (Color::White, false) => (Square(0), Square(3)),
                        (Color::Black, true) => (Square(63), Square(61)),
                        (Color::Black, false) => (Square(56), Square(59)),
                    };
                    if let Some(rook) = self.get_piece(rook_to) {
                        self.set_piece(rook_to, None);
                        self.set_piece(rook_from, Some(rook));
                    } else {
                        debug_assert!(false, "rook missing during unmake of castling");
                    }
                }
                SpecialMove::Promotion => {
                    // Already handled by restoring a pawn at the from square
                }
            }
        }

        self.to_move = _undo.previous_to_move;
        self.en_passant = _undo.previous_en_passant;
        self.castling_rights = _undo.previous_castling_rights;
        self.halfmove_clock = _undo.previous_halfmove_clock;
        self.fullmove_clock = _undo.previous_fullmove_clock;
    }

    fn find_king(&self, _color: Color) -> Option<Square> {
        for square_idx in 0..64 {
            if self.get_piece(Square(square_idx as u8))
                == Some(Piece {
                    piece_type: PieceType::King,
                    color: _color,
                })
            {
                return Some(Square(square_idx as u8));
            }
        }
        None
    }

    fn is_in_check(&self, _color: Color) -> bool {
        let king_location = match self.find_king(_color) {
            Some(square) => square,
            None => return false,
        };

        let opponent_color = if _color == Color::White {
            Color::Black
        } else {
            Color::White
        };

        // Check if any opponent piece can attack the king square
        // We need to check each piece type separately to avoid recursion

        // Check for pawn attacks
        let pawn_offsets = match _color {
            Color::White => vec![7, 9],   // Black pawns attack from above
            Color::Black => vec![-7, -9], // White pawns attack from below
        };

        for offset in pawn_offsets {
            let attack_square = king_location.0 as i8 + offset;
            if attack_square >= 0 && attack_square < 64 {
                let file_diff = ((king_location.0 % 8) as i8 - (attack_square % 8)).abs();
                if file_diff == 1 {
                    if let Some(piece) = self.get_piece(Square(attack_square as u8)) {
                        if piece.piece_type == PieceType::Pawn && piece.color == opponent_color {
                            return true;
                        }
                    }
                }
            }
        }

        // Check for knight attacks
        let knight_offsets = [17, 15, 10, 6, -6, -10, -15, -17];
        for offset in knight_offsets {
            let attack_square = king_location.0 as i8 + offset;
            if attack_square >= 0 && attack_square < 64 {
                let from_file = (king_location.0 % 8) as i8;
                let from_rank = (king_location.0 / 8) as i8;
                let to_file = (attack_square % 8) as i8;
                let to_rank = (attack_square / 8) as i8;

                if (from_file - to_file).abs() <= 2 && (from_rank - to_rank).abs() <= 2 {
                    if let Some(piece) = self.get_piece(Square(attack_square as u8)) {
                        if piece.piece_type == PieceType::Knight && piece.color == opponent_color {
                            return true;
                        }
                    }
                }
            }
        }

        // Check for sliding pieces (rook, bishop, queen)
        // Check orthogonal directions (rook/queen)
        let orthogonal_dirs = [1, -1, 8, -8];
        for dir in orthogonal_dirs {
            let mut pos = king_location.0 as i8;
            let file = pos % 8;

            loop {
                pos += dir;
                if pos < 0 || pos >= 64 {
                    break;
                }

                let new_file = pos % 8;

                if dir == 1 && new_file <= file {
                    break;
                }
                if dir == -1 && new_file >= file {
                    break;
                }

                if let Some(piece) = self.get_piece(Square(pos as u8)) {
                    if piece.color == opponent_color {
                        if piece.piece_type == PieceType::Rook
                            || piece.piece_type == PieceType::Queen
                        {
                            return true;
                        }
                    }
                    break;
                }
            }
        }

        // Check diagonal directions (bishop/queen)
        let diagonal_dirs = [7, -7, 9, -9];
        for dir in diagonal_dirs {
            let mut pos = king_location.0 as i8;
            let file = pos % 8;
            let rank = pos / 8;

            loop {
                pos += dir;
                if pos < 0 || pos >= 64 {
                    break;
                }

                let new_file = pos % 8;
                let new_rank = pos / 8;

                if (file - new_file).abs() != (rank - new_rank).abs() {
                    break;
                }

                if let Some(piece) = self.get_piece(Square(pos as u8)) {
                    if piece.color == opponent_color {
                        if piece.piece_type == PieceType::Bishop
                            || piece.piece_type == PieceType::Queen
                        {
                            return true;
                        }
                    }
                    break;
                }
            }
        }

        let king_offsets = [1, -1, 7, -7, 8, -8, 9, -9];
        for offset in king_offsets {
            let check_pos = king_location.0 as i8 + offset;
            if check_pos >= 0 && check_pos < 64 {
                let from_file = (king_location.0 % 8) as i8;
                let from_rank = (king_location.0 / 8) as i8;
                let to_file = (check_pos % 8) as i8;
                let to_rank = (check_pos / 8) as i8;

                if (from_file - to_file).abs() <= 1 && (from_rank - to_rank).abs() <= 1 {
                    if let Some(piece) = self.get_piece(Square(check_pos as u8)) {
                        if piece.piece_type == PieceType::King && piece.color == opponent_color {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        let pawn_offsets = match by_color {
            Color::White => vec![-7, -9],
            Color::Black => vec![7, 9],
        };

        for offset in pawn_offsets {
            let attack_from = square.0 as i8 - offset;
            if attack_from >= 0 && attack_from < 64 {
                let file_diff = ((square.0 % 8) as i8 - (attack_from % 8)).abs();
                if file_diff == 1 {
                    if let Some(piece) = self.get_piece(Square(attack_from as u8)) {
                        if piece.piece_type == PieceType::Pawn && piece.color == by_color {
                            return true;
                        }
                    }
                }
            }
        }

        // Check for knight attacks
        let knight_offsets = [17, 15, 10, 6, -6, -10, -15, -17];
        for offset in knight_offsets {
            let attack_from = square.0 as i8 - offset;
            if attack_from >= 0 && attack_from < 64 {
                let from_file = (attack_from % 8) as i8;
                let from_rank = (attack_from / 8) as i8;
                let to_file = (square.0 % 8) as i8;
                let to_rank = (square.0 / 8) as i8;

                if (from_file - to_file).abs() <= 2 && (from_rank - to_rank).abs() <= 2 {
                    if let Some(piece) = self.get_piece(Square(attack_from as u8)) {
                        if piece.piece_type == PieceType::Knight && piece.color == by_color {
                            return true;
                        }
                    }
                }
            }
        }

        // Check for sliding pieces (rook, bishop, queen)
        // Check orthogonal directions (rook/queen)
        let orthogonal_dirs = [1, -1, 8, -8];
        for dir in orthogonal_dirs {
            let mut pos = square.0 as i8;
            let file = pos % 8;

            loop {
                pos += dir;
                if pos < 0 || pos >= 64 {
                    break;
                }

                let new_file = pos % 8;

                if dir == 1 && new_file <= file {
                    break;
                }
                if dir == -1 && new_file >= file {
                    break;
                }

                if let Some(piece) = self.get_piece(Square(pos as u8)) {
                    if piece.color == by_color {
                        if piece.piece_type == PieceType::Rook
                            || piece.piece_type == PieceType::Queen
                        {
                            return true;
                        }
                    }
                    break;
                }
            }
        }

        // Check diagonal directions (bishop/queen)
        let diagonal_dirs = [7, -7, 9, -9];
        for dir in diagonal_dirs {
            let mut pos = square.0 as i8;
            let file = pos % 8;
            let rank = pos / 8;

            loop {
                pos += dir;
                if pos < 0 || pos >= 64 {
                    break;
                }

                let new_file = pos % 8;
                let new_rank = pos / 8;

                if (file - new_file).abs() != (rank - new_rank).abs() {
                    break;
                }

                if let Some(piece) = self.get_piece(Square(pos as u8)) {
                    if piece.color == by_color {
                        if piece.piece_type == PieceType::Bishop
                            || piece.piece_type == PieceType::Queen
                        {
                            return true;
                        }
                    }
                    break;
                }
            }
        }

        let king_offsets = [1, -1, 7, -7, 8, -8, 9, -9];
        for offset in king_offsets {
            let check_pos = square.0 as i8 + offset;
            if check_pos >= 0 && check_pos < 64 {
                let from_file = (square.0 % 8) as i8;
                let from_rank = (square.0 / 8) as i8;
                let to_file = (check_pos % 8) as i8;
                let to_rank = (check_pos / 8) as i8;

                if (from_file - to_file).abs() <= 1 && (from_rank - to_rank).abs() <= 1 {
                    if let Some(piece) = self.get_piece(Square(check_pos as u8)) {
                        if piece.piece_type == PieceType::King && piece.color == by_color {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn to_fen(&self) -> String {
        todo!("Implement to_fen")
    }
    fn from_fen(_fen: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        todo!("Implement from_fen")
    }
}
