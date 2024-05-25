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

// Information about the state of the game at this moment
#[derive(Clone, Copy, Debug)]
pub struct GameState {
    en_passant: i8,
    white_king_castling: bool,
    white_queen_castling: bool,
    black_king_castling: bool,
    black_queen_castling: bool,
    pub last_position: Option<Position>,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            en_passant: -1,
            white_king_castling: true,
            white_queen_castling: true,
            black_king_castling: true,
            black_queen_castling: true,
            last_position: None,
        }
    }
}

#[derive(Clone)]
// all fields are public for debugging
// TODO: remove pub
pub struct ChessGame {
    pub score: i32,
    pub current_player: Players,
    pub board: [[Option<Piece>; 8]; 8],
    king_positions: [Position; 2],
    pub move_stack: Vec<Move>,
    pub state: ArrayVec<GameState, 256>,
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
            king_positions: [
                Position::new(0, 4).unwrap(),
                Position::new(7, 4).unwrap(),
            ],
            current_player: Players::White,
            score: 0,
            state: ArrayVec::new(),
        };
        game.state.push(GameState::default());
        game
    }

    pub fn from_fen(fen: &str) -> Result<ChessGame, &str> {
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
                    },
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
                    _ => return Err("Unknown character met")
                }
            }
        } else {
            return Err("Invalid FEN");
        }

        let current_player = if let Some(next_player) = terms.next() {
            match next_player.chars().next().unwrap() {
                'w' => Players::White,
                'b' => Players::Black,
                _ => return Err("Invalid FEN")
            }
        } else {
            return Err("Invalid FEN");
        };

        let mut state = GameState {
            en_passant: -1,
            white_king_castling: false,
            white_queen_castling: false,
            black_king_castling: false,
            black_queen_castling: false,
            last_position: None
        };

        if let Some(castling_rights) = terms.next() {
            let mut rights = castling_rights.chars();
            while let Some(right) = rights.next() {
                match right {
                    'K' => state.white_king_castling = true,
                    'Q' => state.white_queen_castling = true,
                    'k' => state.black_king_castling = true,
                    'q' => state.black_queen_castling = true,
                    _ => continue
                }
            }
        } else {
            return Err("Invalid FEN");
        }

        if let Some(en_passant) = terms.next() {
            if en_passant != "-" {
                let mut chars = en_passant.chars();
                if let Some(col) = chars.next() {
                    state.en_passant = ((col as u8) - b'a') as i8;
                    if !(0..8).contains(&state.en_passant) {
                        return Err("Invalid FEN");
                    }
                } else {
                    return Err("Invalid FEN");
                }
            }
        } else {
            return Err("Invalid FEN");
        }


        let mut game = ChessGame {
            board,
            move_stack: Vec::with_capacity(1000),
            king_positions: [white_king_pos, black_king_pos],
            current_player,
            score: 0,
            state: ArrayVec::new(),
        };

        game.state.push(state);
        dbg!(game.clone());
        dbg!(game.state.clone());
        Ok(game)
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
        state.en_passant = -1;
        match _move {
            #[rustfmt::skip]
            Move::Normal { piece, start, end, captured_piece } => {
                self.set_position(start, None);
                self.set_position(end, Some(piece));

                if captured_piece.is_some() {
                    state.last_position = Some(end);
                }

                if piece.piece_type == PieceTypes::King {
                    self.set_king_position(self.current_player, end);
                    match self.current_player {
                        Players::White => {
                            state.white_king_castling = false;
                            state.white_queen_castling = false;
                        }
                        Players::Black =>  {
                            state.black_king_castling = false;
                            state.black_queen_castling = false;
                        }
                    }
                } else if piece.piece_type == PieceTypes::Rook {
                    if start.col() == 0 {
                        match self.current_player {
                            Players::White => state.white_queen_castling = false,
                            Players::Black => state.black_queen_castling = false,
                        }
                    } else if start.col() == 7 {
                        match self.current_player {
                            Players::White => state.white_king_castling = false,
                            Players::Black => state.black_king_castling = false,
                        }
                    }
                }

                state.en_passant = 
                    if piece.piece_type == PieceTypes::Pawn && i8::abs(end.row() - start.row()) == 2 
                        { start.col() } else { -1 }
            }
            #[rustfmt::skip]
            Move::Promovation { owner, start, end, captured_piece } => {
                self.set_position(start, None);
                self.set_position(
                    end,
                    Some(Piece {
                        owner,
                        piece_type: PieceTypes::Queen,
                    }),
                );
                
                if captured_piece.is_some() {
                    state.last_position = Some(end);
                }
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
                self.set_king_position(self.current_player, new_king);
                match self.current_player {
                    Players::White => {
                        state.white_king_castling = false;
                        state.white_queen_castling = false;
                    }
                    Players::Black => {
                        state.black_king_castling = false;
                        state.black_queen_castling = false;
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
                        state.white_king_castling = false;
                        state.white_queen_castling = false;
                    }
                    Players::Black => {
                        state.black_king_castling = false;
                        state.black_queen_castling = false;
                    }
                }
            }
        };
        self.current_player = self.current_player.the_other();
        // SAFETY: The game should not be longer than 256 moves
        unsafe {
            self.state.push_unchecked(state);
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
        self.state.pop();
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

    pub fn get_moves(&mut self, mut moves: &mut ArrayVec<Move, 128>, verify_king: bool) {
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
            .map(|_move| _move.pgn_notation())
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

    pub fn get_uci(&self) -> String {
        let moves: Vec<_> = self
            .move_stack
            .iter()
            .map(|_move| _move.uci_notation())
            .collect();

        let mut s = String::new();

        for _move in moves {
            s.push_str(_move.as_str());
            s.push(' ');
        }

        s
    }
}

impl Move {
    pub fn uci_notation(&self) -> String {
        let mut s = String::new();
        match self {
            Move::Normal { start, end, .. } => {
                s.push((start.col() as u8 + b'a') as char);
                s.push((start.row() as u8 + b'1') as char);
                s.push((end.col() as u8 + b'a') as char);
                s.push((end.row() as u8 + b'1') as char);
            }
            Move::Promovation { start, end, .. } => {
                s.push((start.col() as u8 + b'a') as char);
                s.push((start.row() as u8 + b'1') as char);
                s.push((end.col() as u8 + b'a') as char);
                s.push((end.row() as u8 + b'1') as char);
                s.push('q');
            }
            Move::CastlingShort { owner } => {
                let row = match owner {
                    Players::White => '1',
                    Players::Black => '8',
                };
                s.push('e');
                s.push(row);
                s.push('g');
                s.push(row);
            }
            Move::CastlingLong { owner } => {
                let row = match owner {
                    Players::White => '1',
                    Players::Black => '8',
                };
                s.push('e');
                s.push(row);
                s.push('c');
                s.push(row);
            }
            Move::EnPassant {
                owner,
                start_col,
                end_col,
            } => {
                let (start_row, end_row) = match owner {
                    Players::White => ('5', '6'),
                    Players::Black => ('4', '3'),
                };
                s.push((*start_col as u8 + b'a') as char);
                s.push(start_row);
                s.push((*end_col as u8 + b'a') as char);
                s.push(end_row);
            }
        }
        s
    }

    pub fn pgn_notation(&self) -> String {
        match self {
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
        }
    }

    pub fn from_uci_notation(s: &str, game: &ChessGame) -> Result<Self, &'static str> {
        if s.len() != 4 && s.len() != 5 {
            return Err("Invalid move length");
        } else if s.chars().nth(4).is_some_and(|c| c != 'q') {
            return Err("Only promovations to queen are allowed for now");
        } else if s == "e1g1" {
            return Ok(Move::CastlingShort {
                owner: Players::White,
            });
        } else if s == "e8g8" {
            return Ok(Move::CastlingShort {
                owner: Players::Black,
            });
        } else if s == "e1c1" {
            return Ok(Move::CastlingLong {
                owner: Players::White,
            });
        } else if s == "e8c8" {
            return Ok(Move::CastlingLong {
                owner: Players::Black,
            });
        } else {
            let mut chars = s.bytes();
            let start_col = chars.next().unwrap().wrapping_sub(b'a') as i8;
            let start_row = chars.next().unwrap().wrapping_sub(b'1') as i8;
            let end_col = chars.next().unwrap().wrapping_sub(b'a') as i8;
            let end_row = chars.next().unwrap().wrapping_sub(b'1') as i8;

            let start = Position::new(start_row, start_col);
            let end = Position::new(end_row, end_col);

            if start.is_none() || end.is_none() {
                return Err("Invalid position");
            }

            let start = start.unwrap();
            let end = end.unwrap();

            if s.len() == 5 {
                return Ok(Move::Promovation {
                    owner: game.current_player,
                    start,
                    end,
                    captured_piece: *game.get_position(end),
                });
            }

            if let Some(piece) = *game.get_position(start) {
                return Ok(Move::Normal {
                    piece,
                    start,
                    end,
                    captured_piece: *game.get_position(end),
                });
            }

            return Err("Start square is empty");
        }
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
            Move::CastlingLong { owner } => write!(f, "castling long {:?} ", *owner),
            Move::CastlingShort { owner } => write!(f, "castling short {:?} ", *owner),
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
                write!(f, "{} ", i + 1)?;
                row.iter().try_for_each(|place| -> std::fmt::Result {
                    write!(f, "|{}", place.map(|piece| piece.as_char()).unwrap_or(' '))
                })?;
                write!(f, "|\n")
            })?;
        write!(f, "\n   a b c d e f g h\n")
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
