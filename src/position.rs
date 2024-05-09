//! 局面関連。

use crate::action::Action;
use crate::board::Board;
use crate::piece::{Piece, PieceArray};
use crate::square::Square;
use crate::zobrist::{Key, ZOBRIST_TABLE};

/// 局面。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Position {
    board: Board,
    key: Key,
    piece_counts: PieceArray<u8>,
}

impl Position {
    /// 初期盤面を指定して局面を作る。
    pub fn new(board: Board) -> Self {
        let key = Square::all()
            .map(|sq| {
                board
                    .get(sq)
                    .map_or(0, |piece| ZOBRIST_TABLE.board(piece, sq))
            })
            .reduce(std::ops::BitXor::bitxor)
            .unwrap();

        let piece_counts = PieceArray::from_fn(|piece| board.piece_count(piece) as u8);

        Self {
            board,
            key,
            piece_counts,
        }
    }

    /// 盤面を返す。
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// ハッシュ値を返す。
    pub fn key(&self) -> Key {
        self.key
    }

    /// 指定した駒種の数を返す。
    pub fn piece_count(&self, piece: Piece) -> u8 {
        self.piece_counts[piece]
    }

    /// 合法手を列挙する。
    pub fn actions(&self) -> impl std::iter::FusedIterator<Item = Action> + Clone + '_ {
        self.board
            .piece_components()
            .filter(|(_piece, mb)| !mb.is_single())
            .map(|(piece, mb)| Action::new(piece, mb))
    }

    /// 着手を行い、結果の局面を返す。
    pub fn do_action(&self, action: &Action) -> Self {
        let board = self.board.erase(action.mask());

        let mut key = self.key;
        for sq in self.board.xor_mask(&board).squares() {
            // 着手前、sq には駒があったとは限らないことに注意(列が詰め直されるケースがあるので)。
            if let Some(piece_before) = self.board.get(sq) {
                key ^= ZOBRIST_TABLE.board(piece_before, sq);
            }
            if let Some(piece_after) = board.get(sq) {
                key ^= ZOBRIST_TABLE.board(piece_after, sq);
            }
        }

        let mut piece_counts = self.piece_counts.clone();
        piece_counts[action.piece()] -= action.square_count() as u8;

        Self {
            board,
            key,
            piece_counts,
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::square::*;

    use super::*;

    fn sq_new(col: Col, row: Row) -> Square {
        Square::new(col, row)
    }

    fn parse_board(s: impl AsRef<str>) -> Board {
        s.as_ref().parse().unwrap()
    }

    fn pos_do_action(pos: &Position, sq: Square) -> Position {
        let action = Action::from_board_square(pos.board(), sq);
        pos.do_action(&action)
    }

    #[test]
    fn test_position() {
        assert_eq!(Position::new(Board::empty()).key(), 0);

        let pos_start = Position::new(parse_board(indoc! {"
            1......2
            155....2
            111.4..2
            12144..1
            12133.51
            12135551
        "}));
        let pos = pos_do_action(&pos_start, sq_new(COL_2, ROW_5));
        let pos = pos_do_action(&pos, sq_new(COL_1, ROW_1));

        let pos_expect = Position::new(parse_board(indoc! {"
            .....2..
            .....2..
            ..4..2..
            244..1..
            233.51..
            235551..
        "}));
        assert_eq!(pos.board(), pos_expect.board());
        assert_eq!(pos.key(), pos_expect.key());
        for piece in Piece::all() {
            assert_eq!(pos.piece_count(piece), pos_expect.piece_count(piece));
        }
    }
}
