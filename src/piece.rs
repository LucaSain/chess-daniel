use arrayvec::ArrayVec;

use super::{ChessGame, Move, Players, Position};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum PieceTypes {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub piece_type: PieceTypes,
    pub owner: Players,
}

macro_rules! push {
    ( $moves:ident, $move:expr ) => {
        // SAFETY: The number of possible moves on the board at any given time
        // should never exceed the arrays capacity (256)
        #[allow(unused_unsafe)]
        unsafe {
            $moves.push_unchecked($move);
        }
    };
}

macro_rules! find_moves_loops {
    ( $moves:ident, $pos:ident, $game:ident, $piece_type:ident, $( $x:expr ),* ) => {
        $(
        for delta in $x {
            if let Some(new_pos) = $pos.add(delta) {
                let place = $game.get_position(new_pos);
                let _move = Move::Normal {
                    piece: *$piece_type,
                    start: $pos,
                    end: new_pos,
                    captured_piece: *place,
                };

                if let Some(piece) = place  {
                    if piece.owner != $game.current_player {
                        push!($moves, _move);
                    }
                    break;
                }

                push!($moves, _move);
            } else {
                break;
            }
        }
        )*
    };
}

impl Piece {
    pub fn score(&self, pos: Position) -> i32 {
        // SAFETY: Position is always valid
        unsafe {
            let piece_score = *match self.piece_type {
                PieceTypes::Pawn => &[0, 90, 100, 115, 118, 120, 123, 130],
                PieceTypes::Knight => &[300, 300, 330, 331, 333, 335, 335, 340],
                PieceTypes::Bishop => &[310, 310, 315, 323, 334, 338, 339, 340],
                PieceTypes::Rook => &[500, 500, 510, 510, 510, 510, 520, 530],
                PieceTypes::Queen => &[900, 905, 910, 920, 920, 920, 920, 920],
                PieceTypes::King => &[100000, 99960, 99950, 99950, 99950, 99950, 99950, 99950],
            }
            .get_unchecked(match self.owner {
                Players::White => pos.row(),
                Players::Black => 7 - pos.row(),
            } as usize);
            let col_score = match self.piece_type {
                PieceTypes::King => 100,
                _ => *[96, 97, 98, 100, 100, 98, 97, 96].get_unchecked(pos.col() as usize),
            };
            match self.owner {
                Players::White => piece_score * col_score,
                Players::Black => -piece_score * col_score,
            }
        }
    }

    pub fn get_moves(&self, moves: &mut ArrayVec<Move, 128>, game: &ChessGame, pos: Position) {
        match self.piece_type {
            PieceTypes::Pawn => {
                let first_row = match self.owner {
                    Players::White => 1,
                    Players::Black => 6,
                };

                let last_row = match self.owner {
                    Players::White => 7,
                    Players::Black => 0,
                };

                let normal_delta = match self.owner {
                    Players::White => (1, 0),
                    Players::Black => (-1, 0),
                };

                let first_row_delta = match self.owner {
                    Players::White => (2, 0),
                    Players::Black => (-2, 0),
                };

                // SAFETY: First moves for pawns always exist
                unsafe {
                    if pos.row() == first_row
                        && game.get_position(pos.add_unsafe(normal_delta)).is_none()
                        && game.get_position(pos.add_unsafe(first_row_delta)).is_none()
                    {
                        push!(
                            moves,
                            Move::Normal {
                                piece: *self,
                                start: pos,
                                end: pos.add_unsafe(first_row_delta),
                                captured_piece: None,
                            }
                        );
                    }
                }

                let side_deltas = match self.owner {
                    Players::White => [(1, 1), (1, -1)],
                    Players::Black => [(-1, 1), (-1, -1)],
                };

                if let Some(new_pos) = pos.add(normal_delta) {
                    if game.get_position(new_pos).is_none() {
                        let _move = if last_row == new_pos.row() {
                            Move::Promovation {
                                owner: game.current_player,
                                start: pos,
                                end: new_pos,
                                captured_piece: None,
                            }
                        } else {
                            Move::Normal {
                                piece: *self,
                                start: pos,
                                end: new_pos,
                                captured_piece: None,
                            }
                        };
                        push!(moves, _move);
                    }
                }

                for delta in side_deltas {
                    if let Some(new_pos) = pos.add(delta) {
                        let place = game.get_position(new_pos);
                        if place.is_some_and(|piece| piece.owner != self.owner) {
                            let _move = if last_row == new_pos.row() {
                                Move::Promovation {
                                    owner: game.current_player,
                                    start: pos,
                                    end: new_pos,
                                    captured_piece: *place,
                                }
                            } else {
                                Move::Normal {
                                    piece: *self,
                                    start: pos,
                                    end: new_pos,
                                    captured_piece: *place,
                                }
                            };
                            push!(moves, _move);
                        }
                    }
                }
                // TODO: En passant
            }
            PieceTypes::King => {
                let other_king_pos = game.get_king_position(game.current_player.the_other());
                for delta in [
                    (0, 1),
                    (0, -1),
                    (1, 0),
                    (-1, 0),
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                ]
                .into_iter()
                {
                    if let Some(new_pos) = pos.add(delta) {
                        let place = game.get_position(new_pos);
                        if !place.is_some_and(|piece| piece.owner == game.current_player) {
                            // Kings can't move into each other
                            if i8::abs(new_pos.row() - other_king_pos.row()) <= 1
                                && i8::abs(new_pos.col() - other_king_pos.col()) <= 1
                            {
                                continue;
                            }
                            push!(
                                moves,
                                Move::Normal {
                                    piece: *self,
                                    start: pos,
                                    end: new_pos,
                                    captured_piece: *place,
                                }
                            );
                        }
                    }
                }

                if !game.has_castled[game.current_player as usize] {
                    let row = match game.current_player {
                        Players::White => 0,
                        Players::Black => 7,
                    };
                    // SAFETY: Theses are hardcoded valid positions
                    let (king, rook_1, rook_2) = unsafe {
                        (
                            Position::new_unsafe(row, 4),
                            Position::new_unsafe(row, 7),
                            Position::new_unsafe(row, 0),
                        )
                    };
                    // SAFETY: Theses are hardcoded valid positions
                    let pos_short =
                        unsafe { [Position::new_unsafe(row, 5), Position::new_unsafe(row, 6)] };
                    // TODO make sure those empty squares are not targeted
                    if pos_short
                        .iter()
                        .all(|pos| game.get_position(*pos).is_none())
                        && game.get_position(king)
                            == &Some(Piece {
                                piece_type: PieceTypes::King,
                                owner: game.current_player,
                            })
                        && game.get_position(rook_1)
                            == &Some(Piece {
                                piece_type: PieceTypes::Rook,
                                owner: game.current_player,
                            })
                    {
                        push!(
                            moves,
                            Move::CastlingShort {
                                owner: game.current_player,
                            }
                        );
                    }
                    // SAFETY: Theses are hardcoded valid positions
                    let pos_long = unsafe {
                        [
                            Position::new_unsafe(row, 1),
                            Position::new_unsafe(row, 2),
                            Position::new_unsafe(row, 3),
                        ]
                    };

                    if pos_long.iter().all(|pos| game.get_position(*pos).is_none())
                        && game.get_position(king)
                            == &Some(Piece {
                                piece_type: PieceTypes::King,
                                owner: game.current_player,
                            })
                        && game.get_position(rook_2)
                            == &Some(Piece {
                                piece_type: PieceTypes::Rook,
                                owner: game.current_player,
                            })
                    {
                        push!(
                            moves,
                            Move::CastlingLong {
                                owner: game.current_player,
                            }
                        );
                    }
                }
            }
            PieceTypes::Knight => {
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
                    if let Some(new_pos) = pos.add(delta) {
                        let place = game.get_position(new_pos);
                        if !place.is_some_and(|piece| piece.owner == game.current_player) {
                            push!(
                                moves,
                                Move::Normal {
                                    piece: *self,
                                    start: pos,
                                    end: new_pos,
                                    captured_piece: *place,
                                }
                            );
                        }
                    }
                }
            }
            PieceTypes::Rook => {
                find_moves_loops![
                    moves,
                    pos,
                    game,
                    self,
                    (1..).map(|x| (0, x)),
                    (1..).map(|x| (0, -x)),
                    (1..).map(|x| (x, 0)),
                    (1..).map(|x| (-x, 0))
                ];
            }
            PieceTypes::Bishop => {
                find_moves_loops![
                    moves,
                    pos,
                    game,
                    self,
                    (1..).map(|x| (x, x)),
                    (1..).map(|x| (-x, -x)),
                    (1..).map(|x| (x, -x)),
                    (1..).map(|x| (-x, x))
                ];
            }

            PieceTypes::Queen => {
                find_moves_loops![
                    moves,
                    pos,
                    game,
                    self,
                    (1..).map(|x| (0, x)),
                    (1..).map(|x| (0, -x)),
                    (1..).map(|x| (x, 0)),
                    (1..).map(|x| (-x, 0)),
                    (1..).map(|x| (x, x)),
                    (1..).map(|x| (-x, -x)),
                    (1..).map(|x| (x, -x)),
                    (1..).map(|x| (-x, x))
                ];
            }
        }
    }

    pub fn as_char(&self) -> char {
        match self.owner {
            Players::White => match self.piece_type {
                PieceTypes::King => '♔',
                PieceTypes::Queen => '♕',
                PieceTypes::Rook => '♖',
                PieceTypes::Bishop => '♗',
                PieceTypes::Knight => '♘',
                PieceTypes::Pawn => '♙',
            },
            Players::Black => match self.piece_type {
                PieceTypes::King => '♚',
                PieceTypes::Queen => '♛',
                PieceTypes::Rook => '♜',
                PieceTypes::Bishop => '♝',
                PieceTypes::Knight => '♞',
                PieceTypes::Pawn => '♟',
            },
        }
    }

    pub fn as_char_ascii(&self) -> &str {
        match self.piece_type {
            PieceTypes::King => &"K",
            PieceTypes::Queen => &"Q",
            PieceTypes::Rook => &"R",
            PieceTypes::Bishop => &"B",
            PieceTypes::Knight => &"N",
            PieceTypes::Pawn => &"",
        }
    }
}
