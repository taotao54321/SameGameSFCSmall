//! 着手関連。

use anyhow::ensure;
use arrayvec::ArrayVec;

use crate::board::{Board, MaskBoard};
use crate::hint::*;
use crate::piece::Piece;
use crate::square::{Col, Row, Square};

/// 着手。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Action {
    piece: Piece,
    mb: MaskBoard,
}

impl Action {
    /// 駒種と盤面マスクを指定して着手を作る。
    ///
    /// `mb` は 2 つ以上のマスを含んでいなければならない。
    pub fn new(piece: Piece, mb: MaskBoard) -> Self {
        unsafe { assert_unchecked!(mb.square_count() >= 2) }

        Self { piece, mb }
    }

    /// 盤面とマスを指定して着手を作る。
    ///
    /// `board` のマス `sq` には駒があり、かつ同種の駒が繋がっていなければならない。
    pub fn from_board_square(board: &Board, sq: Square) -> Self {
        let piece = board.get(sq).expect("盤面のマス {sq} に駒がない");

        let mb = board.piece_mask(piece).flood_fill(sq);
        assert!(
            mb.square_count() >= 2,
            "盤面のマス {sq} から同種の駒が 2 個以上繋がっていなければならない"
        );

        Self::new(piece, mb)
    }

    /// 駒種を返す。
    pub fn piece(&self) -> Piece {
        self.piece
    }

    /// 盤面マスクを返す。
    pub fn mask(&self) -> &MaskBoard {
        &self.mb
    }

    /// この着手により消える駒数を返す。
    pub fn square_count(&self) -> u32 {
        self.mb.square_count()
    }

    /// この着手により消える駒を含む最小のマスを返す。
    pub fn least_square(&self) -> Square {
        unsafe { self.mb.least_square_unchecked() }
    }
}

const HISTORY_CAP: usize = Col::NUM * Row::NUM / 2;

/// 着手履歴。
#[repr(transparent)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActionHistory(ArrayVec<Square, HISTORY_CAP>);

impl ActionHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_slice(&self) -> &[Square] {
        self.0.as_slice()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, sq: Square) {
        self.0.push(sq);
    }

    /// # Safety
    ///
    /// 容量オーバーしてはならない。
    pub unsafe fn push_unchecked(&mut self, sq: Square) {
        self.0.push_unchecked(sq);
    }

    pub fn remove_last(&mut self) {
        self.0.pop();
    }

    /// # Safety
    ///
    /// `self` は空であってはならない。
    pub unsafe fn remove_last_unchecked(&mut self) {
        self.0.set_len(self.len() - 1)
    }

    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl std::iter::FromIterator<Square> for ActionHistory {
    fn from_iter<I: IntoIterator<Item = Square>>(sqs: I) -> Self {
        Self(ArrayVec::from_iter(sqs))
    }
}

impl std::iter::IntoIterator for ActionHistory {
    type Item = Square;
    type IntoIter = <ArrayVec<Square, HISTORY_CAP> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a ActionHistory {
    type Item = &'a Square;
    type IntoIter = std::slice::Iter<'a, Square>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl std::str::FromStr for ActionHistory {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<_> = s.split_ascii_whitespace().collect();
        ensure!(
            tokens.len() <= HISTORY_CAP,
            "着手履歴は {HISTORY_CAP} 手以下でなければならない"
        );

        tokens.into_iter().map(str::parse).collect()
    }
}

impl std::fmt::Display for ActionHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, &sq) in self.iter().enumerate() {
            if i != 0 {
                f.write_str(" ")?;
            }
            sq.fmt(f)?;
        }

        Ok(())
    }
}
