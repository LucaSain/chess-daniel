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
    pub move_stack: Vec<Move>, // debug
    current_player: Players,
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
            Move::Normal { piece, start, end, captured_piece } => {
                self.set_position(start, Some(piece));
                self.set_position(end, captured_piece);
            }
            // TODO: other moves
            _ => (),
        };

        self.current_player = self.current_player.the_other();
        _move
    }

    pub fn get_moves(&self) -> ArrayVec<Move, 64> {
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

        let mut moves = ArrayVec::new();
        for list in piece_moves {
            for item in list.iter() {
                unsafe {
                    moves.push_unchecked(*item);
                }
            }
        }

        moves
    }

    // Returns if position is targeted by enemy pieces (including by the king or not,
    // this is required for finding valid moves for the king)
    pub fn is_targeted(&self, position: Position, including_king: bool) -> bool {
        // This function should only be called with valid positions
        if self.get_position(position).is_none() {
            panic!();
        }

        // When finding is a move is valid for the king we need to check
        // if it is targeted by a non-king piece, so do this conditionally
        if including_king {
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
                        piece.owner != self.current_player && piece.piece_type == PieceTypes::King
                    }) {
                        return true;
                    }
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
            (-2, -1),
            (-1, 2),
            (2, -1),
        ]
        .iter()
        .map(|(row, col)| Position::new(*row, *col))
        {
            if let Some(place) = self.get_position(position + delta) {
                if place.is_some_and(|piece| {
                    piece.owner != self.current_player && piece.piece_type == PieceTypes::Knight
                }) {
                    return true;
                }
            }
        }

        // Verify for pawns
        match self.current_player {
            Players::White => {
                if let Some(place) = self.get_position(position + Position::new(1, 1)) {
                    if place.is_some_and(|piece| {
                        piece.owner != self.current_player && piece.piece_type == PieceTypes::Pawn
                    }) {
                        return true;
                    }
                }
                if let Some(place) = self.get_position(position + Position::new(1, -1)) {
                    if place.is_some_and(|piece| {
                        piece.owner != self.current_player && piece.piece_type == PieceTypes::Pawn
                    }) {
                        return true;
                    }
                }
            }
            Players::Black => {
                if let Some(place) = self.get_position(position + Position::new(-1, 1)) {
                    if place.is_some_and(|piece| {
                        piece.owner != self.current_player && piece.piece_type == PieceTypes::Pawn
                    }) {
                        return true;
                    }
                }
                if let Some(place) = self.get_position(position + Position::new(-1, -1)) {
                    if place.is_some_and(|piece| {
                        piece.owner != self.current_player && piece.piece_type == PieceTypes::Pawn
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
                            if piece.owner != self.current_player &&
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
