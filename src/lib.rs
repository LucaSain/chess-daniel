use std::str::FromStr;

use arrayvec::ArrayVec;

mod piece;
pub use piece::*;

mod position;
pub use position::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Players {
    White = 1,
    Black = -1,
}
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
#[repr(align(8))]
pub enum Move {
    Normal {
        piece: Piece,
        start: Position,
        end: Position,
        captured_piece: Option<Piece>,
    },
    Promovation {
        // Assumed to alwasys be to queen
        owner: Players,
        start: Position,
        end: Position,
        captured_piece: Option<Piece>,
    },
    CastlingShort {
        owner: Players,
    },
    CastlingLong {
        owner: Players,
    },
    EnPassant {
        owner: Players,
        start_col: i8,
        end_col: i8,
    }, // No en passant
}

#[derive(Clone)]
// all fields are public for debugging
// TODO: remove pub
pub struct ChessGame {
    pub score: i32,
    pub current_player: Players,
    pub board: [[Option<Piece>; 8]; 8],
    king_positions: [Position; 2],
    has_castled: [bool; 2],
    pub move_stack: Vec<Move>,
    pub en_passant_stack: ArrayVec<i8, 256>,
}

impl Players {
    fn the_other(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl ChessGame {
    pub fn new() -> Self {
        #[rustfmt::skip]
        let mut game = ChessGame {
            board: [
                [
                    Some(Piece {piece_type: PieceTypes::Rook, owner: Players::White}),
                    Some(Piece {piece_type: PieceTypes::Knight, owner: Players::White}),
                    Some(Piece {piece_type: PieceTypes::Bishop, owner: Players::White}),
                    Some(Piece {piece_type: PieceTypes::Queen, owner: Players::White}),
                    Some(Piece {piece_type: PieceTypes::King, owner: Players::White}),
                    Some(Piece {piece_type: PieceTypes::Bishop, owner: Players::White}),
                    Some(Piece {piece_type: PieceTypes::Knight, owner: Players::White}),
                    Some(Piece {piece_type: PieceTypes::Rook, owner: Players::White}),
                ],
                [Some(Piece {piece_type: PieceTypes::Pawn, owner: Players::White}); 8],
                [None; 8],
                [None; 8],
                [None; 8],
                [None; 8],
                [Some(Piece {piece_type: PieceTypes::Pawn, owner: Players::Black}); 8],
                [
                    Some(Piece {piece_type: PieceTypes::Rook, owner: Players::Black}),
                    Some(Piece {piece_type: PieceTypes::Knight, owner: Players::Black}),
                    Some(Piece {piece_type: PieceTypes::Bishop, owner: Players::Black}),
                    Some(Piece {piece_type: PieceTypes::Queen, owner: Players::Black}),
                    Some(Piece {piece_type: PieceTypes::King, owner: Players::Black}),
                    Some(Piece {piece_type: PieceTypes::Bishop, owner: Players::Black}),
                    Some(Piece {piece_type: PieceTypes::Knight, owner: Players::Black}),
                    Some(Piece {piece_type: PieceTypes::Rook, owner: Players::Black}),
                ],
            ],
            move_stack: Vec::with_capacity(1000),
            has_castled: [false; 2],
            king_positions: [
                Position::new(0, 4).unwrap(),
                Position::new(7, 4).unwrap(),
            ],
            current_player: Players::White,
            score: 0,
            en_passant_stack: ArrayVec::new(),
        };
        game.en_passant_stack.push(-1);
        game
    }

    pub fn get_position(&self, position: Position) -> &Option<Piece> {
        // SAFETY: position is always valid
        unsafe {
            self.board
                .get_unchecked(position.row() as usize)
                .get_unchecked(position.col() as usize)
        }
    }

    // pub for debugging
    pub fn set_position(&mut self, position: Position, new_place: Option<Piece>) {
        // SAFETY: position is always valid
        unsafe {
            let place = self
                .board
                .get_unchecked_mut(position.row() as usize)
                .get_unchecked_mut(position.col() as usize);

            if let Some(piece) = place {
                self.score -= piece.score(position);
            }

            *place = new_place;

            if let Some(piece) = place {
                self.score += piece.score(position);
            }
        }
    }

    pub fn get_king_position(&self, player: Players) -> Position {
        // SAFETY: Hardcoded values are valid
        unsafe {
            match player {
                Players::White => *self.king_positions.get_unchecked(0),
                Players::Black => *self.king_positions.get_unchecked(1),
            }
        }
    }

    fn set_king_position(&mut self, player: Players, position: Position) {
        // SAFETY: Hardcoded values are valid
        unsafe {
            match player {
                Players::White => *self.king_positions.get_unchecked_mut(0) = position,
                Players::Black => *self.king_positions.get_unchecked_mut(1) = position,
            };
        }
    }

    pub fn get_castled(&self, player: Players) -> bool {
        // SAFETY: Hardcoded values are valid
        unsafe {
            match player {
                Players::White => *self.has_castled.get_unchecked(0),
                Players::Black => *self.has_castled.get_unchecked(1),
            }
        }
    }

    fn set_castled(&mut self, player: Players, value: bool) {
        // SAFETY: Hardcoded values are valid
        unsafe {
            match player {
                Players::White => *self.has_castled.get_unchecked_mut(0) = value,
                Players::Black => *self.has_castled.get_unchecked_mut(1) = value,
            }
        }
    }

    pub fn push_history(&mut self, _move: Move) {
        self.move_stack.push(_move);

        self.push(_move);
    }

    pub fn push(&mut self, _move: Move) {
        let mut valid_en_passant = -1;
        match _move {
            #[rustfmt::skip]
            Move::Normal { piece, start, end, .. } => {
                self.set_position(start, None);
                self.set_position(end, Some(piece));

                if piece.piece_type == PieceTypes::King {
                    self.set_king_position(self.current_player, end);
                }

                valid_en_passant =
                if piece.piece_type == PieceTypes::Pawn && i8::abs(end.row() - start.row()) == 2
                    { start.col() } else { -1 }
            }
            #[rustfmt::skip]
            Move::Promovation { owner, start, end, .. } => {
                self.set_position(start, None);
                self.set_position(
                    end,
                    Some(Piece {
                        owner,
                        piece_type: PieceTypes::Queen,
                    }),
                );
            }
            Move::EnPassant {
                owner,
                start_col,
                end_col,
            } => {
                // SAFETY: Theses are hardcoded valid positions
                unsafe {
                    let old_pawn = match owner {
                        Players::White => Position::new_unsafe(4, start_col),
                        Players::Black => Position::new_unsafe(3, start_col),
                    };
                    let new_pawn = match owner {
                        Players::White => Position::new_unsafe(5, end_col),
                        Players::Black => Position::new_unsafe(2, end_col),
                    };
                    let taken_pawn = match owner {
                        Players::White => Position::new_unsafe(4, end_col),
                        Players::Black => Position::new_unsafe(3, end_col),
                    };
                    self.set_position(taken_pawn, None);
                    self.set_position(old_pawn, None);
                    self.set_position(
                        new_pawn,
                        Some(Piece {
                            piece_type: PieceTypes::Pawn,
                            owner: owner,
                        }),
                    )
                }
            }
            Move::CastlingLong { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };
                // SAFETY: Theses are hardcoded valid positions
                let (old_king, new_king, old_rook, new_rook) = unsafe {
                    (
                        Position::new_unsafe(row, 4),
                        Position::new_unsafe(row, 2),
                        Position::new_unsafe(row, 0),
                        Position::new_unsafe(row, 3),
                    )
                };

                self.set_position(old_rook, None);
                self.set_position(old_king, None);
                self.set_position(
                    new_rook,
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                );

                self.set_position(
                    new_king,
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                );

                self.set_castled(self.current_player, true);
                self.set_king_position(self.current_player, new_king);
            }
            Move::CastlingShort { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };
                // SAFETY: Theses are hardcoded valid positions
                let (old_king, new_king, old_rook, new_rook) = unsafe {
                    (
                        Position::new_unsafe(row, 4),
                        Position::new_unsafe(row, 6),
                        Position::new_unsafe(row, 7),
                        Position::new_unsafe(row, 5),
                    )
                };

                self.set_position(old_rook, None);
                self.set_position(old_king, None);
                self.set_position(
                    new_rook,
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                );

                self.set_position(
                    new_king,
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                );

                self.set_castled(self.current_player, true);
                self.set_king_position(self.current_player, new_king);
            }
        };
        self.current_player = self.current_player.the_other();
        // SAFETY: The game should not be longer than 256 moves
        unsafe {
            self.en_passant_stack.push_unchecked(valid_en_passant);
        }
    }

    pub fn pop_history(&mut self) -> Option<Move> {
        let _move = self.move_stack.pop();
        _move.inspect(|_move| {
            self.pop(*_move);
        });

        _move
    }

    pub fn pop(&mut self, _move: Move) {
        self.en_passant_stack.pop();
        self.current_player = self.current_player.the_other();

        match _move {
            #[rustfmt::skip]
            Move::Normal { piece, start, end, captured_piece } => {
                self.set_position(start, Some(piece));
                self.set_position(end, captured_piece);

                if piece.piece_type == PieceTypes::King {
                    self.set_king_position(self.current_player, start);
                }
            }
            #[rustfmt::skip]
            Move::Promovation { owner, start, end, captured_piece } => {
                self.set_position(
                    start,
                    Some(Piece {
                        piece_type: PieceTypes::Pawn,
                        owner
                    }),
                );
                self.set_position(end, captured_piece);
            }
            Move::EnPassant {
                owner,
                start_col,
                end_col,
            } => {
                // SAFETY: Theses are hardcoded valid positions
                unsafe {
                    let old_pawn = match owner {
                        Players::White => Position::new_unsafe(4, start_col),
                        Players::Black => Position::new_unsafe(3, start_col),
                    };
                    let new_pawn = match owner {
                        Players::White => Position::new_unsafe(5, end_col),
                        Players::Black => Position::new_unsafe(2, end_col),
                    };
                    let taken_pawn = match owner {
                        Players::White => Position::new_unsafe(4, end_col),
                        Players::Black => Position::new_unsafe(3, end_col),
                    };
                    self.set_position(new_pawn, None);
                    self.set_position(
                        taken_pawn,
                        Some(Piece {
                            piece_type: PieceTypes::Pawn,
                            owner: owner.the_other(),
                        }),
                    );
                    self.set_position(
                        old_pawn,
                        Some(Piece {
                            piece_type: PieceTypes::Pawn,
                            owner: owner,
                        }),
                    );
                }
            }
            Move::CastlingLong { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };
                // SAFETY: Theses are hardcoded valid positions
                let (old_king, new_king, old_rook, new_rook) = unsafe {
                    (
                        Position::new_unsafe(row, 4),
                        Position::new_unsafe(row, 2),
                        Position::new_unsafe(row, 0),
                        Position::new_unsafe(row, 3),
                    )
                };

                self.set_position(new_rook, None);
                self.set_position(new_king, None);
                self.set_position(
                    old_rook,
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                );

                self.set_position(
                    old_king,
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                );

                self.set_castled(self.current_player, false);
                self.set_king_position(owner, old_king);
            }
            Move::CastlingShort { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };
                // SAFETY: Theses are hardcoded valid positions
                let (old_king, new_king, old_rook, new_rook) = unsafe {
                    (
                        Position::new_unsafe(row, 4),
                        Position::new_unsafe(row, 6),
                        Position::new_unsafe(row, 7),
                        Position::new_unsafe(row, 5),
                    )
                };

                self.set_position(new_rook, None);
                self.set_position(new_king, None);
                self.set_position(
                    old_rook,
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                );

                self.set_position(
                    old_king,
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                );

                self.set_castled(self.current_player, false);
                self.set_king_position(owner, old_king);
            }
        };
    }

    pub fn get_moves(&mut self, mut moves: &mut ArrayVec<Move, 128>) {
        let king_place = self.get_position(self.get_king_position(self.current_player));
        if !king_place.is_some_and(|piece| piece.piece_type == PieceTypes::King) {
            // no available moves;
            return;
        }

        for r in 0..8 {
            for c in 0..8 {
                // SAFETY: Theses are hardcoded valid positions
                unsafe {
                    if let Some(piece) = self.board.get_unchecked(r).get_unchecked(c) {
                        if piece.owner == self.current_player {
                            let pos = Position::new_unsafe(r as i8, c as i8);
                            piece.get_moves(&mut moves, self, pos);
                        }
                    }
                }
            }
        }
    }

    // Returns if player's position is targeted by enemy pieces
    pub fn is_targeted(&self, position: Position, player: Players) -> bool {
        // Verifiy for kings
        for delta in [
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
            (1, 1),
            (-1, 1),
            (1, -1),
            (-1, -1),
        ]
        .into_iter()
        {
            if let Some(new_pos) = position.add(delta) {
                if self.get_position(new_pos).is_some_and(|piece| {
                    piece.owner != player && piece.piece_type == PieceTypes::King
                }) {
                    return true;
                }
            }
        }

        // Verify for knights
        for delta in [
            (1, 2),
            (2, 1),
            (-1, -2),
            (-2, -1),
            (1, -2),
            (-2, 1),
            (-1, 2),
            (2, -1),
        ]
        .into_iter()
        {
            if let Some(new_pos) = position.add(delta) {
                if self.get_position(new_pos).is_some_and(|piece| {
                    piece.owner != player && piece.piece_type == PieceTypes::Knight
                }) {
                    return true;
                }
            }
        }

        // Verify for pawns
        match player {
            Players::White => {
                if let Some(new_pos) = position.add((1, 1)) {
                    if self.get_position(new_pos).is_some_and(|piece| {
                        piece.owner != player && piece.piece_type == PieceTypes::Pawn
                    }) {
                        return true;
                    }
                }
                if let Some(new_pos) = position.add((1, -1)) {
                    if self.get_position(new_pos).is_some_and(|piece| {
                        piece.owner != player && piece.piece_type == PieceTypes::Pawn
                    }) {
                        return true;
                    }
                }
            }
            Players::Black => {
                if let Some(new_pos) = position.add((-1, 1)) {
                    if self.get_position(new_pos).is_some_and(|piece| {
                        piece.owner != player && piece.piece_type == PieceTypes::Pawn
                    }) {
                        return true;
                    }
                }
                if let Some(new_pos) = position.add((-1, -1)) {
                    if self.get_position(new_pos).is_some_and(|piece| {
                        piece.owner != player && piece.piece_type == PieceTypes::Pawn
                    }) {
                        return true;
                    }
                }
            }
        };

        // Helpful macro
        macro_rules! search_enemies_loops {
            ( $piece_type1:expr, $piece_type2:expr, $( $x:expr ),* ) => {
                $(
                for delta in $x {
                    if let Some(new_pos) = position.add(delta) {
                        if let Some(piece) = self.get_position(new_pos)  {
                            if piece.owner != player &&
                                (piece.piece_type == $piece_type1 || piece.piece_type == $piece_type2) {
                                return true
                            }
                            break;
                        }
                    } else {
                        break;
                    }
                }
                )*
            };
        }

        // Verify lines for rooks/queens
        search_enemies_loops![
            PieceTypes::Rook,
            PieceTypes::Queen,
            (1..).map(|x| (0, x)),
            (1..).map(|x| (0, -x)),
            (1..).map(|x| (x, 0)),
            (1..).map(|x| (-x, 0))
        ];

        // Verify diagonals for bishops/queens
        search_enemies_loops![
            PieceTypes::Bishop,
            PieceTypes::Queen,
            (1..).map(|x| (x, x)),
            (1..).map(|x| (-x, -x)),
            (1..).map(|x| (x, -x)),
            (1..).map(|x| (-x, x))
        ];

        false
    }

    pub fn get_pgn(&self) -> String {
        let moves: Vec<_> = self
            .move_stack
            .iter()
            .map(|_move| match _move {
                Move::Normal {
                    piece,
                    start,
                    end,
                    captured_piece,
                } => {
                    let mut s = String::new();
                    s.push_str(piece.as_char_ascii());
                    s.push(((start.col()) as u8 + b'a') as char);
                    if captured_piece.is_some() {
                        s.push('x');
                    }
                    s.push(((end.col()) as u8 + b'a') as char);
                    s.push_str((end.row() + 1).to_string().as_str());
                    s
                }
                Move::CastlingShort { .. } => String::from_str("O-O").unwrap(),
                Move::CastlingLong { .. } => String::from_str("O-O-O").unwrap(),
                Move::EnPassant {
                    start_col,
                    end_col,
                    owner,
                } => {
                    let mut s = String::new();
                    s.push((*start_col as u8 + b'a') as char);
                    s.push('x');
                    s.push((*end_col as u8 + b'a') as char);
                    match owner {
                        Players::White => s.push('6'),
                        Players::Black => s.push('3'),
                    };
                    s
                }
                Move::Promovation {
                    end,
                    captured_piece,
                    ..
                } => {
                    let mut s = String::new();
                    if captured_piece.is_some() {
                        s.push('x');
                    }
                    s.push(((end.col()) as u8 + b'a') as char);
                    s.push_str((end.row() + 1).to_string().as_str());
                    s.push('=');
                    s.push('Q');
                    s
                }
            })
            .collect();

        let mut s = String::new();

        for (i, _move) in moves.iter().enumerate() {
            if i % 2 == 0 {
                s.push_str((i / 2 + 1).to_string().as_str());
                s.push('.');
                s.push(' ');
            }
            s.push_str(_move.as_str());
            s.push(' ');
        }

        s
    }
}

impl std::fmt::Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Move::Normal {
                piece,
                start,
                end,
                captured_piece,
            } => write!(
                f,
                "{:?} {:?} from {} {} to {} {}, captured {:?} ",
                piece.owner,
                piece.piece_type,
                start.row(),
                start.col(),
                end.row(),
                end.col(),
                captured_piece.map(|piece| format!("{:?} {:?}", piece.owner, piece.piece_type))
            ),
            _ => write!(f, "not supported"),
        }
    }
}

impl std::fmt::Debug for ChessGame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "\n")?;
        self.board
            .iter()
            .enumerate()
            .rev()
            .try_for_each(|(i, row)| -> std::fmt::Result {
                write!(f, "{} ", i)?;
                row.iter().try_for_each(|place| -> std::fmt::Result {
                    write!(f, "|{}", place.map(|piece| piece.as_char()).unwrap_or(' '))
                })?;
                write!(f, "|\n")
            })?;
        write!(f, "\n   0 1 2 3 4 5 6 7\n")
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
