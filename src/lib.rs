#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Players {
    White,
    Black,
}

#[derive(Clone, Copy)]
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
    // No en passant
}

#[derive(Clone)]
// all fields are public for debugging
// TODO: remove pub
pub struct ChessGame {
    pub board: [[Option<Piece>; 8]; 8],
    pub move_stack: Vec<Move>, // debug
    pub current_player: Players,
    pub has_castled: [bool; 2],
    pub king_positions: [Position; 2], // for finding if it is in check
    pub score: f64,
}

use arrayvec::ArrayVec;

mod piece;
pub use piece::*;

mod position;
pub use position::*;

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
        let game = ChessGame {
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
            move_stack: Vec::with_capacity(100),
            has_castled: [false; 2],
            king_positions: [
                Position::new(0, 4).unwrap(),
                Position::new(7, 4).unwrap(),
            ],
            current_player: Players::White,
            score: 0.0,
        };

        game
    }

    pub fn get_position(&self, position: Position) -> &Option<Piece> {
        // position is always valid
        unsafe {
            self.board
                .get_unchecked(position.row() as usize)
                .get_unchecked(position.col() as usize)
        }
    }

    // pub for debugging
    pub fn set_position(
        &mut self,
        position: Position,
        new_place: Option<Piece>,
        update_score: bool,
    ) {
        // position is always valid
        unsafe {
            let place = self
                .board
                .get_unchecked_mut(position.row() as usize)
                .get_unchecked_mut(position.col() as usize);
            if update_score {
                self.score -= place.map(|piece| piece.score(position)).unwrap_or_default();
                *place = new_place;
                self.score += place.map(|piece| piece.score(position)).unwrap_or_default();
            } else {
                *place = new_place
            }
        }
    }

    fn real_push(&mut self, _move: Move, update_score: bool) {
        self.move_stack.push(_move);

        match _move {
            #[rustfmt::skip]
            Move::Normal { piece, start, end, .. } => {
                self.set_position(start, None, update_score);
                self.set_position(end, Some(piece), update_score);

                if piece.piece_type == PieceTypes::King {
                    self.king_positions[self.current_player as usize] = end;
                }
            }
            #[rustfmt::skip]
            Move::Promovation { owner, start, end, .. } => {
                self.set_position(start, None, update_score);
                self.set_position(
                    end,
                    Some(Piece {
                        owner,
                        piece_type: PieceTypes::Queen,
                    }),
                    update_score,
                );
            }
            Move::CastlingLong { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };

                let (old_king, new_king, old_rook, new_rook) = unsafe {
                    (
                        Position::new_unsafe(row, 4),
                        Position::new_unsafe(row, 2),
                        Position::new_unsafe(row, 0),
                        Position::new_unsafe(row, 3),
                    )
                };

                self.set_position(old_rook, None, update_score);
                self.set_position(old_king, None, update_score);
                self.set_position(
                    new_rook,
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                    update_score,
                );

                self.set_position(
                    new_king,
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                    update_score,
                );

                self.has_castled[self.current_player as usize] = true;
                self.king_positions[self.current_player as usize] = new_king;
            }
            Move::CastlingShort { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };

                let (old_king, new_king, old_rook, new_rook) = unsafe {
                    (
                        Position::new_unsafe(row, 4),
                        Position::new_unsafe(row, 6),
                        Position::new_unsafe(row, 7),
                        Position::new_unsafe(row, 5),
                    )
                };

                self.set_position(old_rook, None, update_score);
                self.set_position(old_king, None, update_score);
                self.set_position(
                    new_rook,
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                    update_score,
                );

                self.set_position(
                    new_king,
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                    update_score,
                );

                self.has_castled[self.current_player as usize] = true;
                self.king_positions[self.current_player as usize] = new_king;
            }
        };

        self.current_player = self.current_player.the_other();
    }

    fn real_pop(&mut self, update_score: bool) -> Move {
        let _move = self.move_stack.pop().expect("Tried to pop a new game");
        self.current_player = self.current_player.the_other();

        match _move {
            #[rustfmt::skip]
            Move::Normal { piece, start, end, captured_piece } => {
                self.set_position(start, Some(piece), update_score);
                self.set_position(end, captured_piece, update_score);

                if piece.piece_type == PieceTypes::King {
                    self.king_positions[self.current_player as usize] = start;
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
                    update_score,
                );
                self.set_position(end, captured_piece, update_score);
            }
            Move::CastlingLong { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };

                let (old_king, new_king, old_rook, new_rook) = unsafe {
                    (
                        Position::new_unsafe(row, 4),
                        Position::new_unsafe(row, 2),
                        Position::new_unsafe(row, 0),
                        Position::new_unsafe(row, 3),
                    )
                };

                self.set_position(new_rook, None, update_score);
                self.set_position(new_king, None, update_score);
                self.set_position(
                    old_rook,
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                    update_score,
                );

                self.set_position(
                    old_king,
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                    update_score,
                );

                self.has_castled[owner as usize] = false;
                self.king_positions[owner as usize] = old_king;
            }
            Move::CastlingShort { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };

                let (old_king, new_king, old_rook, new_rook) = unsafe {
                    (
                        Position::new_unsafe(row, 4),
                        Position::new_unsafe(row, 6),
                        Position::new_unsafe(row, 7),
                        Position::new_unsafe(row, 5),
                    )
                };

                self.set_position(new_rook, None, update_score);
                self.set_position(new_king, None, update_score);
                self.set_position(
                    old_rook,
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                    update_score,
                );

                self.set_position(
                    old_king,
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                    update_score,
                );

                self.has_castled[owner as usize] = false;
                self.king_positions[owner as usize] = old_king;
            }
        };

        _move
    }

    pub fn push(&mut self, _move: Move) {
        self.real_push(_move, true);
    }

    pub fn pop(&mut self) {
        self.real_pop(true);
    }

    pub fn get_moves(&mut self) -> ArrayVec<Move, 128> {
        let piece_moves: ArrayVec<ArrayVec<Move, 27>, 32> = self
            .board
            .iter()
            .enumerate()
            .flat_map(|(r, v)| {
                v.iter()
                    .enumerate()
                    .map(move |(c, v)| (unsafe { Position::new_unsafe(r as i8, c as i8) }, v))
            })
            .filter_map(|(position, place)| {
                place
                    .filter(|piece| piece.owner == self.current_player)
                    .map(|_| (position, place))
            })
            .map(|(position, place)| {
                place
                    .map(|piece| piece.get_moves(self, position))
                    .unwrap_or_default()
            })
            .collect();

        // Only select moves which don't put king in check
        let mut moves = ArrayVec::new();
        let player = self.current_player;
        for list in piece_moves {
            for item in list.iter() {
                self.real_push(*item, false);
                if !self.is_targeted(self.king_positions[player as usize]) {
                    unsafe {
                        moves.push_unchecked(*item);
                    }
                }
                self.real_pop(false);
            }
        }

        moves
    }

    // Returns if piece is targeted by enemy pieces
    pub fn is_targeted(&self, position: Position) -> bool {
        // This function should only be called with valid pieces
        let player = self.get_position(position).unwrap().owner;

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
