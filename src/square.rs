//! 盤面のマス関連。

use std::num::NonZeroU8;

use anyhow::{anyhow, ensure, Context as _};

use crate::hint::assert_unchecked;

/// 盤面の列。左から右の順。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Col(NonZeroU8);

pub const COL_1: Col = unsafe { Col::from_inner_unchecked(1) };
pub const COL_2: Col = unsafe { Col::from_inner_unchecked(2) };
pub const COL_3: Col = unsafe { Col::from_inner_unchecked(3) };
pub const COL_4: Col = unsafe { Col::from_inner_unchecked(4) };
pub const COL_5: Col = unsafe { Col::from_inner_unchecked(5) };
pub const COL_6: Col = unsafe { Col::from_inner_unchecked(6) };
pub const COL_7: Col = unsafe { Col::from_inner_unchecked(7) };
pub const COL_8: Col = unsafe { Col::from_inner_unchecked(8) };

impl Col {
    pub const NUM: usize = 8;

    pub const MIN_VALUE: u8 = 1;
    pub const MAX_VALUE: u8 = 8;

    pub const MIN: Self = unsafe { Self::from_inner_unchecked(Self::MIN_VALUE) };
    pub const MAX: Self = unsafe { Self::from_inner_unchecked(Self::MAX_VALUE) };

    /// 内部値から列を作る。
    pub const fn from_inner(inner: u8) -> Option<Self> {
        if Self::is_valid(inner) {
            Some(unsafe { Self::from_inner_unchecked(inner) })
        } else {
            None
        }
    }

    /// 内部値から列を作る。
    ///
    /// # Safety
    ///
    /// `inner` は有効値でなければならない。
    pub const unsafe fn from_inner_unchecked(inner: u8) -> Self {
        assert_unchecked!(Self::is_valid(inner));

        Self(NonZeroU8::new_unchecked(inner))
    }

    const fn is_valid(inner: u8) -> bool {
        matches!(inner, Self::MIN_VALUE..=Self::MAX_VALUE)
    }

    /// 内部値を返す。
    pub const fn to_inner(self) -> u8 {
        self.0.get()
    }

    /// 0-based のインデックスに変換する。
    pub const fn to_index(self) -> usize {
        (self.to_inner() - 1) as usize
    }

    /// 左隣の列を返す。
    pub const fn prev(self) -> Option<Self> {
        if !matches!(self, Self::MIN) {
            Some(unsafe { self.prev_unchecked() })
        } else {
            None
        }
    }

    /// 左隣の列を返す。
    ///
    /// # Safety
    ///
    /// `self != Self::MIN` でなければならない。
    pub const unsafe fn prev_unchecked(self) -> Self {
        assert_unchecked!(!matches!(self, Self::MIN));

        Self::from_inner_unchecked(self.to_inner() - 1)
    }

    /// 右隣の列を返す。
    pub const fn next(self) -> Option<Self> {
        if !matches!(self, Self::MAX) {
            Some(unsafe { self.next_unchecked() })
        } else {
            None
        }
    }

    /// 右隣の列を返す。
    ///
    /// # Safety
    ///
    /// `self != Self::MAX` でなければならない。
    pub const unsafe fn next_unchecked(self) -> Self {
        assert_unchecked!(!matches!(self, Self::MAX));

        Self::from_inner_unchecked(self.to_inner() + 1)
    }

    /// 全ての列を昇順で列挙する。
    pub fn all(
    ) -> impl DoubleEndedIterator<Item = Self> + ExactSizeIterator + std::iter::FusedIterator + Clone
    {
        (Self::MIN_VALUE..=Self::MAX_VALUE)
            .map(|inner| unsafe { Self::from_inner_unchecked(inner) })
    }
}

impl std::str::FromStr for Col {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let col: u8 = s
            .parse()
            .with_context(|| format!("Col のパースに失敗: '{s}'"))?;

        Col::from_inner(col).ok_or_else(|| anyhow!("Col の値が無効: {col}"))
    }
}

impl std::fmt::Display for Col {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// 盤面の行。下から上の順。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Row(NonZeroU8);

pub const ROW_1: Row = unsafe { Row::from_inner_unchecked(1) };
pub const ROW_2: Row = unsafe { Row::from_inner_unchecked(2) };
pub const ROW_3: Row = unsafe { Row::from_inner_unchecked(3) };
pub const ROW_4: Row = unsafe { Row::from_inner_unchecked(4) };
pub const ROW_5: Row = unsafe { Row::from_inner_unchecked(5) };
pub const ROW_6: Row = unsafe { Row::from_inner_unchecked(6) };

impl Row {
    pub const NUM: usize = 6;

    pub const MIN_VALUE: u8 = 1;
    pub const MAX_VALUE: u8 = 6;

    pub const MIN: Self = unsafe { Self::from_inner_unchecked(Self::MIN_VALUE) };
    pub const MAX: Self = unsafe { Self::from_inner_unchecked(Self::MAX_VALUE) };

    /// 内部値から行を作る。
    pub const fn from_inner(inner: u8) -> Option<Self> {
        if Self::is_valid(inner) {
            Some(unsafe { Self::from_inner_unchecked(inner) })
        } else {
            None
        }
    }

    /// 内部値から行を作る。
    ///
    /// # Safety
    ///
    /// `inner` は有効値でなければならない。
    pub const unsafe fn from_inner_unchecked(inner: u8) -> Self {
        assert_unchecked!(Self::is_valid(inner));

        Self(NonZeroU8::new_unchecked(inner))
    }

    const fn is_valid(inner: u8) -> bool {
        matches!(inner, Self::MIN_VALUE..=Self::MAX_VALUE)
    }

    /// 内部値を返す。
    pub const fn to_inner(self) -> u8 {
        self.0.get()
    }

    /// 0-based のインデックスに変換する。
    pub const fn to_index(self) -> usize {
        (self.to_inner() - 1) as usize
    }

    /// 全ての行を昇順で列挙する。
    pub fn all(
    ) -> impl DoubleEndedIterator<Item = Self> + ExactSizeIterator + std::iter::FusedIterator + Clone
    {
        (Self::MIN_VALUE..=Self::MAX_VALUE)
            .map(|inner| unsafe { Self::from_inner_unchecked(inner) })
    }
}

impl std::str::FromStr for Row {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let row: u8 = s
            .parse()
            .with_context(|| format!("Row のパースに失敗: '{s}'"))?;

        Row::from_inner(row).ok_or_else(|| anyhow!("Row の値が無効: {row}"))
    }
}

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// 盤面のマス (column-major)。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Square(NonZeroU8);

impl Square {
    pub const NUM: usize = 48;

    pub const MIN_VALUE: u8 = 1;
    pub const MAX_VALUE: u8 = 48;

    pub const MIN: Self = unsafe { Self::from_inner_unchecked(Self::MIN_VALUE) };
    pub const MAX: Self = unsafe { Self::from_inner_unchecked(Self::MAX_VALUE) };

    /// 内部値からマスを作る。
    pub const fn from_inner(inner: u8) -> Option<Self> {
        if Self::is_valid(inner) {
            Some(unsafe { Self::from_inner_unchecked(inner) })
        } else {
            None
        }
    }

    /// 内部値からマスを作る。
    ///
    /// # Safety
    ///
    /// `inner` は有効値でなければならない。
    pub const unsafe fn from_inner_unchecked(inner: u8) -> Self {
        assert_unchecked!(Self::is_valid(inner));

        Self(NonZeroU8::new_unchecked(inner))
    }

    const fn is_valid(inner: u8) -> bool {
        matches!(inner, Self::MIN_VALUE..=Self::MAX_VALUE)
    }

    /// 内部値を返す。
    pub const fn to_inner(self) -> u8 {
        self.0.get()
    }

    /// 0-based のインデックスに変換する。
    pub const fn to_index(self) -> usize {
        (self.to_inner() - 1) as usize
    }

    /// 列と行からマスを作る。
    pub const fn new(col: Col, row: Row) -> Self {
        let sq = Row::NUM as u8 * (col.to_inner() - 1) + row.to_inner();

        unsafe { Self::from_inner_unchecked(sq) }
    }

    /// マスが属する列を返す。
    pub const fn col(self) -> Col {
        let col = 1 + (self.to_inner() - 1) / Row::NUM as u8;

        unsafe { Col::from_inner_unchecked(col) }
    }

    /// マスが属する行を返す。
    pub const fn row(self) -> Row {
        let row = 1 + (self.to_inner() - 1) % Row::NUM as u8;

        unsafe { Row::from_inner_unchecked(row) }
    }

    /// 全てのマスを昇順で列挙する。
    pub fn all(
    ) -> impl DoubleEndedIterator<Item = Self> + ExactSizeIterator + std::iter::FusedIterator + Clone
    {
        (Self::MIN_VALUE..=Self::MAX_VALUE)
            .map(|inner| unsafe { Self::from_inner_unchecked(inner) })
    }
}

impl std::str::FromStr for Square {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fields: Vec<_> = s.split(',').collect();
        ensure!(fields.len() == 2, "Square のパースに失敗: '{s}'");

        let col: Col = fields[0]
            .parse()
            .with_context(|| format!("Square の列のパースに失敗: {}", fields[0]))?;
        let row: Row = fields[1]
            .parse()
            .with_context(|| format!("Square の行のパースに失敗: {}", fields[1]))?;

        Ok(Self::new(col, row))
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.col(), self.row())
    }
}

/// `Col` でインデックスアクセスできる配列。
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColArray<T>([T; Col::NUM]);

impl<T: Default> Default for ColArray<T> {
    fn default() -> Self {
        Self::from_fn(|_| T::default())
    }
}

impl<T: Clone> ColArray<T> {
    pub fn from_elem(elem: T) -> Self {
        Self::from_fn(|_| elem.clone())
    }
}

impl<T> ColArray<T> {
    pub const fn new(inner: [T; Col::NUM]) -> Self {
        Self(inner)
    }

    pub fn from_fn(mut f: impl FnMut(Col) -> T) -> Self {
        Self::new(std::array::from_fn(|i| {
            f(unsafe { Col::from_inner_unchecked((i + 1) as u8) })
        }))
    }

    pub const fn as_array(&self) -> &[T; Col::NUM] {
        &self.0
    }

    pub fn enumerate(
        &self,
    ) -> impl DoubleEndedIterator<Item = (Col, &T)> + ExactSizeIterator + std::iter::FusedIterator + Clone
    {
        Col::all().map(|col| (col, &self[col]))
    }
}

impl<T> std::ops::Index<Col> for ColArray<T> {
    type Output = T;

    fn index(&self, col: Col) -> &Self::Output {
        unsafe { self.0.get_unchecked(col.to_index()) }
    }
}

impl<T> std::ops::IndexMut<Col> for ColArray<T> {
    fn index_mut(&mut self, col: Col) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(col.to_index()) }
    }
}

/// `Row` でインデックスアクセスできる配列。
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RowArray<T>([T; Row::NUM]);

impl<T: Default> Default for RowArray<T> {
    fn default() -> Self {
        Self::from_fn(|_| T::default())
    }
}

impl<T: Clone> RowArray<T> {
    pub fn from_elem(elem: T) -> Self {
        Self::from_fn(|_| elem.clone())
    }
}

impl<T> RowArray<T> {
    pub const fn new(inner: [T; Row::NUM]) -> Self {
        Self(inner)
    }

    pub fn from_fn(mut f: impl FnMut(Row) -> T) -> Self {
        Self::new(std::array::from_fn(|i| {
            f(unsafe { Row::from_inner_unchecked((i + 1) as u8) })
        }))
    }

    pub const fn as_array(&self) -> &[T; Row::NUM] {
        &self.0
    }

    pub fn enumerate(
        &self,
    ) -> impl DoubleEndedIterator<Item = (Row, &T)> + ExactSizeIterator + std::iter::FusedIterator + Clone
    {
        Row::all().map(|row| (row, &self[row]))
    }
}

impl<T> std::ops::Index<Row> for RowArray<T> {
    type Output = T;

    fn index(&self, row: Row) -> &Self::Output {
        unsafe { self.0.get_unchecked(row.to_index()) }
    }
}

impl<T> std::ops::IndexMut<Row> for RowArray<T> {
    fn index_mut(&mut self, row: Row) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(row.to_index()) }
    }
}

/// `Square` でインデックスアクセスできる配列。
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SquareArray<T>([T; Square::NUM]);

impl<T: Default> Default for SquareArray<T> {
    fn default() -> Self {
        Self::from_fn(|_| T::default())
    }
}

impl<T: Clone> SquareArray<T> {
    pub fn from_elem(elem: T) -> Self {
        Self::from_fn(|_| elem.clone())
    }
}

impl<T> SquareArray<T> {
    pub const fn new(inner: [T; Square::NUM]) -> Self {
        Self(inner)
    }

    pub fn from_fn(mut f: impl FnMut(Square) -> T) -> Self {
        Self::new(std::array::from_fn(|i| {
            f(unsafe { Square::from_inner_unchecked((i + 1) as u8) })
        }))
    }

    pub const fn as_array(&self) -> &[T; Square::NUM] {
        &self.0
    }

    pub fn enumerate(
        &self,
    ) -> impl DoubleEndedIterator<Item = (Square, &T)>
           + ExactSizeIterator
           + std::iter::FusedIterator
           + Clone {
        Square::all().map(|sq| (sq, &self[sq]))
    }
}

impl<T> std::ops::Index<Square> for SquareArray<T> {
    type Output = T;

    fn index(&self, sq: Square) -> &Self::Output {
        unsafe { self.0.get_unchecked(sq.to_index()) }
    }
}

impl<T> std::ops::IndexMut<Square> for SquareArray<T> {
    fn index_mut(&mut self, sq: Square) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(sq.to_index()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_sq(s: impl AsRef<str>) -> Square {
        s.as_ref().parse().unwrap()
    }

    #[test]
    fn test_square_new() {
        for (col, row) in itertools::iproduct!(Col::all(), Row::all()) {
            let sq = Square::new(col, row);
            assert_eq!(sq.col(), col);
            assert_eq!(sq.row(), row);
        }
    }

    #[test]
    fn test_square_io() {
        for sq in Square::all() {
            let s = sq.to_string();
            assert_eq!(parse_sq(s), sq);
        }
    }
}
