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
pub struct ChessGame {
    board: [[Option<Piece>; 8]; 8],
    pub move_stack: Vec<Move>, // debug
    current_player: Players,
    king_positions: [Position; 2], // for finding if it is in check
}

mod mod_piece;
use arrayvec::ArrayVec;
use mod_piece::*;

mod mod_position;
pub use mod_position::*;

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
                Position::new(0, 4),
                Position::new(7, 4),
            ],
            current_player: Players::White,
        };

        game
    }

    pub fn get_position(&self, position: Position) -> Option<&Option<Piece>> {
        self.board
            .get(position.row() as usize)
            .and_then(|row| row.get(position.col() as usize))
    }

    fn set_position(&mut self, position: Position, new_place: Option<Piece>) {
        self.board.get_mut(position.row() as usize).and_then(|row| {
            row.get_mut(position.col() as usize)
                .map(|place| *place = new_place)
        });
    }

    pub fn push(&mut self, _move: Move) {
        self.move_stack.push(_move);

        match _move {
            #[rustfmt::skip]
            Move::Normal { piece, start, end, .. } => {
                self.set_position(start, None);
                self.set_position(end, Some(piece));

                if piece.piece_type == PieceTypes::King {
                    self.king_positions[self.current_player as usize] = end;
                }
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
            Move::CastlingLong { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };

                self.set_position(Position::new(row, 0), None);
                self.set_position(
                    Position::new(row, 3),
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                );

                self.set_position(Position::new(row, 4), None);
                self.set_position(
                    Position::new(row, 2),
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                );

                self.has_castled[self.current_player as usize] = true;
                self.king_positions[self.current_player as usize] = Position::new(row, 2);
            }
            Move::CastlingShort { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };

                self.set_position(Position::new(row, 7), None);
                self.set_position(
                    Position::new(row, 5),
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                );

                self.set_position(Position::new(row, 4), None);
                self.set_position(
                    Position::new(row, 6),
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                );

                self.has_castled[self.current_player as usize] = true;
                self.king_positions[self.current_player as usize] = Position::new(row, 6)
            }
        };

        self.current_player = self.current_player.the_other();
    }

    pub fn pop(&mut self) -> Move {
        let _move = self.move_stack.pop().expect("Tried to pop a new game");
        self.current_player = self.current_player.the_other();

        match _move {
            #[rustfmt::skip]
            Move::Normal { piece, start, end, captured_piece } => {
                self.set_position(start, Some(piece));
                self.set_position(end, captured_piece);

                if piece.piece_type == PieceTypes::King {
                    self.king_positions[self.current_player as usize] = start;
                }
            }
            #[rustfmt::skip]
            Move::Promovation { owner, start, end, captured_piece } => {
                self.set_position(start, Some(Piece { piece_type: PieceTypes::Pawn, owner }));
                self.set_position(end, captured_piece);
            }
            Move::CastlingLong { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };

                self.set_position(
                    Position::new(row, 0),
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                );
                self.set_position(Position::new(row, 3), None);

                self.set_position(
                    Position::new(row, 4),
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                );
                self.set_position(Position::new(row, 2), None);

                self.has_castled[owner as usize] = false;
                self.king_positions[owner as usize] = Position::new(row, 4);
            }
            Move::CastlingShort { owner } => {
                let row = match owner {
                    Players::White => 0,
                    Players::Black => 7,
                };

                self.set_position(
                    Position::new(row, 7),
                    Some(Piece {
                        piece_type: PieceTypes::Rook,
                        owner,
                    }),
                );
                self.set_position(Position::new(row, 5), None);

                self.set_position(
                    Position::new(row, 4),
                    Some(Piece {
                        piece_type: PieceTypes::King,
                        owner,
                    }),
                );
                self.set_position(Position::new(row, 6), None);

                self.has_castled[owner as usize] = false;
                self.king_positions[owner as usize] = Position::new(row, 4)
            }
        };

        _move
    }

    pub fn get_moves(&mut self) -> ArrayVec<Move, 128> {
        let piece_moves: ArrayVec<ArrayVec<Move, 27>, 32> = self
            .board
            .iter()
            .enumerate()
            .flat_map(|(r, v)| {
                v.iter()
                    .enumerate()
                    .map(move |(c, v)| (Position::new(r as i8, c as i8), v))
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
                self.push(*item);
                if !self.is_targeted(self.king_positions[player as usize]) {
                unsafe {
                    moves.push_unchecked(*item);
                }
                }
                self.pop();
            }
        }

        moves
    }

    // Returns if piece is targeted by enemy pieces
    pub fn is_targeted(&self, position: Position) -> bool {
        // This function should only be called with valid pieces
        let player = self.get_position(position).unwrap().unwrap().owner;

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
            .iter()
            .map(|(row, col)| Position::new(*row, *col))
            {
                if let Some(place) = self.get_position(position + delta) {
                    if place.is_some_and(|piece| {
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
        .iter()
        .map(|(row, col)| Position::new(*row, *col))
        {
            if let Some(place) = self.get_position(position + delta) {
                if place.is_some_and(|piece| {
                    piece.owner != player && piece.piece_type == PieceTypes::Knight
                }) {
                    return true;
                }
            }
        }

        // Verify for pawns
        match player {
            Players::White => {
                if let Some(place) = self.get_position(position + Position::new(1, 1)) {
                    if place.is_some_and(|piece| {
                        piece.owner != player && piece.piece_type == PieceTypes::Pawn
                    }) {
                        return true;
                    }
                }
                if let Some(place) = self.get_position(position + Position::new(1, -1)) {
                    if place.is_some_and(|piece| {
                        piece.owner != player && piece.piece_type == PieceTypes::Pawn
                    }) {
                        return true;
                    }
                }
            }
            Players::Black => {
                if let Some(place) = self.get_position(position + Position::new(-1, 1)) {
                    if place.is_some_and(|piece| {
                        piece.owner != player && piece.piece_type == PieceTypes::Pawn
                    }) {
                        return true;
                    }
                }
                if let Some(place) = self.get_position(position + Position::new(-1, -1)) {
                    if place.is_some_and(|piece| {
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
                    if let Some(place) = self.get_position(position + delta) {
                        if let Some(piece) = place  {
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
            (1..).map(|x| Position::new(0, x)),
            (1..).map(|x| Position::new(0, -x)),
            (1..).map(|x| Position::new(x, 0)),
            (1..).map(|x| Position::new(-x, 0))
        ];

        // Verify diagonals for bishops/queens
        search_enemies_loops![
            PieceTypes::Bishop,
            PieceTypes::Queen,
            (1..).map(|x| Position::new(x, x)),
            (1..).map(|x| Position::new(-x, -x)),
            (1..).map(|x| Position::new(x, -x)),
            (1..).map(|x| Position::new(-x, x))
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
        self.board.iter().try_for_each(|row| -> std::fmt::Result {
            row.iter().try_for_each(|place| -> std::fmt::Result {
                write!(f, "|{}", place.map(|piece| piece.as_char()).unwrap_or(' '))
            })?;
            write!(f, "|\n")
        })
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
