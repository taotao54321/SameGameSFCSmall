use clap::Parser;

use samegame_sfc_small::*;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, default_value_t = 39, value_parser = parse_int::parse::<usize>)]
    inc_timing_min: usize,

    #[arg(long, default_value_t = 40, value_parser = parse_int::parse::<usize>)]
    inc_timing_max: usize,

    #[arg(long, default_value_t = 0, value_parser = parse_int::parse::<u8>)]
    counter_min: u8,

    #[arg(long, default_value_t = u8::MAX, value_parser = parse_int::parse::<u8>)]
    counter_max: u8,

    #[arg(long, default_value_t = 0, value_parser = parse_int::parse::<Score>)]
    best_score_ini: Score,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let states = 0..=u16::MAX;
    let counters = cli.counter_min..=cli.counter_max;
    let inc_timings = cli.inc_timing_min..=cli.inc_timing_max;

    let ans = solve_problems_many(states, counters, inc_timings, cli.best_score_ini);

    println!(
        "{}\t0x{:04X}\t0x{:02X}\t{}\t{}",
        ans.score(),
        ans.rng_state(),
        ans.rng_counter(),
        ans.rng_inc_timing(),
        ans.solution()
    );

    Ok(())
}
