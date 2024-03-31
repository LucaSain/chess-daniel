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
    CastlingShort,
    CastlingLong,
    Promovation,
}

#[derive(Clone)]
pub struct ChessGame {
    board: [[Option<Piece>; 8]; 8],
    pub targeted_by_white: [[i8; 8]; 8],
    pub targeted_by_black: [[i8; 8]; 8],
    pub move_stack: Vec<Move>, // debug
    current_player: Players,
}

mod mod_piece;
use arrayvec::ArrayVec;
use mod_piece::*;

mod mod_position;
use mod_position::*;

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
            targeted_by_white: [[0; 8]; 8],
            targeted_by_black: [[0; 8]; 8],
            move_stack: Vec::with_capacity(100),
            current_player: Players::White, 
        };

        for (position, place) in game.clone().board.iter().enumerate().flat_map(|(r, v)| {
            v.iter()
                .enumerate()
                .map(move |(c, v)| (Position::new(r as i8, c as i8), v))
        }) {
            if let Some(piece) = place {
                game.real_push(
                    Move::Normal {
                        piece: *piece,
                        start: position,
                        end: position,
                        captured_piece: None,
                    },
                    true,
                )
            }
        }

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

    // pub fn get_targeted(&self, position: Position, player: Players) -> Option<i8> {
    //     match player {
    //         Players::White => self.targeted_by_white,
    //         Players::Black => self.targeted_by_black,
    //     }
    //     .get(position.row() as usize)
    //     .and_then(|row| row.get(position.col() as usize))
    //     .map(|num| *num)
    // }

    // pub fn inc_targeted(&mut self, position: Position, player: Players) {
    //     match player {
    //         Players::White => &mut self.targeted_by_white,
    //         Players::Black => &mut self.targeted_by_black,
    //     }
    //     .get_mut(position.row() as usize)
    //     .and_then(|row| {
    //         row.get_mut(position.col() as usize).map(|num| {
    //             *num += 1;
    //         })
    //     });
    // }

    // pub fn dec_targeted(&mut self, position: Position, player: Players) {
    //     match player {
    //         Players::White => &mut self.targeted_by_white,
    //         Players::Black => &mut self.targeted_by_black,
    //     }
    //     .get_mut(position.row() as usize)
    //     .and_then(|row| {
    //         row.get_mut(position.col() as usize).map(|num| {
    //             *num -= 1;
    //         })
    //     });
    // }

    pub fn push(&mut self, _move: Move) {
        self.real_push(_move, false);
        // std::process::Command::new("clear").status().unwrap();
        // println!("{:#?}", self.clone());
        // println!("{:#?}", _move.clone());
    }

    fn real_push(&mut self, _move: Move, is_new_game: bool) {
        // if !is_new_game {
        //     match _move {
        //         #[rustfmt::skip]
        //         Move::Normal { piece, start, .. } => {
        //             piece.get_moves_protect(self, start).iter().for_each(|_move| {
        //                 match _move {
        //                     Move::Normal { piece, end, .. } => {
        //                         self.dec_targeted(*end, piece.owner);
        //                     }
        //                     // TODO others
        //                     _ => panic!(),
        //                 }
        //             })
        //         }
        //         _ => (),
        //     }
        // }
        if !is_new_game {
            self.move_stack.push(_move);
        }
        match _move {
            #[rustfmt::skip]
            Move::Normal { piece, start, end, .. } => {
                self.set_position(start, None);
                self.set_position(end, Some(piece));
                
                // after a normal move we can't exepect the castling since the king moved
                // piece.get_moves_protect(self, end).iter().for_each(|_move| {
                //     match _move {
                //         Move::Normal { piece, end, .. } => {
                //             self.inc_targeted(*end, piece.owner);
                //         }
                //         // TODO others
                //         _ => panic!(),
                //     }
                // })
            }
            // TODO: other moves
            _ => (),
        };
        self.current_player = self.current_player.the_other();
    }

    pub fn pop(&mut self) -> Move {
        let _move = self.move_stack.pop().expect("Tried to pop a new game");

        match _move {
            #[rustfmt::skip]
            Move::Normal { piece, start, end, captured_piece }=> {             
                // piece.get_moves_protect(self, end).iter().for_each(|_move| {
                //     match _move {
                //         Move::Normal { piece, end, .. } => {
                //             self.dec_targeted(*end, piece.owner);
                //         }
                //         // TODO others
                //         _ => panic!(),
                //     }
                // });

                self.set_position(start, Some(piece));
                self.set_position(end, captured_piece);
            }
            // TODO: other moves
            _ => (),
        };

        // match _move {
        //     #[rustfmt::skip]
        //     Move::Normal { piece, start, .. } => {
        //         piece.get_moves_protect(self, start).iter().for_each(|_move| {
        //             match _move {
        //                 Move::Normal { piece, end, .. } => {
        //                     self.inc_targeted(*end, piece.owner);
        //                 }
        //                 // TODO others
        //                 _ => panic!(),
        //             }
        //         })
        //     }
        //     _ => (),
        // }

        self.current_player = self.current_player.the_other();
        _move
    }

    pub fn get_moves(&self) -> ArrayVec<Move, 512> {
        let piece_moves = self
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
            });

        let mut moves = ArrayVec::<Move, 512>::new();
        for list in piece_moves {
            for item in list.iter() {
                unsafe {
                    moves.push_unchecked(*item);
                }
            }
        }

        moves
    }
}

impl std::fmt::Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let piece = match self.piece_type {
            PieceTypes::Pawn => "p",
            PieceTypes::Rook => "r",
            PieceTypes::Bishop => "b",
            PieceTypes::Knight => "k",
            PieceTypes::Queen => "q",
            PieceTypes::King => "_",
        };
        write!(
            f,
            "{}",
            match self.owner {
                Players::White => piece.to_owned(),
                Players::Black => piece.to_uppercase(),
            }
        )
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
        write!(
            f,
            "\n\n{:?},\n{:?},\n{:?},\n{:?},\n{:?},\n{:?},\n{:?},\n{:?}\n",
            self.board[0].map(|place| place.unwrap_or(Piece {
                piece_type: PieceTypes::King,
                owner: Players::White
            })),
            self.board[1].map(|place| place.unwrap_or(Piece {
                piece_type: PieceTypes::King,
                owner: Players::White
            })),
            self.board[2].map(|place| place.unwrap_or(Piece {
                piece_type: PieceTypes::King,
                owner: Players::White
            })),
            self.board[3].map(|place| place.unwrap_or(Piece {
                piece_type: PieceTypes::King,
                owner: Players::White
            })),
            self.board[4].map(|place| place.unwrap_or(Piece {
                piece_type: PieceTypes::King,
                owner: Players::White
            })),
            self.board[5].map(|place| place.unwrap_or(Piece {
                piece_type: PieceTypes::King,
                owner: Players::White
            })),
            self.board[6].map(|place| place.unwrap_or(Piece {
                piece_type: PieceTypes::King,
                owner: Players::White
            })),
            self.board[7].map(|place| place.unwrap_or(Piece {
                piece_type: PieceTypes::King,
                owner: Players::White
            })),
        )
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
