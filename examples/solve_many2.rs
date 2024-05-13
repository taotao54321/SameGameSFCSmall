use clap::Parser;

use samegame_sfc_small::*;

/// 与えられた乱数パラメータ集合内で最大スコアを求める。
#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, default_value_t = 0, value_parser = parse_int::parse::<u16>)]
    state_min: u16,

    #[arg(long, default_value_t = u16::MAX, value_parser = parse_int::parse::<u16>)]
    state_max: u16,

    #[arg(long, default_value_t = 0, value_parser = parse_int::parse::<u8>)]
    counter_min: u8,

    #[arg(long, default_value_t = u8::MAX, value_parser = parse_int::parse::<u8>)]
    counter_max: u8,

    #[arg(long, default_value_t = 39)]
    inc_timing_min: usize,

    #[arg(long, default_value_t = 40)]
    inc_timing_max: usize,

    #[arg(long, default_value_t = 0)]
    best_score_ini: Score,
}

fn make_range<T: PartialOrd>(min: T, max: T) -> std::ops::RangeInclusive<T> {
    assert!(min <= max);
    min..=max
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let states = make_range(cli.state_min, cli.state_max);
    let counters = make_range(cli.counter_min, cli.counter_max);
    let inc_timings = make_range(cli.inc_timing_min, cli.inc_timing_max);

    let mut solver = Solver::new(cli.best_score_ini);
    let mut best_ans: Option<Answer> = None;

    for (state, counter, inc_timing) in itertools::iproduct!(states, counters, inc_timings) {
        let Some(board) = gen_board(state, counter, inc_timing) else {
            eprintln!("Regen: state=0x{state:04X} counter=0x{counter:02X} inc_timing={inc_timing}");
            continue;
        };

        eprintln!("Search: state=0x{state:04X} counter=0x{counter:02X} inc_timing={inc_timing}");

        if let Some((score, solution)) = solver.solve(board) {
            eprintln!("Found {score}: state=0x{state:04X} counter=0x{counter:02X} inc_timing={inc_timing} solution={solution}");
            best_ans.replace(Answer {
                state,
                counter,
                inc_timing,
                score,
                solution,
            });
        }
    }

    if let Some(ans) = best_ans {
        println!(
            "{}\t0x{:04X}\t0x{:02X}\t{}\t{}",
            ans.score, ans.state, ans.counter, ans.inc_timing, ans.solution
        );
    } else {
        eprintln!("NO SOLUTION");
    }

    Ok(())
}

fn gen_board(state: u16, counter: u8, inc_timing: usize) -> Option<Board> {
    GameRng::new(state).gen_board(counter, inc_timing)
}

#[derive(Debug)]
struct Answer {
    state: u16,
    counter: u8,
    inc_timing: usize,
    score: Score,
    solution: ActionHistory,
}
