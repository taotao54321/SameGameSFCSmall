use std::num::NonZeroU64;

use crate::action::ActionHistory;
use crate::board::Board;
use crate::position::Position;
use crate::score::{score_erase, Score, SCORE_PERFECT};
use crate::util::chmax;

/// 与えられた盤面に対する最大スコアとその手順を返す。
pub fn solve_problem(board: Board) -> (Score, ActionHistory) {
    // 初期盤面が空の場合について考えたくないので、先に処理してしまう。
    if board.is_empty() {
        return (SCORE_PERFECT, ActionHistory::new());
    }

    let pos = Position::new(board);

    Solver::new().solve(&pos)
}

#[derive(Debug)]
struct Solver {
    dp: HashTable,
}

impl Solver {
    fn new() -> Self {
        Self {
            dp: HashTable::new(),
        }
    }

    fn solve(mut self, pos_root: &Position) -> (Score, ActionHistory) {
        let score = self.dfs(pos_root);
        eprintln!("TT entry count: {}", self.dp.entry_count());

        // 経路復元。
        let mut solution = ActionHistory::new();
        let mut pos = pos_root.clone();
        loop {
            let best_action = pos.actions().max_by_key(|action| {
                let pos_child = pos.do_action(action);
                let gain_action = score_erase(action.square_count());
                // 空の盤面は DP テーブルに載らないので例外処理が必要。
                // それ以外の盤面は DP テーブルに載っているはず。
                let gain_child = if pos_child.board().is_empty() {
                    SCORE_PERFECT
                } else {
                    let HashTableProbe::Found(score) = self.dp.probe(pos_child.key()) else {
                        eprintln!("この盤面の DP エントリが見つからない!?");
                        eprint!("{}", pos_child.board());
                        unreachable!();
                    };
                    score
                };
                gain_action + gain_child
            });
            let Some(best_action) = best_action else {
                break;
            };
            solution.push(best_action.least_square());
            pos = pos.do_action(&best_action);
        }

        (score, solution)
    }

    /// `pos` から追加で獲得できる最大スコアを返す。
    fn dfs(&mut self, pos: &Position) -> Score {
        // 空の盤面に対する DP エントリが作られないよう、先にパーフェクト判定する。
        // 他の終了局面については仮作成するエントリの gain_max が 0 なのでそのままでよい。
        if pos.board().is_empty() {
            return SCORE_PERFECT;
        }

        let key = pos.key();

        match self.dp.probe(key) {
            HashTableProbe::Found(gain_max) => gain_max,
            HashTableProbe::Created(dp_idx) => {
                let mut gain_max = 0;
                for action in pos.actions() {
                    let pos_child = pos.do_action(&action);
                    let gain_action = score_erase(action.square_count());
                    let gain_child = self.dfs(&pos_child);
                    chmax!(gain_max, gain_action + gain_child);
                }

                // 終了局面ならば単に 0 を返す。
                // DP テーブルに仮作成したエントリの gain_max は 0 なのでそのままでよい。
                // 終了局面であることと gain_max が 0 であることは同値。
                if gain_max == 0 {
                    return 0;
                }

                self.dp.set_gain_max(dp_idx, gain_max);
                gain_max
            }
        }
    }
}

const HASH_TABLE_CAP_BITS: u32 = 30;
const HASH_TABLE_CAP: usize = 1 << HASH_TABLE_CAP_BITS;
const _: () = assert!(HASH_TABLE_CAP.is_power_of_two());

const KEY_HI_BITS: u32 = 64 - HASH_TABLE_CAP_BITS;
const KEY_HI_SHIFT: u32 = 64 - KEY_HI_BITS;

fn calc_key_hi(key: u64) -> u64 {
    key >> KEY_HI_SHIFT
}

/// メモ化再帰のためのハッシュテーブルのエントリ。
///
/// * 下位 12 bit: 1 + (この局面から追加で獲得できる最大スコア)。
/// * 上位 (64 - HASH_TABLE_CAP_BITS) bit: この局面のハッシュ値の上位部分。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HashTableEntry(NonZeroU64);

const _: () = assert!(std::mem::size_of::<Option<HashTableEntry>>() == 8);

impl HashTableEntry {
    const GAIN_MAX_BITS: u32 = 12;
    const GAIN_MAX_MASK: u64 = (1 << Self::GAIN_MAX_BITS) - 1;

    const KEY_HI_MASK: u64 = ((1 << KEY_HI_BITS) - 1) << KEY_HI_SHIFT;

    fn new(key: u64, gain_max: Score) -> Self {
        let value_gain_max = u64::from(1 + gain_max);
        let value_key = key & Self::KEY_HI_MASK;
        let value = value_gain_max | value_key;

        Self(unsafe { NonZeroU64::new_unchecked(value) })
    }

    fn gain_max(self) -> Score {
        ((self.0.get() & Self::GAIN_MAX_MASK) - 1) as Score
    }

    fn set_gain_max(&mut self, gain_max: Score) {
        let value_gain_max = u64::from(1 + gain_max);
        let value = (self.0.get() & !Self::GAIN_MAX_MASK) | value_gain_max;

        self.0 = unsafe { NonZeroU64::new_unchecked(value) };
    }

    fn key_hi(self) -> u64 {
        self.0.get() >> KEY_HI_SHIFT
    }
}

/// メモ化再帰のためのハッシュテーブル。
///
/// インデックス衝突については linear probing で対処する。
/// ハッシュ値自体の衝突については特に対策していない。
#[derive(Debug)]
struct HashTable {
    entry_count: usize,
    array: Box<[Option<HashTableEntry>; HASH_TABLE_CAP]>,
}

impl HashTable {
    const INDEX_MASK: usize = HASH_TABLE_CAP - 1;

    fn new() -> Self {
        Self {
            entry_count: 0,
            array: vec![None; HASH_TABLE_CAP].try_into().unwrap(),
        }
    }

    fn entry_count(&self) -> usize {
        self.entry_count
    }

    /// ハッシュ値 `key` に対応するエントリを探し、結果を返す。
    ///
    /// エントリが既に存在する場合、その値 (gain_max) を返す。
    /// エントリ自体がまだ存在しない場合、仮の値でエントリを作成し、そのインデックスを返す。
    fn probe(&mut self, key: u64) -> HashTableProbe {
        // linear probe

        // key に対応する局面が終了局面の場合、盤面が空でないなら仮作成したエントリはそのままにできる。
        // (gain_max を 0 として仮作成するので)
        // しかし、盤面が空の場合は仮作成してしまうとエントリの値が正しくなくなる。
        //
        // これを安直に解決するならハッシュ値 0 に対して SCORE_PERFECT を返すようにすればよいが、
        // 偶然ハッシュ値が 0 の空でない盤面が生じてしまうとほぼ確実に解がおかしくなる。
        //
        // というわけで、一応 Solver 側で空の盤面に対する例外処理を行い、
        // 空の盤面は DP テーブルに載らないようにしておく。

        let mut idx = key as usize & Self::INDEX_MASK;
        loop {
            let entry = unsafe { self.array.get_unchecked_mut(idx) };
            match entry {
                None => {
                    self.entry_count += 1;
                    if self.entry_count.is_power_of_two() {
                        eprintln!("TT entry count: {}", self.entry_count);
                    }
                    entry.replace(HashTableEntry::new(key, 0));
                    return HashTableProbe::Created(idx);
                }
                Some(entry) if entry.key_hi() == calc_key_hi(key) => {
                    return HashTableProbe::Found(entry.gain_max());
                }
                _ => idx = idx.wrapping_add(1) & Self::INDEX_MASK,
            }
        }
    }

    /// `probe()` で仮作成したエントリの値を `gain_max` に設定する。
    fn set_gain_max(&mut self, idx: usize, gain_max: Score) {
        let entry = unsafe { self.array.get_unchecked_mut(idx) };
        let entry = unsafe { entry.as_mut().unwrap_unchecked() };

        entry.set_gain_max(gain_max);
    }
}

#[derive(Debug)]
enum HashTableProbe {
    Found(Score),
    Created(usize),
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

    fn solution_new(sqs: impl IntoIterator<Item = Square>) -> ActionHistory {
        sqs.into_iter().collect()
    }

    #[test]
    #[ignore]
    fn test_solve_problem() {
        assert_eq!(
            solve_problem(Board::empty()),
            (SCORE_PERFECT, solution_new([]))
        );

        {
            let board = parse_board(indoc! {"
                12345123
                51234512
                45123451
                34512345
                23451234
                12345123
            "});
            assert_eq!(solve_problem(board), (0, solution_new([])));
        }
        {
            let board = parse_board(indoc! {"
                ........
                ........
                ........
                ........
                ........
                111.....
            "});
            assert_eq!(
                solve_problem(board),
                (
                    score_erase(3) + SCORE_PERFECT,
                    solution_new([sq_new(COL_1, ROW_1)])
                )
            );
        }
    }
}
