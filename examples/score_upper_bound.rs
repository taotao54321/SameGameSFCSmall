//! ゲーム内で出現しうる初期局面集合に対するスコア上界を雑に見積もる。

use samegame_sfc_small::*;

fn main() -> anyhow::Result<()> {
    /*
    for (seed, counter, inc_counter_after, board) in enumerate_boards() {
        let score_ub = score_upper_bound(&board);
        println!("0x{seed:04X}\t0x{counter:02X}\t{inc_counter_after}\t{score_ub}");
    }
    */

    let score_ub_max = enumerate_boards()
        .map(|(_, _, _, board)| score_upper_bound(&board))
        .max()
        .unwrap();
    println!("{score_ub_max}");

    Ok(())
}

fn enumerate_boards() -> impl Iterator<Item = (u16, u8, usize, Board)> {
    itertools::iproduct!(0..=u16::MAX, 0..=u8::MAX, 0..=48).filter_map(
        |(seed, counter, inc_counter_after)| {
            let board = gen_board(seed, counter, inc_counter_after)?;
            Some((seed, counter, inc_counter_after, board))
        },
    )
}

fn gen_board(seed: u16, counter: u8, inc_counter_after: usize) -> Option<Board> {
    GameRng::new(seed).gen_board(counter, inc_counter_after)
}

fn score_upper_bound(board: &Board) -> Score {
    // 2 個以上存在する駒種全てが 1 手で全消しできると仮定して上界を求める。
    // 適宜パーフェクトボーナスも加算する。

    let mut res = 0;
    let mut perfect = true;
    for piece in Piece::all() {
        let count = board.piece_count(piece);
        match count {
            0 => {}
            1 => perfect = false,
            _ => res += score_erase(count),
        }
    }

    if perfect {
        res += SCORE_PERFECT;
    }

    res
}
