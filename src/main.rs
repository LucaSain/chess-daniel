mod chess_game;
mod move_struct;
mod piece;
mod position;

use arrayvec::ArrayVec;
use chess_game::ChessGame;
use move_struct::Move;

use std::{
    cmp::Ordering,
    io::stdin,
    time::{Duration, Instant},
};

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
            return i32::MIN + 100 + game.len() as i32;
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
        });
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
        });
    }

    let mut best_score = i32::MIN + 10;
    for _move in &moves {
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

fn get_best_move(game: &mut ChessGame, depth: u8) -> (Option<Move>, i32, bool) {
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves, true);

    // If there is only one move available don't bother searching
    if moves.len() == 1 {
        return (moves.first().copied(), 0, true);
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

    (best_move, best_score, false)
}

fn get_best_move_in_time(game: &mut ChessGame, duration: Duration) -> Option<Move> {
    let now = Instant::now();
    let mut last_score: Option<i32> = None;
    for depth in 5..20 {
        let (best_move, best_score, is_only_move) = get_best_move(game, depth);
        let average_score = match last_score {
            Some(score) => (score + best_score) / 2,
            None => best_score,
        };
        last_score = Some(best_score);

        println!("info depth {}", depth);
        println!("info score cp {}", average_score / 100);
        // If mate can be forced, or there is only a single move available, stop searching
        let elapsed_time = now.elapsed();
        if elapsed_time > duration || is_only_move || best_score > i32::MAX - 1000 {
            return best_move;
        }
    }

    unreachable!()
}

fn uci_talk() {
    let mut game = ChessGame::default();
    // Source: https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf
    'main_loop: for line in stdin().lines() {
        let line = line.unwrap();
        let mut terms = line.split_ascii_whitespace();
        while let Some(term) = terms.next() {
            match term {
                "uci" => {
                    println!("id name daniel_chess");
                    println!("id author Malanca Daniel");
                    println!("uciok");
                    continue 'main_loop;
                }
                "isready" => {
                    println!("readyok");
                    continue 'main_loop;
                }
                "position" => {
                    if let Some(term) = terms.next() {
                        match term {
                            "startpos" => {
                                game = ChessGame::default();
                                if let Some(term) = terms.next() {
                                    if term == "moves" {
                                        for move_str in terms.by_ref() {
                                            let _move =
                                                match Move::from_uci_notation(move_str, &game) {
                                                    Ok(_move) => _move,
                                                    Err(_) => continue 'main_loop,
                                                };

                                            let mut moves = ArrayVec::new();
                                            game.get_moves(&mut moves, true);
                                            if moves
                                                .iter()
                                                .any(|allowed_move| _move == *allowed_move)
                                            {
                                                game.push(_move);
                                                // Hard limit onto the number
                                                // of possible moves in a game;
                                                if game.len() >= 400 {
                                                    continue 'main_loop;
                                                }
                                            } else {
                                                continue 'main_loop;
                                            }
                                        }
                                    }
                                }
                            }
                            _ => continue 'main_loop,
                        }
                    } else {
                        continue 'main_loop;
                    }
                }
                "go" => {
                    if let Some(best_move) =
                        get_best_move_in_time(&mut game, Duration::from_millis(2500))
                    {
                        println!("bestmove {}", best_move.uci_notation());
                        game.push(best_move);
                    }
                }
                "quit" => {
                    return;
                }
                _ => continue,
            }
        }
    }
}

fn main() {
    let mut args = std::env::args();
    args.next();
    if let Some(arg) = args.next() {
        if arg == "test" {
            // Generate best moves for a couple different positions
            // This is used for benchmarking and PGO optimization
            let depth = args
                .next()
                .unwrap_or(String::from("7"))
                .parse()
                .unwrap_or(7);
            let mut game = ChessGame::default();
            for i in 3..=depth {
                get_best_move(&mut game, i);
            }

            game =
                ChessGame::new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                    .unwrap();
            for i in 3..=depth {
                get_best_move(&mut game, i);
            }
            game = ChessGame::new("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();
            for i in 3..=depth {
                get_best_move(&mut game, i);
            }
            game = ChessGame::new("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0")
                .unwrap();
            for i in 3..=depth {
                get_best_move(&mut game, i);
            }
            game = ChessGame::new("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap();
            for i in 3..=depth {
                get_best_move(&mut game, i);
            }
            game = ChessGame::new(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            )
            .unwrap();
            for i in 3..=depth {
                get_best_move(&mut game, i);
            }
            return;
        } else if arg == "auto" {
            let mut game = ChessGame::default();
            let time = args.next().unwrap().parse().unwrap();
            loop {
                let mut moves = ArrayVec::new();
                game.get_moves(&mut moves, true);
                println!("{}", game.get_pgn());
                dbg!(game.clone());
                let next_move = match get_best_move_in_time(&mut game, Duration::from_millis(time))
                {
                    Some(_move) => _move,
                    None => break,
                };
                game.push_history(next_move);
            }
            return;
        }
    }
    // Enter UCI mode
    uci_talk();
}

#[cfg(test)]
mod performance_test;
