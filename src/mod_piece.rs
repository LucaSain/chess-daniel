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
                let end = $pos + delta;
                if let Some(place) = $game.get_position(end) {
                    let _move = Move::Normal {
                        piece: *$piece_type,
                        start: $pos,
                        end,
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
    pub fn get_moves(&self, game: &ChessGame, pos: Position) -> ArrayVec<Move, 27> {
        let mut moves = ArrayVec::new();
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
                    Players::White => Position::new(1, 0),
                    Players::Black => Position::new(-1, 0),
                };

                let first_row_delta = match self.owner {
                    Players::White => Position::new(2, 0),
                    Players::Black => Position::new(-2, 0),
                };

                if pos.row() == first_row
                    && game.get_position(pos + normal_delta).unwrap().is_none()
                    && game.get_position(pos + first_row_delta).unwrap().is_none()
                {
                    unsafe {
                        moves.push_unchecked(Move::Normal {
                            piece: *self,
                            start: pos,
                            end: pos + first_row_delta,
                            captured_piece: None,
                        });
                    }
                }
                let side_deltas = match self.owner {
                    Players::White => [Position::new(1, 1), Position::new(1, -1)],
                    Players::Black => [Position::new(-1, 1), Position::new(-1, -1)],
                };

                if game
                    .get_position(pos + normal_delta)
                    .is_some_and(|place| place.is_none())
                {
                    let _move = if last_row == (pos + normal_delta).row() {
                        Move::Promovation {
                            owner: game.current_player,
                            start: pos,
                            end: pos + normal_delta,
                            captured_piece: None,
                        }
                    } else {
                        Move::Normal {
                            piece: *self,
                            start: pos,
                            end: pos + normal_delta,
                            captured_piece: None,
                        }
                    };

                    unsafe {
                        moves.push_unchecked(_move);
                    }
                }

                for delta in side_deltas {
                    if let Some(place) = game.get_position(pos + delta) {
                        if place.is_some_and(|piece| piece.owner != self.owner) {
                            let _move = if last_row == (pos + delta).row() {
                                Move::Promovation {
                                    owner: game.current_player,
                                    start: pos,
                                    end: pos + delta,
                                    captured_piece: *place,
                                }
                            } else {
                                Move::Normal {
                                    piece: *self,
                                    start: pos,
                                    end: pos + delta,
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
                .iter()
                .map(|(row, col)| Position::new(*row, *col))
                {
                    if let Some(place) = game.get_position(pos + delta) {
                        if place.is_none()
                            || place.is_some_and(|piece| piece.owner != game.current_player)
                        {
                            unsafe {
                                moves.push_unchecked(Move::Normal {
                                    piece: *self,
                                    start: pos,
                                    end: pos + delta,
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

                    let pos_short = [Position::new(row, 5), Position::new(row, 6)];
                    // TODO make sure those empty squares are not targeted
                    if pos_short
                        .iter()
                        .all(|pos| game.get_position(*pos).unwrap().is_none())
                        && game.get_position(Position::new(row, 4)).unwrap()
                            == &Some(Piece {
                                piece_type: PieceTypes::King,
                                owner: game.current_player,
                            })
                        && game.get_position(Position::new(row, 7)).unwrap()
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

                    let pos_long = [
                        Position::new(row, 1),
                        Position::new(row, 2),
                        Position::new(row, 3),
                    ];

                    if pos_long
                        .iter()
                        .all(|pos| game.get_position(*pos).unwrap().is_none())
                        && game.get_position(Position::new(row, 4)).unwrap()
                            == &Some(Piece {
                                piece_type: PieceTypes::King,
                                owner: game.current_player,
                            })
                        && game.get_position(Position::new(row, 0)).unwrap()
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
                .iter()
                .map(|(row, col)| Position::new(*row, *col))
                {
                    if let Some(place) = game.get_position(pos + delta) {
                        if place.is_none()
                            || place.is_some_and(|piece| piece.owner != game.current_player)
                        {
                            unsafe {
                                moves.push_unchecked(Move::Normal {
                                    piece: *self,
                                    start: pos,
                                    end: pos + delta,
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
                    (1..).map(|x| Position::new(0, x)),
                    (1..).map(|x| Position::new(0, -x)),
                    (1..).map(|x| Position::new(x, 0)),
                    (1..).map(|x| Position::new(-x, 0))
                ];
            }

            PieceTypes::Bishop => {
                find_moves_loops![
                    moves,
                    pos,
                    game,
                    self,
                    (1..).map(|x| Position::new(x, x)),
                    (1..).map(|x| Position::new(-x, -x)),
                    (1..).map(|x| Position::new(x, -x)),
                    (1..).map(|x| Position::new(-x, x))
                ];
            }

            PieceTypes::Queen => {
                find_moves_loops![
                    moves,
                    pos,
                    game,
                    self,
                    (1..).map(|x| Position::new(0, x)),
                    (1..).map(|x| Position::new(0, -x)),
                    (1..).map(|x| Position::new(x, 0)),
                    (1..).map(|x| Position::new(-x, 0)),
                    (1..).map(|x| Position::new(x, x)),
                    (1..).map(|x| Position::new(-x, -x)),
                    (1..).map(|x| Position::new(x, -x)),
                    (1..).map(|x| Position::new(-x, x))
                ];
            }
        }

        moves
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
