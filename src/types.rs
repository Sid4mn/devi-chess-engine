use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialMove {
    Castle,
    Promotion,
    EnPassant,
}

pub type CastleMask = u8;

pub const WK: CastleMask = 1 << 0;
pub const WQ: CastleMask = 1 << 1;
pub const BK: CastleMask = 1 << 2;
pub const BQ: CastleMask = 1 << 3;

pub const ALL_CASTLING: CastleMask = WK | WQ | BK | BQ;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    pub fn new(p_type: PieceType, color: Color) -> Piece {
        Piece {
            piece_type: p_type,
            color: color,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square(pub u8); // 0-63

impl Square {
    pub fn new(val: u8) -> Self {
        debug_assert!(val < 64);
        Square(val)
    }

    pub fn file(self) -> char {
        (b'a' + (self.0 % 8)) as char
    }

    pub fn rank(self) -> char {
        (b'1' + (self.0 / 8)) as char
    }

    pub fn to_alg(self) -> String {
        let mut s = String::with_capacity(2);
        s.push(self.file());
        s.push(self.rank());
        s
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub special_move: Option<SpecialMove>,
    pub promotion: Option<PieceType>,
}

impl Move {
    pub fn new(
        _from: Square,
        _to: Square,
        special: Option<SpecialMove>,
        _promo: Option<PieceType>,
    ) -> Self {
        Move {
            from: _from,
            to: _to,
            special_move: special,
            promotion: _promo,
        }
    }

    pub fn to_algebraic(&self) -> String {
        let from_file = (self.from.0 % 8) as u8 + b'a';
        let from_rank = (self.from.0 / 8) as u8 + b'1';
        let to_file = (self.to.0 % 8) as u8 + b'a';
        let to_rank = (self.to.0 / 8) as u8 + b'1';

        let mut result = format!(
            "{}{}{}{}",
            from_file as char, from_rank as char, to_file as char, to_rank as char
        );

        if let Some(promotion) = self.promotion {
            result.push(match promotion {
                PieceType::Queen => 'q',
                PieceType::Rook => 'r',
                PieceType::Bishop => 'b',
                PieceType::Knight => 'n',
                _ => '?',
            });
        }

        result
    }
}
