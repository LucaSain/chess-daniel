use std::collections::BTreeMap;

use chess::*;

const NUM_MOVES: usize = 30;

fn get_best_move(game: &mut ChessGame, depth: usize) -> (Option<Move>, f64) {
    if depth == 0 {
        return (None, game.score());
    }

    let player = game.current_player;
    let iter = game.get_moves().into_iter().map(|_move| {
        game.push(_move);
        let best_move = get_best_move(game, depth - 1);
        game.pop();
        (Some(_move), best_move.1)
    });

    match player {
        Players::White => iter
            .max_by(|move1, move2| move1.1.total_cmp(&move2.1))
            .unwrap_or(if game.is_targeted(game.king_positions[player as usize]) {
                (None, -1000.0)
            } else {
                (None, 0.0)
            }),
        Players::Black => iter
            .min_by(|move1, move2| move1.1.total_cmp(&move2.1))
            .unwrap_or(if game.is_targeted(game.king_positions[player as usize]) {
                (None, 1000.0)
            } else {
                (None, 0.0)
            }),
    }
}

fn get_best_move_main(game: &mut ChessGame, depth: usize) -> Option<Move> {
    let mut best_moves = Vec::<Vec<Move>>::with_capacity(1000);
    game.get_moves()
        .into_iter()
        .for_each(|_move| best_moves.push(vec![_move]));

    for k in 0..depth {
        let mut new_moves = BTreeMap::<i64, Vec<Move>>::new();
        best_moves.iter().enumerate().for_each(|(i, vec)| {
            vec.iter().for_each(|_move| game.push(*_move));

            game.get_moves()
                .into_iter()
                .enumerate()
                .for_each(|(j, _move)| {
                    let score = get_best_move(game, 3).1;
                    let mut new_vec = vec.clone();
                    new_vec.push(_move);
                    new_moves.insert(
                        (score * 100000.0) as i64 + depth as i64 * 100 + i as i64 * 10 + j as i64,
                        new_vec,
                    );
                });

            vec.iter().for_each(|_| {
                game.pop();
            });
        });

        let best_paths: Vec<_> = match game.current_player {
            Players::White => new_moves
                .into_iter()
                .rev()
                .take(NUM_MOVES)
                // .map(|(_, moves)| moves)
                .collect(),
            Players::Black => new_moves
                .into_iter()
                .take(NUM_MOVES)
                // .map(|(_, moves)| moves)
                .collect(),
        };
        if k == 0 {
            if game.current_player == Players::White && best_paths[0].0 > 999000000 {
                return best_paths[0].1.first().map(|x| *x);
            } else if game.current_player == Players::Black && best_paths[0].0 < -999000000 {
                return best_paths[0].1.first().map(|x| *x);
            }
        }
        best_moves = best_paths.into_iter().map(|(_, moves)| moves).collect();
    }
    best_moves[0].first().map(|x| *x)
}

fn main() {
    let mut game = ChessGame::new();
    loop {
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

        // let _move = get_best_move_main(&mut game, 6);
        let _move = get_best_move(&mut game, 6).0;
        game.push(_move.unwrap());
        dbg!(_move.unwrap());
        dbg!(game.score());
        dbg!(game.clone());
    }
}
