use arrayvec::ArrayVec;
use chess::*;
// use std::collections::{BTreeMap, HashMap, HashSet};
// static mut COUNT: usize = 0;

// fn count_moves(game: &mut ChessGame, depth: usize) -> usize {
//     if depth == 0 {
//         return 1;
//     }
//     let moves = game.get_moves();

//     moves
//         .iter()
//         .map(|_move| {
//             game.push(*_move);
//             let count = count_moves(game, depth - 1);
//             game.pop(*_move);
//             count
//         })
//         .fold(0, |acc, num| acc + num)
// }

fn get_best_move_score(game: &mut ChessGame, depth: u8) -> i32 {
    if depth == 0 {
        return game.score;
    }

    let player = game.current_player;
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves);
    let iter = moves.iter().map(|_move| {
        game.push(*_move);
        let best_score = get_best_move_score(game, depth - 1);
        game.pop(*_move);
        best_score
    });

    match player {
        Players::White => iter.max().unwrap_or(i32::MIN),
        Players::Black => iter.min().unwrap_or(i32::MAX),
    }
}

fn get_best_move(game: &mut ChessGame, depth: u8) -> (Option<Move>, i32) {
    let player = game.current_player;
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves);
    let iter = moves.iter().map(|_move| {
        game.push(*_move);
        let best_move = get_best_move_score(game, depth - 1);
        game.pop(*_move);
        (Some(*_move), best_move)
    });

    let mut best_move = None;
    let mut best_score;

    match player {
        Players::White => {
            best_score = i32::MIN;
            for (_move, score) in iter {
                if score > best_score {
                    best_move = _move;
                    best_score = score;
                }
            }
        }
        Players::Black => {
            best_score = i32::MAX;
            for (_move, score) in iter {
                if score < best_score {
                    best_move = _move;
                    best_score = score;
                }
            }
        }
    }

    (best_move, best_score)
}

// fn get_best_move_main(game: &mut ChessGame, depth: usize) -> Option<Move> {
//     let mut best_moves = Vec::<Vec<Move>>::with_capacity(1000);
//     game.get_moves()
//         .into_iter()
//         .for_each(|_move| best_moves.push(vec![_move]));

//     for k in 0..depth {
//         let mut new_moves = BTreeMap::<i64, (&[Move], Move)>::new();
//         best_moves.iter().enumerate().for_each(|(i, vec)| {
//             vec.iter().for_each(|_move| game.push(*_move));

//             game.get_moves()
//                 .into_iter()
//                 .enumerate()
//                 .for_each(|(j, _move)| {
//                     let score = get_best_move(game, 4).1;
//                     // let mut new_vec = vec.clone();
//                     // new_vec.push((&vec, _move));
//                     new_moves.insert(
//                         (score * 100000.0) as i64 + depth as i64 * 100 + i as i64 * 10 + j as i64,
//                         (&vec, _move),
//                     );
//                 });

//             vec.iter().for_each(|_| {
//                 game.pop();
//             });
//         });

//         // if k == 0 {
//         //     if game.current_player == Players::White && new_moves.first_key_value().unwrap().first().unwrap()[0].0 > 90000000 {
//         //         return best_paths[0].1 .0.first().map(|x| *x);
//         //     } else if game.current_player == Players::Black && best_paths.last().unwrap.0 < -90000000 {
//         //         return best_paths[0].1 .0.first().map(|x| *x);
//         //     }
//         // }
//         // let mut taken_moves = HashMap::<Move, u8>::new();
//         // // taken_moves.in (*best_moves.first().unwrap().first().unwrap());
//         // let best_paths: Vec<_> = match game.current_player {
//         //     Players::White => new_moves.into_iter().rev().fpr_each(_z)//.take(NUM_MOVES).collect(),
//         //     Players::Black => new_moves.into_iter().take(NUM_MOVES).collect(),
//         // };

//         // best_moves = best_paths
//         //     .into_iter()
//         //     .map(|(_, (moves, _move))| {
//         //         let mut new_moves = moves.to_owned();
//         //         new_moves.push(_move);
//         //         new_moves
//         //     })
//         //     .collect();

//         dbg!(best_moves
//             .iter()
//             .map(|x| x.first().unwrap())
//             .collect::<Vec<_>>());
//     }
//     best_moves[0].first().map(|x| *x)
// }

fn main() {
    let mut args = std::env::args();
    args.next();
    let arg = args.next().unwrap();
    let depth = args.next().unwrap().parse().unwrap();
    let mut game = ChessGame::new();

    if arg == "test" {
        let _move = get_best_move(&mut game, depth);
        return;
    } else if arg == "auto" {
        loop {
            println!("{}", game.get_pgn());
            dbg!(game.clone());
            let _move = get_best_move(&mut game, depth);
            game.push_history(_move.0.unwrap());
        }
    } else if arg == "play" {
        loop {
            // let avg_moves = (count_moves(&mut game, 4) as f64).powf(1.0 / 4.0);

            // let mut move_count = 1.0;
            // let mut depth = 0;
            // for i in 0.. {
            //     move_count *= avg_moves;
            //     if move_count > 200_000_000.0 {
            //         depth = i;
            //         break;
            //     }
            // }

            // if depth < 3 {
            //     depth = 3;
            // }
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

                let pos1 = pos1.unwrap();
                let pos2 = pos2.unwrap();

                let mut moves = ArrayVec::new();
                game.get_moves(&mut moves);
                let _move = moves.iter().find(|_move| match _move {
                    Move::Normal { start, end, .. } => *start == pos1 && *end == pos2,
                    Move::Promovation { start, end, .. } => *start == pos1 && *end == pos2,
                    Move::CastlingShort { .. } => val1 == 10,
                    Move::CastlingLong { .. } => val1 == 20,
                });

                if let Some(_move) = _move {
                    game.push_history(*_move);
                    break;
                }
            }
        }
    }
}
