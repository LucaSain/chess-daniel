use std::{io::stdin, time::Duration};

use arrayvec::ArrayVec;

use crate::{chess_game::ChessGame, move_struct::Move, search::get_best_move_in_time};

pub fn uci_talk() {
    let mut game = ChessGame::default();
    let time_per_move = Duration::from_millis(
        std::env::var("CHESS_TIME_PER_MOVE")
            .unwrap_or("5000".to_string())
            .parse()
            .unwrap_or(5000),
    );

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
                            "fen" => {
                                // TODO: I think it's possible to also get moves
                                // starting from this position
                                // i.e. position fen <fen> moves <moves>
                                if let Ok(fen_game) =
                                    ChessGame::new(terms.remainder().unwrap_or_default())
                                {
                                    game = fen_game;
                                }
                            }
                            _ => continue 'main_loop,
                        }
                    } else {
                        continue 'main_loop;
                    }
                }
                "go" => {
                    if let Some(best_move) = get_best_move_in_time(&mut game, time_per_move) {
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
