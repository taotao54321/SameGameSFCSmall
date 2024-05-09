//! 初形で手詰まりになる面を探す。

use samegame_sfc_small::*;

fn main() {
    for (state, counter, inc_timing, board) in enumerate_boards() {
        let pos = Position::new(board);
        if pos.actions().next().is_none() {
            println!("0x{state:04X}\t0x{counter:02X}\t{inc_timing}");
        }
    }
}

fn enumerate_boards() -> impl Iterator<Item = (u16, u8, usize, Board)> {
    itertools::iproduct!(0..=u16::MAX, 0..=u8::MAX, 0..=48).filter_map(
        |(state, counter, inc_timing)| {
            let board = GameRng::new(state).gen_board(counter, inc_timing)?;
            Some((state, counter, inc_timing, board))
        },
    )
}
