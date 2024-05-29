use arrayvec::ArrayVec;
use seq_macro::seq;

use crate::move_struct::Move;
use crate::piece::{Piece, PieceTypes};
use crate::position::Position;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Players {
    White = 1,
    Black = -1,
}

/// Information about the state of the game at a moment in time that can't be derived easily
/// Because of that, we hold it in a stack in the ChessGame struct
#[derive(Clone, Copy, Debug)]
pub struct GameState {
    /// First 4 bits represent en passant
    /// The last 4 bits of castling_rights indicate castling rights
    bitfield: u8,
}

impl GameState {
    pub fn en_passant(self) -> i8 {
        (self.bitfield & 0b1111) as i8
    }
    pub fn set_en_passant(&mut self, value: i8) {
        self.bitfield = (self.bitfield & 0b11110000) + (value as u8);
    }

    pub fn white_king_castling(self) -> bool {
        (self.bitfield & (1 << 4)) != 0
    }
    pub fn set_white_king_castling_false(&mut self) {
        self.bitfield &= !(1 << 4);
    }
    pub fn set_white_king_castling_true(&mut self) {
        self.bitfield |= 1 << 4;
    }

    pub fn white_queen_castling(self) -> bool {
        (self.bitfield & (1 << 5)) != 0
    }
    pub fn set_white_queen_castling_false(&mut self) {
        self.bitfield &= !(1 << 5);
    }
    pub fn set_white_queen_castling_true(&mut self) {
        self.bitfield |= 1 << 5;
    }

    pub fn black_king_castling(self) -> bool {
        (self.bitfield & (1 << 6)) != 0
    }
    pub fn set_black_king_castling_false(&mut self) {
        self.bitfield &= !(1 << 6);
    }
    pub fn set_black_king_castling_true(&mut self) {
        self.bitfield |= 1 << 6;
    }

    pub fn black_queen_castling(self) -> bool {
        (self.bitfield & (1 << 7)) != 0
    }
    pub fn set_black_queen_castling_false(&mut self) {
        self.bitfield &= !(1 << 7);
    }
    pub fn set_black_queen_castling_true(&mut self) {
        self.bitfield |= 1 << 7;
    }
}

impl Default for GameState {
    /// Default state is no en passant square, and no castling rights
    fn default() -> Self {
        Self {
            // 8 Represents no en passant square
            bitfield: 8,
        }
    }
}

#[derive(Clone)]
pub struct ChessGame {
    pub score: i32,
    pub current_player: Players,
    pub move_stack: Vec<Move>,
    board: [[Option<Piece>; 8]; 8],
    king_positions: [Position; 2],
    state: ArrayVec<GameState, 512>,
}

impl Players {
    pub fn the_other(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl Default for ChessGame {
    fn default() -> Self {
        #[rustfmt::skip]
        let mut game = Self {
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
            king_positions: [
                Position::new(0, 4).unwrap(),
                Position::new(7, 4).unwrap(),
            ],
            current_player: Players::White,
            score: 0,
            state: ArrayVec::new(),
        };

        let mut state = GameState::default();
        state.set_white_king_castling_true();
        state.set_white_queen_castling_true();
        state.set_black_king_castling_true();
        state.set_black_queen_castling_true();

        game.state.push(state);
        game
    }
}

impl ChessGame {
    pub fn new(fen: &str) -> Result<Self, &str> {
        let mut terms = fen.split_ascii_whitespace();

        let mut board = [[None; 8]; 8];
        let mut white_king_pos = Position::new(0, 0).unwrap();
        let mut black_king_pos = Position::new(0, 0).unwrap();

        if let Some(pieces) = terms.next() {
            let mut row = 7;
            let mut col = 0;
            for character in pieces.chars() {
                match character {
                    '/' => {
                        if row == 0 {
                            return Err("Too many rows");
                        }
                        col = 0;
                        row -= 1;
                    }
                    piece if piece.is_ascii_alphabetic() => {
                        if col == 8 {
                            return Err("Too many columns");
                        }
                        let piece = Piece::from_char_ascii(piece)?;
                        if piece.piece_type == PieceTypes::King {
                            match piece.owner {
                                Players::White => white_king_pos = Position::new(row, col).unwrap(),
                                Players::Black => black_king_pos = Position::new(row, col).unwrap(),
                            }
                        }
                        board[row as usize][col as usize] = Some(piece);
                        col += 1;
                    }
                    empty_count if character.is_ascii_digit() => {
                        col += (empty_count as u8 - b'0') as i8;
                    }
                    _ => return Err("Unknown character met"),
                }
            }
        } else {
            return Err("Invalid FEN");
        }

        let current_player = if let Some(next_player) = terms.next() {
            match next_player.chars().next().unwrap() {
                'w' => Players::White,
                'b' => Players::Black,
                _ => return Err("Invalid FEN"),
            }
        } else {
            return Err("Invalid FEN");
        };

        let mut state = GameState::default();
        if let Some(castling_rights) = terms.next() {
            for right in castling_rights.chars() {
                match right {
                    'K' => state.set_white_king_castling_true(),
                    'Q' => state.set_white_queen_castling_true(),
                    'k' => state.set_black_king_castling_true(),
                    'q' => state.set_black_queen_castling_true(),
                    _ => continue,
                }
            }
        } else {
            return Err("Invalid FEN");
        }

        if let Some(en_passant) = terms.next() {
            if en_passant != "-" {
                let mut chars = en_passant.chars();
                if let Some(col) = chars.next() {
                    state.set_en_passant(((col as u8) - b'a') as i8);
                    if !(0..8).contains(&state.en_passant()) {
                        return Err("Invalid FEN");
                    }
                } else {
                    return Err("Invalid FEN");
                }
            }
        } else {
            return Err("Invalid FEN");
        }

        let mut game = Self {
            board,
            move_stack: Vec::with_capacity(1000),
            king_positions: [white_king_pos, black_king_pos],
            current_player,
            score: 0,
            state: ArrayVec::new(),
        };

        game.state.push(state);
        Ok(game)
    }

    pub fn len(&self) -> usize {
        self.state.len()
    }

    pub fn get_position(&self, position: Position) -> &Option<Piece> {
        // SAFETY: position is always valid
        unsafe {
            self.board
                .get_unchecked(position.row() as usize)
                .get_unchecked(position.col() as usize)
        }
    }

    fn set_position(&mut self, position: Position, new_place: Option<Piece>) {
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

    pub fn state(&self) -> &GameState {
        // SAFETY: There should always be a valid state
        unsafe { self.state.last().unwrap_unchecked() }
    }

    pub fn push_history(&mut self, _move: Move) {
        self.move_stack.push(_move);

        self.push(_move);
    }

    pub fn push(&mut self, _move: Move) {
        let mut state = *self.state();
        state.set_en_passant(8);
        match _move {
            #[rustfmt::skip]
            Move::Normal { piece, start, end, captured_piece } => {
                self.set_position(start, None);
                self.set_position(end, Some(piece));

                if piece.piece_type == PieceTypes::King {
                    self.set_king_position(self.current_player, end);
                    match self.current_player {
                        Players::White => {
                            state.set_white_king_castling_false();
                            state.set_white_queen_castling_false();
                        }
                        Players::Black =>  {
                            state.set_black_king_castling_false();
                            state.set_black_queen_castling_false();
                        }
                    }
                } else if piece.piece_type == PieceTypes::Rook {
                    if start.col() == 0 {
                        match self.current_player {
                            Players::White => state.set_white_queen_castling_false(),
                            Players::Black => state.set_black_queen_castling_false(),
                        }
                    } else if start.col() == 7 {
                        match self.current_player {
                            Players::White => state.set_white_king_castling_false(),
                            Players::Black => state.set_black_king_castling_false(),
                        }
                    }
                }

                if captured_piece.is_some_and(|piece| piece.piece_type == PieceTypes::Rook) {
                    // SAFETY: Hardcoded positions are valid
                    let (pos1, pos2, pos3, pos4) = unsafe {
                        (
                        Position::new_unsafe(0, 0),
                        Position::new_unsafe(0, 7),
                        Position::new_unsafe(7, 0),
                        Position::new_unsafe(7, 7),
                    )
                    };

                    if end == pos1 {
                        state.set_white_queen_castling_false();
                    } else if end == pos2 {
                        state.set_white_king_castling_false();
                    } else if end == pos3 {
                        state.set_black_queen_castling_false();
                    } else if end == pos4 {
                        state.set_black_king_castling_false();
                    }
                }

                if piece.piece_type == PieceTypes::Pawn && i8::abs(end.row() - start.row()) == 2 {
                    state.set_en_passant(start.col());
                }
            }
            #[rustfmt::skip]
            Move::Promotion { owner, start, end, new_piece, .. } => {
                self.set_position(start, None);
                self.set_position(
                    end,
                    Some(Piece {
                        owner,
                        piece_type: new_piece,
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
                            owner,
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
                self.set_king_position(self.current_player, new_king);
                match self.current_player {
                    Players::White => {
                        state.set_white_king_castling_false();
                        state.set_white_queen_castling_false();
                    }
                    Players::Black => {
                        state.set_black_king_castling_false();
                        state.set_black_queen_castling_false();
                    }
                }
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

                self.set_king_position(self.current_player, new_king);
                match self.current_player {
                    Players::White => {
                        state.set_white_king_castling_false();
                        state.set_white_queen_castling_false();
                    }
                    Players::Black => {
                        state.set_black_king_castling_false();
                        state.set_black_queen_castling_false();
                    }
                }
            }
        };
        self.current_player = self.current_player.the_other();
        // SAFETY: The game will not be longer than 512 moves
        unsafe {
            self.state.push_unchecked(state);
        }
    }

    pub fn pop(&mut self, _move: Move) {
        // SAFETY: There is always a previous state
        unsafe {
            // self.state.pop() without verification for being empty
            self.state.set_len(self.len() - 1);
        }
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
            Move::Promotion { owner, start, end, captured_piece, .. } => {
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
                            owner,
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

                self.set_king_position(owner, old_king);
            }
        };
    }

    /// `moves` will be cleared by this function to be sure it has room for all moves
    pub fn get_moves(&mut self, moves: &mut ArrayVec<Move, 256>, verify_king: bool) {
        moves.clear();

        let king_place = self.get_position(self.get_king_position(self.current_player));
        if !king_place.is_some_and(|piece| piece.piece_type == PieceTypes::King) {
            // no available moves;
            return;
        }

        seq!(row in 0..8 {
            seq!(col in 0..8 {
                // SAFETY: Theses are hardcoded valid positions,
                // and moves is empty at the beginning
                unsafe {
                    if let Some(piece) = *self
                            .board
                            .get_unchecked(row)
                            .get_unchecked(col)
                        {
                            if piece.owner == self.current_player {
                                let pos = Position::new_unsafe(row, col);
                                piece.get_moves(moves, self, pos);
                            }
                        }
                    }
            });
        });

        // If verify_king then remove moves which put the king in check (invalid moves)
        // We remove invalid moves by overwriting them with the following valid moves
        if verify_king {
            let mut keep_index = 0;
            let player = self.current_player;
            for index in 0..moves.len() {
                // SAFETY: 0 <= keep_index <= index < moves.len()
                let _move = unsafe { *moves.get_unchecked(index) };
                self.push(_move);
                let condition = !self.is_targeted(self.get_king_position(player), player);
                self.pop(_move);
                if condition {
                    // Keep this move
                    unsafe { *moves.get_unchecked_mut(keep_index) = _move };
                    keep_index += 1;
                }
            }
            unsafe { moves.set_len(keep_index) };
        }
    }

    /// Returns if player's position is targeted by enemy pieces
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
        ] {
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
        ] {
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
        let moves: Vec<_> = self.move_stack.iter().map(Move::pgn_notation).collect();

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

impl std::fmt::Debug for ChessGame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f)?;
        self.board
            .iter()
            .enumerate()
            .rev()
            .try_for_each(|(i, row)| -> std::fmt::Result {
                write!(f, "{} ", i + 1)?;
                row.iter().try_for_each(|place| -> std::fmt::Result {
                    write!(f, "|{}", place.map(|piece| piece.as_char()).unwrap_or(' '))
                })?;
                writeln!(f, "|")
            })?;
        writeln!(f, "\n   a b c d e f g h")
    }
}
