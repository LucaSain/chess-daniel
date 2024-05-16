use arrayvec::ArrayVec;

use super::{ChessGame, Move, Players, Position};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PieceTypes {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Piece {
    pub piece_type: PieceTypes,
    pub owner: Players,
}

macro_rules! find_moves_loops {
    ( $moves:ident, $pos:ident, $game:ident, $piece_type:ident, $( $x:expr ),* ) => {
        {
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
                            unsafe {$moves.push_unchecked(_move);}
                        }
                        break;
                    }

                    unsafe {$moves.push_unchecked(_move);}
                } else {
                    break;
                }
            }
            )*
        }
    };
}

impl Piece {
    pub fn score(&self, pos: Position) -> f64 {
        unsafe {
            (match self.owner {
                Players::White => 1.0,
                Players::Black => -1.0,
            }) * match self.piece_type {
                PieceTypes::Pawn => &[0.0, 0.9, 1.05, 1.15, 1.18, 1.2, 1.23, 1.3],
                PieceTypes::Knight => &[3.0, 3.0, 3.3, 3.31, 3.33, 3.35, 3.35, 3.4],
                PieceTypes::Bishop => &[3.1, 3.1, 3.15, 3.23, 3.34, 3.38, 3.39, 3.4],
                PieceTypes::Rook => &[5.0, 5.0, 5.1, 5.1, 5.1, 5.1, 5.2, 5.3],
                PieceTypes::Queen => &[9.0, 9.05, 9.1, 9.2, 9.2, 9.1, 9.2, 9.4],
                PieceTypes::King => &[0.0, -0.1, -0.2, -0.3, -0.3, -0.3, -0.3, -0.3],
            }
            .get_unchecked(match self.owner {
                Players::White => pos.row(),
                Players::Black => 7 - pos.row(),
            } as usize)
                * [0.96, 0.97, 0.98, 1.0, 1.0, 0.98, 0.97, 0.96].get_unchecked(pos.col() as usize)
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

                // First moves always exist
                unsafe {
                    if pos.row() == first_row
                        && game.get_position(pos.add_unsafe(normal_delta)).is_none()
                        && game.get_position(pos.add_unsafe(first_row_delta)).is_none()
                    {
                        unsafe {
                            moves.push_unchecked(Move::Normal {
                                piece: *self,
                                start: pos,
                                end: pos.add_unsafe(first_row_delta),
                                captured_piece: None,
                            });
                        }
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

                        unsafe {
                            moves.push_unchecked(_move);
                        }
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

                            unsafe {
                                moves.push_unchecked(_move);
                            }
                        }
                    }
                }
                // TODO: En passant
            }
            PieceTypes::King => {
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
                        if place.is_none()
                            || place.is_some_and(|piece| piece.owner != game.current_player)
                        {
                            unsafe {
                                moves.push_unchecked(Move::Normal {
                                    piece: *self,
                                    start: pos,
                                    end: new_pos,
                                    captured_piece: *place,
                                });
                            }
                        }
                    }
                }

                if !game.has_castled[game.current_player as usize] {
                    let row = match game.current_player {
                        Players::White => 0,
                        Players::Black => 7,
                    };
                    let (king, rook_1, rook_2) = unsafe {
                        (
                            Position::new_unsafe(row, 4),
                            Position::new_unsafe(row, 7),
                            Position::new_unsafe(row, 0),
                        )
                    };

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
                        unsafe {
                            moves.push_unchecked(Move::CastlingShort {
                                owner: game.current_player,
                            });
                        }
                    }

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
                        unsafe {
                            moves.push_unchecked(Move::CastlingLong {
                                owner: game.current_player,
                            });
                        }
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
                        if place.is_none()
                            || place.is_some_and(|piece| piece.owner != game.current_player)
                        {
                            unsafe {
                                moves.push_unchecked(Move::Normal {
                                    piece: *self,
                                    start: pos,
                                    end: new_pos,
                                    captured_piece: *place,
                                });
                            }
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
}
