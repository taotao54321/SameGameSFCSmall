use clap::Parser;

use samegame_sfc_small::*;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(value_parser = parse_int::parse::<u16>)]
    state: u16,

    #[arg(value_parser = parse_int::parse::<u8>)]
    counter: u8,

    #[arg(value_parser = parse_int::parse::<usize>)]
    inc_timing: usize,
}

fn main() {
    let cli = Cli::parse();

    let board = GameRng::new(cli.state)
        .gen_board(cli.counter, cli.inc_timing)
        .expect("再生成判定に引っ掛かる");

    print!("{board}");
}
