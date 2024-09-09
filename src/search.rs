use std::{
    cmp::Ordering,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    thread,
    time::Duration,
};

use arrayvec::ArrayVec;

use crate::{chess_game::ChessGame, move_struct::Move, piece::Score};

/// Very simple comparing function to determine which moves are likely to be better
fn simple_move_compare(a: &Move, b: &Move) -> Ordering {
    match a {
        Move::Normal {
            captured_piece: capture_a,
            piece: piece_a,
            ..
        } => match b {
            Move::Normal {
                captured_piece: capture_b,
                piece: piece_b,
                ..
            } => {
                if let Some(cap_piece_a) = capture_a {
                    if let Some(cap_piece_b) = capture_b {
                        if cap_piece_a != cap_piece_b {
                            return cap_piece_a.piece_type.cmp(&cap_piece_b.piece_type);
                        }
                        return piece_b.piece_type.cmp(&piece_a.piece_type);
                    }
                    return Ordering::Less;
                } else if capture_b.is_some() {
                    return Ordering::Greater;
                }
                piece_a.piece_type.cmp(&piece_b.piece_type)
            }
            Move::Promotion { .. } => Ordering::Greater,
            _ => Ordering::Less,
        },
        Move::Promotion { .. } => Ordering::Less,
        _ => Ordering::Greater,
    }
}

/// This function exists in order to improve the performance of the search algorithm
/// It's the same as get_best_move_score but with a depth always equal to 1
///
/// Explanation: due to the nature of the search tree (exponential growth), the majority
/// of the time is spent in this function, so it's eliminating unnecessary branches
fn get_best_move_score_depth_1(game: &mut ChessGame, mut alpha: Score, beta: Score) -> Score {
    let player = game.current_player;
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves, false);

    if moves.is_empty() {
        if !game.is_targeted(game.get_king_position(player), player) {
            return 0;
        } else {
            // The earlier the mate the worse the score for the losing player
            return Score::MIN + 100 + game.len() as Score;
        }
    } else if moves.len() == 1 {
        // If there is only one move available push it and don't decrease depth
        let _move = moves[0];
        game.push(_move);
        let score = -get_best_move_score_depth_1(game, -beta, -alpha);
        game.pop(_move);
        return score;
    }

    for _move in &moves {
        let _move = *_move;
        game.push(_move);
        let score = -game.score * (game.current_player as Score);
        game.pop(_move);

        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }

    alpha
}

/// Core function of the alpha beta search algorithm
/// It halts early and returns None if the should_stop flag is set
/// Otherwise returns the best score for the current player
fn get_best_move_score(
    game: &mut ChessGame,
    should_stop: &AtomicBool,
    depth: u8,
    mut alpha: Score,
    beta: Score,
) -> Option<Score> {
    if should_stop.load(atomic::Ordering::Relaxed) {
        // Halt the search early
        return None;
    }

    if depth == 1 {
        return Some(get_best_move_score_depth_1(game, alpha, beta));
    } else if depth == 0 {
        return Some(game.score * (game.current_player as Score));
    }

    let player = game.current_player;
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves, true);

    if moves.is_empty() {
        if !game.is_targeted(game.get_king_position(player), player) {
            return Some(0);
        } else {
            // The earlier the mate the worse the score for the losing player
            return Some(Score::MIN + 100 + game.len() as Score);
        }
    } else if moves.len() == 1 {
        // If there is only one move available push it and don't decrease depth
        let _move = moves[0];
        game.push(_move);
        let score = -get_best_move_score(game, should_stop, depth, -beta, -alpha)?;
        game.pop(_move);
        return Some(score);
    }

    // We want to sort the moves best on the most likely ones to be good
    if depth >= 5 {
        moves.sort_by_cached_key(|a| {
            game.push(*a);
            let score = get_best_move_score(game, should_stop, depth - 5, -beta, -alpha);
            game.pop(*a);
            score
        });
    } else if depth >= 2 {
        moves.sort_unstable_by(simple_move_compare);
    }

    for _move in &moves {
        let _move = *_move;
        game.push(_move);
        let score = -get_best_move_score(game, should_stop, depth - 1, -beta, -alpha)?;
        game.pop(_move);

        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }

    Some(alpha)
}

/// This function is the entry point for the search algorithm
/// It returns the best move, the score of the best move
/// and a flag indicating if there is only one move available
pub fn get_best_move(
    mut game: ChessGame,
    should_stop: &AtomicBool,
    depth: u8,
) -> Option<(Option<Move>, Score, bool)> {
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves, true);

    // If there is only one move available don't bother searching
    if moves.len() == 1 {
        return Some((moves.first().copied(), 0, true));
    }

    let mut best_move = None;
    let mut best_score = -Score::MAX;

    for _move in moves {
        game.push(_move);
        // Initially alpha == beta
        let score = -get_best_move_score(
            &mut game,
            should_stop,
            depth - 1,
            Score::MIN + 1,
            -best_score,
        )?;
        game.pop(_move);
        if score > best_score {
            best_score = score;
            best_move = Some(_move);
        }
    }

    Some((best_move, best_score, false))
}

/// This function repeatedly calls get_best_move with increasing depth,
/// until the time limit is reached, at which point it returns the best move found so far
pub fn get_best_move_in_time(game: &ChessGame, duration: Duration) -> Option<Move> {
    let mut found_move = None;

    // Stop searching after the duration has passed
    let should_stop = Arc::new(AtomicBool::new(false));
    thread::spawn({
        let should_stop = should_stop.clone();
        move || {
            thread::sleep(duration);
            should_stop.store(true, atomic::Ordering::Relaxed);
        }
    });

    for depth in 5.. {
        let Some((best_move, best_score, is_only_move)) =
            get_best_move(game.clone(), should_stop.as_ref(), depth)
        else {
            return found_move;
        };

        found_move = best_move;

        println!("info depth {}", depth);
        println!("info score cp {}", best_score);

        // If mate can be forced, or there is only a single move available, stop searching
        if is_only_move || best_score > Score::MAX - 1000 {
            return found_move;
        }
    }

    unreachable!()
}
