//! スコア関連。

use crate::hint::assert_unchecked;

/// スコアを表す型。
///
/// 実際には `0..=2409` の値をとる (最大値は 48 個全消し時)。
/// つまり 12bit に収まる。
pub type Score = u32;

pub const SCORE_PERFECT: Score = 200;

/// n 個の駒を消す着手による獲得スコアを返す。
///
/// `n >= 2` でなければならない。
pub const fn score_erase(n: u32) -> Score {
    unsafe { assert_unchecked!(n >= 2) }

    (n - 1).pow(2)
}
