use std::num::{NonZeroU32, NonZeroU64};

use crate::action::ActionHistory;
use crate::board::Board;
use crate::hint::assert_unchecked;
use crate::position::Position;
use crate::score::{score_erase, Score, SCORE_PERFECT};
use crate::util::chmax;

#[derive(Debug)]
pub struct Solver {
    best_score: Score,
    dp: DpTable,
}

impl Solver {
    /// `best_score_ini` より大きいスコアを探索するソルバーを作る。
    pub fn new(best_score_ini: Score) -> Self {
        Self {
            best_score: best_score_ini,
            dp: DpTable::new(),
        }
    }

    /// 現時点での最大スコアを返す。
    pub fn best_score(&self) -> Score {
        self.best_score
    }

    /// 与えられた盤面に対して従来より大きいスコアを探索する。
    /// 見つかった場合、最大スコアの更新も行う。
    pub fn solve(&mut self, board: Board) -> Option<(Score, ActionHistory)> {
        let sub_solver = SubSolver::new(self.best_score, &mut self.dp);
        let res = sub_solver.solve(board);

        eprintln!("DP entry count: {}", self.dp.entry_count());
        self.dp.increment_time();

        let solution;
        (self.best_score, solution) = res?;

        Some((self.best_score, solution))
    }
}

#[derive(Debug)]
struct SubSolver<'solver> {
    best_score: Score,
    best_solution: Option<ActionHistory>,
    history: ActionHistory,
    dp: &'solver mut DpTable,
}

impl<'solver> SubSolver<'solver> {
    fn new(best_score: Score, dp: &'solver mut DpTable) -> Self {
        Self {
            best_score,
            best_solution: None,
            history: ActionHistory::new(),
            dp,
        }
    }

    fn solve(mut self, board: Board) -> Option<(Score, ActionHistory)> {
        let pos = Position::new(board);
        self.dfs(&pos, 0);

        self.best_solution
            .map(|solution| (self.best_score, solution))
    }

    /// 戻り値は `pos` から追加で獲得しうるスコアの上界。
    fn dfs(&mut self, pos: &Position, score: Score) -> Score {
        macro_rules! try_improve {
            ($score:expr) => {{
                if chmax!(self.best_score, $score) {
                    self.best_solution.replace(self.history.clone());
                }
            }};
        }

        // pos がパーフェクトクリアできているなら解の更新を試みて SCORE_PERFECT を返す。
        // (この判定は非常に軽いので最初に行う)
        if pos.board().is_empty() {
            try_improve!(score + SCORE_PERFECT);
            return SCORE_PERFECT;
        }

        let key = pos.key();

        // DP テーブルから pos に対応するエントリを探す。
        let dp_probe = self.dp.probe(key);

        // DP エントリのインデックスとその値 (gain_ub) を得る。
        let (dp_idx, gain_ub) = if let Some(gain_ub) = dp_probe.gain_ub() {
            // DP エントリが既に存在するならその値を返せばよい。
            (dp_probe.into_index(), gain_ub)
        } else {
            // DP エントリがまだ存在しないなら、探索を行わずにわかる範囲で追加スコア上界を見積もる。
            //
            // ここで pos が終了局面ならば解の更新を試みて 0 を返す。
            // 追加スコア上界が 0 ならば pos は終了局面と直ちにわかる。
            // そうでない場合は合法手があるかどうか調べて判定する。
            // (先ほどパーフェクトクリア判定も行ったので、終了局面は決して DP テーブルに載らない)
            //
            // pos が終了局面でないなら、DP テーブルにエントリを新規作成する。
            let gain_ub = pos.score_upper_bound();
            let finished = gain_ub == 0 || !pos.has_action();
            if finished {
                try_improve!(score);
                return 0;
            }
            (dp_probe.make_entry(key, gain_ub), gain_ub)
        };
        // この時点で pos は終了局面でないことが確定する。

        // 現時点での最大スコアを超えられないなら枝刈り。
        if score + gain_ub <= self.best_score {
            return gain_ub;
        }

        // 現時点での最大スコアを超えうるなら、全ての子ノードを探索して pos の追加スコア上界を更新。
        let mut gain_ub_new = 0;
        for action in pos.actions() {
            unsafe { self.history.push_unchecked(action.least_square()) }

            let pos_child = pos.do_action(&action);
            let gain_action = score_erase(action.square_count());
            let gain_ub_child = self.dfs(&pos_child, score + gain_action);
            chmax!(gain_ub_new, gain_action + gain_ub_child);

            unsafe { self.history.remove_last_unchecked() }
        }

        // 新たな追加スコア上界を DP テーブルに記録してから返す。
        self.dp.set_gain_ub(dp_idx, gain_ub_new);
        gain_ub_new
    }
}

const DP_TABLE_CAP_BITS: u32 = 30;
const DP_TABLE_CAP: usize = 1 << DP_TABLE_CAP_BITS;
const _: () = assert!(DP_TABLE_CAP.is_power_of_two());

const KEY_HI_BITS: u32 = 64 - DP_TABLE_CAP_BITS;
const KEY_HI_SHIFT: u32 = 64 - KEY_HI_BITS;
const KEY_HI_MASK: u64 = ((1 << KEY_HI_BITS) - 1) << KEY_HI_SHIFT;

fn calc_key_hi(key: u64) -> u64 {
    key >> KEY_HI_SHIFT
}

/// DP テーブルのエントリ。
///
/// * bit 0-15: 世代 (DP テーブルを毎回再初期化せずに済ませるための機構)。
/// * bit16-28: この局面から追加で獲得しうるスコアの上界。探索を進めるにつれ広義単調減少する。
///             この値が 0 のエントリが作られることはない。
/// * bit29   : (未使用)
/// * bit30-63: この局面のハッシュ値の上位部分。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DpEntry(NonZeroU64);

impl DpEntry {
    const TIME_BITS: u32 = 16;
    const TIME_MASK: u64 = (1 << Self::TIME_BITS) - 1;

    const GAIN_UB_BITS: u32 = 13;
    const GAIN_UB_SHIFT: u32 = 16;
    const GAIN_UB_MASK: u64 = ((1 << Self::GAIN_UB_BITS) - 1) << Self::GAIN_UB_SHIFT;

    fn new(time: u16, key: u64, gain_ub: Score) -> Self {
        unsafe { assert_unchecked!(gain_ub != 0) }

        let value_time = u64::from(time);
        let value_gain_ub = u64::from(gain_ub) << Self::GAIN_UB_SHIFT;
        let value_key = key & KEY_HI_MASK;
        let value = value_time | value_gain_ub | value_key;

        Self(unsafe { NonZeroU64::new_unchecked(value) })
    }

    fn time(self) -> u16 {
        (self.0.get() & Self::TIME_MASK) as u16
    }

    fn gain_ub(self) -> Score {
        ((self.0.get() & Self::GAIN_UB_MASK) >> Self::GAIN_UB_SHIFT) as Score
    }

    fn set_gain_ub(&mut self, gain_ub: Score) {
        unsafe { assert_unchecked!(gain_ub != 0) }

        let value_gain_ub = u64::from(gain_ub) << Self::GAIN_UB_SHIFT;
        let value = (self.0.get() & !Self::GAIN_UB_MASK) | value_gain_ub;

        self.0 = unsafe { NonZeroU64::new_unchecked(value) };
    }

    fn key_hi(self) -> u64 {
        self.0.get() >> KEY_HI_SHIFT
    }
}

/// DP テーブル。
///
/// 世代情報を用いることで、配列を再初期化することなく 0x10000 個の問題を続けて解ける。
///
/// 終了局面は決して DP テーブルに載らない。
#[derive(Debug)]
struct DpTable {
    time: u16,
    entry_count: usize,
    array: Box<[Option<DpEntry>; DP_TABLE_CAP]>,
}

impl DpTable {
    const INDEX_MASK: usize = DP_TABLE_CAP - 1;

    fn new() -> Self {
        Self {
            time: 0,
            entry_count: 0,
            array: vec![None; DP_TABLE_CAP].try_into().unwrap(),
        }
    }

    /// 現在の世代を返す。
    #[allow(dead_code)]
    fn time(&self) -> u16 {
        self.time
    }

    /// 現在の世代におけるエントリ数を返す。
    fn entry_count(&self) -> usize {
        self.entry_count
    }

    /// 世代を更新する。
    ///
    /// 世代がオーバーフローする場合のみテーブル全体が再初期化される。
    fn increment_time(&mut self) {
        let overflow;
        (self.time, overflow) = self.time.overflowing_add(1);

        self.entry_count = 0;

        if overflow {
            self.array.fill(None);
        }
    }

    /// 現在の世代においてハッシュ値 `key` に対応するエントリを探す。
    fn probe(&mut self, key: u64) -> DpTableProbe {
        // linear probing
        let mut idx = key as usize & Self::INDEX_MASK;
        loop {
            let entry = unsafe { *self.array.get_unchecked_mut(idx) };

            match entry {
                None => return DpTableProbe::new_vacant(self, idx),
                Some(entry) if entry.time() != self.time => {
                    return DpTableProbe::new_vacant(self, idx);
                }
                Some(entry) if entry.key_hi() == calc_key_hi(key) => {
                    return DpTableProbe::new_occupied(self, idx, entry.gain_ub());
                }
                _ => idx = (idx + 1) & Self::INDEX_MASK,
            }
        }
    }

    /// `probe()` で仮作成したエントリの値を `gain_ub` に設定する。
    fn set_gain_ub(&mut self, idx: usize, gain_ub: Score) {
        let entry = unsafe { self.array.get_unchecked_mut(idx) };
        let entry = unsafe { entry.as_mut().unwrap_unchecked() };

        entry.set_gain_ub(gain_ub);
    }

    fn make_entry(&mut self, idx: usize, key: u64, gain_ub: Score) {
        self.entry_count += 1;
        /*
        if self.entry_count.is_power_of_two() {
            eprintln!("DP entry count: {}", self.entry_count);
        }
        */

        let entry = unsafe { self.array.get_unchecked_mut(idx) };
        entry.replace(DpEntry::new(self.time, key, gain_ub));
    }
}

#[derive(Debug)]
struct DpTableProbe<'dp> {
    dp: &'dp mut DpTable,
    idx: usize,
    gain_ub: Option<NonZeroU32>,
}

impl<'dp> DpTableProbe<'dp> {
    fn new_occupied(dp: &'dp mut DpTable, idx: usize, gain_ub: Score) -> Self {
        unsafe { assert_unchecked!(gain_ub != 0) }

        Self {
            dp,
            idx,
            gain_ub: Some(unsafe { NonZeroU32::new_unchecked(gain_ub) }),
        }
    }

    fn new_vacant(dp: &'dp mut DpTable, idx: usize) -> Self {
        Self {
            dp,
            idx,
            gain_ub: None,
        }
    }

    /// これが `Some` を返すならエントリは既に存在する。さもなくばエントリはまだ存在しない。
    fn gain_ub(&self) -> Option<Score> {
        self.gain_ub.map(NonZeroU32::get)
    }

    fn into_index(self) -> usize {
        self.idx
    }

    fn make_entry(self, key: u64, gain_ub: Score) -> usize {
        unsafe { assert_unchecked!(self.gain_ub.is_none()) }

        self.dp.make_entry(self.idx, key, gain_ub);

        self.idx
    }
}
