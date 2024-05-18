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

fn get_best_move(game: &mut ChessGame, depth: usize) -> (Option<Move>, i32) {
    if depth == 0 {
        // unsafe { COUNT += 1 }
        return (None, game.score);
    }

    let player = game.current_player;
    let moves = game.get_moves();
    let iter = moves.iter().map(|_move| {
        game.push(*_move);
        let best_move = get_best_move(game, depth - 1);
        game.pop(*_move);
        (Some(*_move), best_move.1)
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
    let mut game = ChessGame::new();
    let num_moves = std::env::var("MOVES")
        .unwrap_or("0".to_owned())
        .parse()
        .unwrap_or(0);
    if num_moves > 0 {
        if num_moves == 11 {
            for _ in 0..20 {
                let _move = get_best_move(&mut game, 3);
                game.push(_move.0.unwrap());
            }
            let _move = get_best_move(&mut game, 6);
            game.push(_move.0.unwrap());
        } else {
            let _move = get_best_move(&mut game, 6);
            game.push(_move.0.unwrap());
            // unsafe {
            //     dbg!(COUNT);
            // }
        }
        return;
    }
    loop {
        // let depth = std::env::var("DEPTH").unwrap().parse().unwrap();
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
        let depth = 6;
        let _move = get_best_move(&mut game, depth);
        // dbg!(game.get_moves());
        // dbg!(game.move_stack.clone());
        game.push_history(_move.0.unwrap());
        println!("{}", game.get_pgn());
        dbg!(_move.0.unwrap());
        dbg!(game.score);
        dbg!(game.clone());
    }
    // let _move = loop {
    //     let mut val = String::new();
    //     std::io::stdin()
    //         .read_line(&mut val)
    //         .expect("Failed to read line");
    //     let mut substr_iter = val.split_whitespace();
    //     let mut next_num = || -> Result<i8, _> {
    //         substr_iter
    //             .next()
    //             .unwrap_or("Not enough input numbers")
    //             .parse()
    //     };

    //     let val1 = next_num().unwrap_or(0);

    //     let val2 = next_num().unwrap_or(0);

    //     let val3 = next_num().unwrap_or(0);
    //     let val4 = next_num().unwrap_or(0);

    //     let _move = game.get_moves().into_iter().find(|_move| match _move {
    //         Move::Normal { start, end, .. } => {
    //             *start == Position::new(val1, val2) && *end == Position::new(val3, val4)
    //         }
    //         Move::CastlingShort { .. } => val1 == 10,
    //         Move::CastlingLong { .. } => val1 == 20,
    //         Move::Promovation { start, end, .. } => {
    //             *start == Position::new(val1, val2) && *end == Position::new(val3, val4)
    //         }
    //     });

    //     match _move {
    //         Some(_move) => {
    //             dbg!(_move);
    //             game.push(_move);
    //             break;
    //         }
    //         None => {
    //             if (val1 == -1) {
    //                 game.pop();
    //                 game.pop();
    //                 dbg!(game.score());
    //                 dbg!(game.clone());
    //             }
    //             continue;
    //         }
    //     }
    // };
    // dbg!(game.score());
    // dbg!(game.clone());
}
