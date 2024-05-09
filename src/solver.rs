use std::num::NonZeroU64;

use crate::action::ActionHistory;
use crate::board::Board;
use crate::position::Position;
use crate::score::{score_erase, Score, SCORE_PERFECT};
use crate::util::chmax;

/// 与えられた盤面に対する最大スコアとその手順を返す。
pub fn solve_problem(board: Board) -> (Score, ActionHistory) {
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
                let gain_child = match self.dp.probe(pos_child.key()) {
                    HashTableProbe::Found(score) => score,
                    HashTableProbe::Created(_) => {
                        unreachable!("あるはずの DP エントリが見つからない!?")
                    }
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

                // 終了局面ならばパーフェクト判定して値を返す。
                // 終了局面であることと gain_max が 0 であることは同値。
                if gain_max == 0 {
                    return if pos.board().is_empty() {
                        SCORE_PERFECT
                    } else {
                        0
                    };
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
