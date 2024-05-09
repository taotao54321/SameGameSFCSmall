//! ゲーム内乱数関連。

use arrayvec::ArrayVec;

use crate::board::Board;
use crate::hint::assert_unchecked;
use crate::piece::Piece;
use crate::square::{Col, ColArray, RowArray, Square};

/// ゲーム内の乱数生成器。
///
/// 16bit シフトレジスタだが、外部のカウンタ変数 (`$7F0F52`) の影響を受ける。
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct GameRng(u16);

impl GameRng {
    /// 内部状態を与えて乱数生成器を作る。
    pub const fn new(state: u16) -> Self {
        Self(state)
    }

    /// 内部状態を返す。
    pub const fn state(self) -> u16 {
        self.0
    }

    /// 内部状態を更新し、`0..=0xFF` の乱数を返す。
    pub fn gen(&mut self, counter: u8) -> u8 {
        let bit = ((self.0 >> 14) ^ self.0) & 1;

        self.0 = self.0 ^ ((self.0 << 8) | u16::from(counter));
        self.0 = (self.0 << 1) | bit;

        (self.0 ^ (self.0 >> 8)) as u8
    }

    /// ランダムな駒を生成する。
    pub fn gen_piece(&mut self, counter: u8) -> Piece {
        let piece = 1 + ((5 * u16::from(self.gen(counter))) >> 8) as u8;
        unsafe { Piece::from_inner_unchecked(piece) }
    }

    /// ランダムな盤面を生成する。
    /// ゲーム内の再生成判定に引っ掛かる場合、`None` を返す。
    ///
    /// `inc_counter_after` は、駒を何個生成した後にカウンタをインクリメントするかのパラメータ。
    /// (ゲーム内では盤面生成中に NMI が発生してカウンタがインクリメントされる。
    /// タイミングは CPU サイクルに依存するが、通常は駒が 39 または 40 個生成された直後に起こるようだ)
    pub fn gen_board(&mut self, counter: u8, inc_counter_after: usize) -> Option<Board> {
        unsafe { assert_unchecked!(inc_counter_after <= Square::NUM) }

        // row-major (下から上の順)
        let mut pieces = ArrayVec::<Piece, { Square::NUM }>::new();
        pieces.extend(std::iter::repeat_with(|| self.gen_piece(counter)).take(inc_counter_after));
        pieces.extend(
            std::iter::repeat_with(|| self.gen_piece(counter.wrapping_add(1)))
                .take(Square::NUM - inc_counter_after),
        );

        let arrays = ColArray::from_fn(|col| {
            RowArray::from_fn(|row| pieces[Col::NUM * row.to_index() + col.to_index()])
        });
        Board::from_piece_arrays(&arrays)
    }
}

impl std::fmt::Debug for GameRng {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameRng(0x{:04X}", self.0)
    }
}
