use std::time::Duration;

use arrayvec::ArrayVec;

use crate::{chess_game::ChessGame, search::get_best_move_in_time};

pub fn autoplay(millis: u64) {
    let mut game = ChessGame::default();

    loop {
        let mut moves = ArrayVec::new();
        game.get_moves(&mut moves, true);
        println!("{}", game.get_pgn());
        println!("{}", &game);
        let next_move = match get_best_move_in_time(&game, Duration::from_millis(millis)) {
            Some(_move) => _move,
            None => break,
        };
        game.push_history(next_move);
    }
}
