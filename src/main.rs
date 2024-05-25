mod chess_game;
mod piece;
mod position;

use arrayvec::ArrayVec;
use chess_game::*;

use std::{cmp::Ordering, io::stdin};

fn get_best_move_score(game: &mut ChessGame, depth: u8, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return game.score * (game.current_player as i32);
    }
    let player = game.current_player;
    let state = *game.state();
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves, depth >= 3);

    if moves.is_empty() {
        if !game.is_targeted(game.get_king_position(player), player) {
            return 0;
        } else {
            // The earlier the mate the worse the score for the losing player
            return i32::MIN + game.len() as i32;
        }
    } else if moves.len() == 1 {
        // If there is only one move available push it and don't decrease depth
        // SAFETY: Length is 1
        let _move = unsafe { *moves.get_unchecked(0) };
        game.push(_move);
        let score = -get_best_move_score(game, depth, -beta, -alpha);
        game.pop(_move);
        return score;
    }

    // We want to sort the moves best on the most likely ones to be good
    if depth >= 5 {
        moves.sort_by_cached_key(|a| {
            game.push(*a);
            let score = get_best_move_score(game, depth - 5, -beta, -alpha);
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
                            if let Some(pos) = state.last_position {
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
        game.push(_move);
        best_score = best_score.max(-get_best_move_score(game, depth - 1, -beta, -alpha));
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
    game.get_moves(&mut moves, true);

    // If there is only one move available don't bother searching
    if moves.len() == 1 {
        return (Some(moves[0]), 0);
    }

    let mut best_move = None;
    let mut best_score = -i32::MAX;

    for _move in moves {
        game.push(_move);
        // Initially alpha == beta
        let score = -get_best_move_score(game, depth - 1, i32::MIN + 1, -best_score);
        game.pop(_move);
        if score > best_score {
            best_score = score;
            best_move = Some(_move);
        }
    }

    (best_move, best_score)
}

fn uci_talk() {
    // Source: https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf
    let mut lines = stdin().lines();
    // Read "uci"
    lines.next().unwrap().unwrap();
    println!("id name daniel_chess");
    println!("id author Malanca Daniel");
    println!("uciok");
    lines.next().unwrap().unwrap();
    println!("readyok");
    lines.next().unwrap().unwrap();
    lines.next().unwrap().unwrap();
    println!("readyok");
    loop {
        let mut game = ChessGame::default();
        let line = lines.next().unwrap().unwrap();
        let mut words = line.split_whitespace();
        words.next().unwrap();
        words.next().unwrap();
        if words.next().is_some() {
            for move_str in words {
                let _move = Move::from_uci_notation(move_str, &game).unwrap();
                let mut moves = ArrayVec::new();
                game.get_moves(&mut moves, true);
                if moves.iter().any(|allowed_move| _move == *allowed_move) {
                    game.push_history(_move);
                } else {
                    break;
                }
            }
        }
        lines.next().unwrap().unwrap();
        let best_move = get_best_move(&mut game, 9).0.unwrap();
        dbg!(game.clone());
        println!("bestmove {}", best_move.uci_notation());
        game.push_history(best_move);
    }
}

fn main() {
    let mut args = std::env::args();
    args.next();
    let mut game = ChessGame::default();
    let next_arg = args.next();
    if next_arg.is_none() {
        uci_talk();
        return;
    }

    let arg = next_arg.unwrap();
    let depth = args.next().unwrap().parse().unwrap();

    if arg == "test" {
        let _move = get_best_move(&mut game, depth);
    } else if arg == "auto" {
        loop {
            let mut moves = ArrayVec::new();
            game.get_moves(&mut moves, true);
            println!("{}", game.get_pgn());
            dbg!(game.clone());
            let _move = get_best_move(&mut game, depth);
            dbg!(_move.1);
            let next_move = _move.0.unwrap();
            game.push_history(next_move);
        }
    }
}

#[cfg(test)]
mod performance_test;
