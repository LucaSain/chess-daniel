use super::*;

// Documentation for perft: https://www.chessprogramming.org/Perft_Results

/// Performance Test
fn perft(game: &mut ChessGame, depth: u8) -> usize {
    let mut moves = ArrayVec::new();
    game.get_moves(&mut moves, true);

    if depth == 1 {
        return moves.len();
    }

    let mut count = 0;

    for _move in moves.iter() {
        let _move = *_move;
        game.push(_move);
        count += perft(game, depth - 1);
        game.pop(_move);
    }

    count
}

#[test]
fn perft1_startpos() {
    let mut game = ChessGame::new();
    assert_eq!(perft(&mut game, 1), 20);
}

#[test]
fn perft2_startpos() {
    let mut game = ChessGame::new();
    assert_eq!(perft(&mut game, 2), 400);
}

#[test]
fn perft3_startpos() {
    let mut game = ChessGame::new();
    assert_eq!(perft(&mut game, 3), 8902);
}

#[test]
fn perft4_startpos() {
    let mut game = ChessGame::new();
    assert_eq!(perft(&mut game, 4), 197281);
}

#[test]
fn perft5_startpos() {
    let mut game = ChessGame::new();
    assert_eq!(perft(&mut game, 5), 4865609);
}
