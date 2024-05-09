//! 駒関連。

use std::num::NonZeroU8;

use crate::hint::assert_unchecked;

/// 駒種。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Piece(NonZeroU8);

impl Piece {
    pub const NUM: usize = 5;

    pub const MIN_VALUE: u8 = 1;
    pub const MAX_VALUE: u8 = 5;

    /// 内部値から駒種を作る。
    pub const fn from_inner(inner: u8) -> Option<Self> {
        if Self::is_valid(inner) {
            Some(unsafe { Self::from_inner_unchecked(inner) })
        } else {
            None
        }
    }

    /// 内部値から駒種を作る。
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

    /// 全ての駒種を昇順で列挙する。
    pub fn all(
    ) -> impl DoubleEndedIterator<Item = Self> + ExactSizeIterator + std::iter::FusedIterator + Clone
    {
        (Self::MIN_VALUE..=Self::MAX_VALUE)
            .map(|inner| unsafe { Self::from_inner_unchecked(inner) })
    }
}

/// `Piece` でインデックスアクセスできる配列。
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PieceArray<T>([T; Piece::NUM]);

impl<T: Default> Default for PieceArray<T> {
    fn default() -> Self {
        Self::from_fn(|_| T::default())
    }
}

impl<T: Clone> PieceArray<T> {
    pub fn from_elem(elem: T) -> Self {
        Self::from_fn(|_| elem.clone())
    }
}

impl<T> PieceArray<T> {
    pub const fn new(inner: [T; Piece::NUM]) -> Self {
        Self(inner)
    }

    pub fn from_fn(mut f: impl FnMut(Piece) -> T) -> Self {
        Self::new(std::array::from_fn(|i| {
            f(unsafe { Piece::from_inner_unchecked((i + 1) as u8) })
        }))
    }

    pub const fn as_array(&self) -> &[T; Piece::NUM] {
        &self.0
    }

    pub fn enumerate(
        &self,
    ) -> impl DoubleEndedIterator<Item = (Piece, &T)>
           + ExactSizeIterator
           + std::iter::FusedIterator
           + Clone {
        Piece::all().map(|piece| (piece, &self[piece]))
    }
}

impl<T> std::ops::Index<Piece> for PieceArray<T> {
    type Output = T;

    fn index(&self, piece: Piece) -> &Self::Output {
        unsafe { self.0.get_unchecked(piece.to_index()) }
    }
}

impl<T> std::ops::IndexMut<Piece> for PieceArray<T> {
    fn index_mut(&mut self, piece: Piece) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(piece.to_index()) }
    }
}
