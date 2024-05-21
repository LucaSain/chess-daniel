use std::cmp::Ordering;

use arrayvec::ArrayVec;
use chess::*;

fn get_best_move_score(
    game: &mut ChessGame,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    last_capture: Option<Position>,
) -> i32 {
    if depth <= 0 {
        return game.score * (game.current_player as i32);
    }
    let player = game.current_player;

    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves);

    if depth >= 3 {
        moves = moves
            .into_iter()
            .filter(|_move| {
                game.push(*_move);
                let condition = !game.is_targeted(game.get_king_position(player), player);
                game.pop(*_move);
                condition
            })
            .collect();
    }

    if moves.is_empty() {
        if !game.is_targeted(game.get_king_position(player), player) {
            return 0;
        } else {
            // The earlier the mate the worse the score for the losing player
            return i32::MIN + 100 - depth as i32;
        }
    } else if moves.len() == 1 {
        // If there is only one move available push it and don't decrease depth
        // SAFETY: Length is 1
        let _move = unsafe { *moves.get_unchecked(0) };
        let capture = match _move {
            Move::Normal {
                end,
                captured_piece,
                ..
            } => captured_piece.map(|_| end),
            _ => None,
        };
        game.push(_move);
        let score = -get_best_move_score(game, depth, -beta, -alpha, capture);
        game.pop(_move);
        return score;
    }

    // We want to sort the moves best on the most likely ones to be good
    if depth >= 5 {
        moves.sort_by_cached_key(|a| {
            game.push(*a);
            let score = get_best_move_score(game, depth - 5, -beta, -alpha, None);
            game.pop(*a);
            score
        })
    } else if depth >= 2 {
        moves.sort_unstable_by(|a, b| match a {
            Move::Normal {
                captured_piece: capture_a,
                piece: piece_a,
                ..
            } => match b {
                Move::Normal {
                    captured_piece: capture_b,
                    piece: piece_b,
                    end: end_b,
                    ..
                } => {
                    if let Some(cap_piece_a) = capture_a {
                        if let Some(cap_piece_b) = capture_b {
                            if let Some(pos) = last_capture {
                                if pos == *end_b {
                                    return Ordering::Greater;
                                }
                            }

                            if cap_piece_a != cap_piece_b {
                                return cap_piece_a.piece_type.cmp(&cap_piece_b.piece_type);
                            }

                            return piece_a.piece_type.cmp(&piece_b.piece_type);
                        }
                        return Ordering::Less;
                    } else if capture_b.is_some() {
                        return Ordering::Greater;
                    }
                    piece_b.piece_type.cmp(&piece_a.piece_type)
                }

                _ => Ordering::Greater,
            },

            _ => Ordering::Less,
        })
    }

    let mut best_score = -1000000000;
    for _move in moves.iter() {
        let _move = *_move;
        let capture = match _move {
            Move::Normal {
                end,
                captured_piece,
                ..
            } => captured_piece.map(|_| end),
            _ => None,
        };
        game.push(_move);
        best_score = best_score.max(-get_best_move_score(
            game,
            depth - 1,
            -beta,
            -alpha,
            capture,
        ));
        game.pop(_move);
        alpha = alpha.max(best_score);
        if alpha >= beta {
            break;
        }
    }

    best_score
}

fn get_best_move(game: &mut ChessGame, depth: u8) -> (Option<Move>, i32) {
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves);

    let player = game.current_player;
    moves = moves
        .into_iter()
        .filter(|_move| {
            game.push(*_move);
            let condition = !game.is_targeted(game.get_king_position(player), player);
            game.pop(*_move);
            condition
        })
        .collect();

    // If there is only one move available don't bother searching
    if moves.len() == 1 {
        return (Some(moves[0]), 0);
    }

    let mut best_move = None;
    let mut best_score = -i32::MAX;

    for _move in moves {
        game.push(_move);
        // Initially alpha == beta
        let score = -get_best_move_score(game, depth - 1, i32::MIN + 1, -best_score, None);
        game.pop(_move);
        if score > best_score {
            best_score = score;
            best_move = Some(_move);
        }
    }

    (best_move, best_score)
}

fn main() {
    let mut args = std::env::args();
    args.next();
    let arg = args.next().unwrap();
    let depth = args.next().unwrap().parse().unwrap();
    let mut game = ChessGame::new();

    if arg == "test" {
        let _move = get_best_move(&mut game, depth);
    } else if arg == "auto" {
        loop {
            let mut moves = ArrayVec::new();
            game.get_moves(&mut moves);
            println!("{}", game.get_pgn());
            dbg!(game.clone());
            let _move = get_best_move(&mut game, depth);
            dbg!(_move.1);
            let next_move = _move.0.unwrap();
            game.push_history(next_move);
        }
    } else if arg == "play" {
        loop {
            println!("{}", game.get_pgn());
            dbg!(game.clone());
            let _move = get_best_move(&mut game, depth);
            game.push_history(_move.0.unwrap());

            println!("{}", game.get_pgn());
            dbg!(game.clone());

            loop {
                let mut val = String::new();
                std::io::stdin()
                    .read_line(&mut val)
                    .expect("Failed to read line");
                let mut substr_iter = val.split_whitespace();
                let mut next_num =
                    || -> Result<i8, _> { substr_iter.next().unwrap_or("...").parse() };

                let val1 = next_num().unwrap_or(0);
                let val2 = next_num().unwrap_or(0);

                let val3 = next_num().unwrap_or(0);
                let val4 = next_num().unwrap_or(0);

                let pos1 = Position::new(val1, val2);
                let pos2 = Position::new(val3, val4);

                if pos1.is_none() || pos2.is_none() {
                    if val1 == -1 {
                        game.pop_history();
                        game.pop_history();
                        dbg!(game.clone());
                        continue;
                    }
                }

                let pos1 = pos1.unwrap_or(Position::new(0, 0).unwrap());
                let pos2 = pos2.unwrap_or(Position::new(0, 0).unwrap());

                let mut moves = ArrayVec::new();
                game.get_moves(&mut moves);
                let _move = moves.iter().find(|_move| match _move {
                    Move::Normal { start, end, .. } => *start == pos1 && *end == pos2,
                    Move::Promovation { start, end, .. } => *start == pos1 && *end == pos2,
                    Move::CastlingShort { .. } => val1 == 10,
                    Move::CastlingLong { .. } => val1 == 20,
                    Move::EnPassant {
                        start_col, end_col, ..
                    } => *start_col == pos1.col() && *end_col == pos2.col(),
                });

                if let Some(_move) = _move {
                    game.push_history(*_move);
                    break;
                }
            }
        }
    } else if arg == "manual" {
        loop {
            println!("{}", game.get_pgn());
            dbg!(game.clone());

            loop {
                let mut val = String::new();
                std::io::stdin()
                    .read_line(&mut val)
                    .expect("Failed to read line");
                let mut substr_iter = val.split_whitespace();
                let mut next_num =
                    || -> Result<i8, _> { substr_iter.next().unwrap_or("...").parse() };

                let val1 = next_num().unwrap_or(0);
                let val2 = next_num().unwrap_or(0);

                let val3 = next_num().unwrap_or(0);
                let val4 = next_num().unwrap_or(0);

                let pos1 = Position::new(val1, val2);
                let pos2 = Position::new(val3, val4);

                if pos1.is_none() || pos2.is_none() {
                    if val1 == -1 {
                        game.pop_history();
                        game.pop_history();
                        dbg!(game.clone());
                        continue;
                    }
                }

                let pos1 = pos1.unwrap_or(Position::new(0, 0).unwrap());
                let pos2 = pos2.unwrap_or(Position::new(0, 0).unwrap());

                let mut moves = ArrayVec::new();
                game.get_moves(&mut moves);
                let _move = moves.iter().find(|_move| match _move {
                    Move::Normal { start, end, .. } => *start == pos1 && *end == pos2,
                    Move::Promovation { start, end, .. } => *start == pos1 && *end == pos2,
                    Move::CastlingShort { .. } => val1 == 10,
                    Move::CastlingLong { .. } => val1 == 20,
                    Move::EnPassant {
                        start_col, end_col, ..
                    } => *start_col == pos1.col() && *end_col == pos2.col(),
                });

                if let Some(_move) = _move {
                    game.push_history(*_move);
                    break;
                }
            }
        }
    }
}
