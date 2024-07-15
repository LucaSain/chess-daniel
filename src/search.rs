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

fn simple_sort(a: &Move, b: &Move) -> Ordering {
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
        // SAFETY: Length is 1
        let _move = unsafe { *moves.get_unchecked(0) };
        game.push(_move);
        let score = -get_best_move_score_depth_1(game, -beta, -alpha);
        game.pop(_move);
        return score;
    }

    for _move in &moves {
        let _move = *_move;
        game.push_depth_1(_move);
        let score = -game.score * (game.current_player as Score);
        game.pop_depth_1(_move);

        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }

    alpha
}
fn get_best_move_score_depth_2(game: &mut ChessGame, mut alpha: Score, beta: Score) -> Score {
    let player = game.current_player;
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves, true);

    if moves.is_empty() {
        if !game.is_targeted(game.get_king_position(player), player) {
            return 0;
        } else {
            // The earlier the mate the worse the score for the losing player
            return Score::MIN + 100 + game.len() as Score;
        }
    } else if moves.len() == 1 {
        // If there is only one move available push it and don't decrease depth
        // SAFETY: Length is 1
        let _move = unsafe { *moves.get_unchecked(0) };
        game.push(_move);
        let score = -get_best_move_score_depth_2(game, -beta, -alpha);
        game.pop(_move);
        return score;
    }

    // We want to sort the moves best on the most likely ones to be good
    moves.sort_unstable_by(simple_sort);

    for _move in &moves {
        let _move = *_move;
        game.push(_move);
        let score = -get_best_move_score_depth_1(game, -beta, -alpha);
        game.pop(_move);
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }

    alpha
}

fn get_best_move_score(
    game: &mut ChessGame,
    should_stop: &AtomicBool,
    depth: u8,
    mut alpha: Score,
    beta: Score,
) -> Result<Score, ()> {
    if should_stop.load(atomic::Ordering::Relaxed) {
        // Halt the search early
        return Err(());
    }

    if depth == 2 {
        return Ok(get_best_move_score_depth_2(game, alpha, beta));
    } else if depth == 1 {
        return Ok(get_best_move_score_depth_1(game, alpha, beta));
    } else if depth == 0 {
        return Ok(game.score * (game.current_player as Score));
    }

    let player = game.current_player;
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves, true);

    if moves.is_empty() {
        if !game.is_targeted(game.get_king_position(player), player) {
            return Ok(0);
        } else {
            // The earlier the mate the worse the score for the losing player
            return Ok(Score::MIN + 100 + game.len() as Score);
        }
    } else if moves.len() == 1 {
        // If there is only one move available push it and don't decrease depth
        // SAFETY: Length is 1
        let _move = unsafe { *moves.get_unchecked(0) };
        game.push(_move);
        let score = -get_best_move_score(game, should_stop, depth, -beta, -alpha)?;
        game.pop(_move);
        return Ok(score);
    }

    // We want to sort the moves best on the most likely ones to be good
    if depth >= 5 {
        moves.sort_by_cached_key(|a| {
            game.push(*a);
            let score = get_best_move_score(game, should_stop, depth - 5, -beta, -alpha);
            game.pop(*a);
            score
        });
    } else {
        moves.sort_unstable_by(simple_sort);
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

    Ok(alpha)
}

pub fn get_best_move(
    mut game: ChessGame,
    should_stop: &AtomicBool,
    depth: u8,
) -> Result<(Option<Move>, Score, bool), ()> {
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves, true);

    // If there is only one move available don't bother searching
    if moves.len() == 1 {
        return Ok((moves.first().copied(), 0, true));
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

    Ok((best_move, best_score, false))
}

pub fn get_best_move_in_time(game: &ChessGame, duration: Duration) -> Option<Move> {
    let mut last_score: Option<Score> = None;
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
        let Ok((best_move, best_score, is_only_move)) =
            get_best_move(game.clone(), should_stop.as_ref(), depth)
        else {
            return found_move;
        };

        found_move = best_move;

        let average_score = match last_score {
            Some(score) => (score + best_score) / 2,
            None => best_score,
        };
        last_score = Some(best_score);

        println!("info depth {}", depth);
        println!("info score cp {}", average_score);
        // If mate can be forced, or there is only a single move available, stop searching
        if is_only_move || best_score > Score::MAX - 1000 {
            return found_move;
        }
    }

    unreachable!()
}
