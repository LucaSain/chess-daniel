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

#[derive(Clone, Copy)]
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
                    unsafe {
                        moves.push_unchecked(Move::Normal {
                            piece: *self,
                            start: pos,
                            end: pos + normal_delta,
                            captured_piece: None,
                        });
                    }
                }

                for delta in side_deltas {
                    if let Some(place) = game.get_position(pos + delta) {
                        if place.is_some_and(|piece| piece.owner != self.owner) {
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

                // TODO: En Passant 😃

                // TODO: Promotion
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
                        if !game.is_targeted(pos + delta, false)
                            && !game
                                .get_position(pos + delta)
                                .unwrap()
                                .is_some_and(|piece| piece.owner == game.current_player)
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

                // TODO: Castling
            }
            PieceTypes::Knight => {
                for delta in [
                    (1, 2),
                    (2, 1),
                    (-1, -2),
                    (-2, -1),
                    (1, -2),
                    (-2, -1),
                    (-1, 2),
                    (2, -1),
                ]
                .iter()
                .map(|(row, col)| Position::new(*row, *col))
                {
                    if let Some(place) = game.get_position(pos + delta) {
                        if place.is_some_and(|piece| piece.owner != game.current_player) {
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
