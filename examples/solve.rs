use std::path::PathBuf;

use anyhow::Context as _;
use clap::Parser;

use samegame_sfc_small::*;

/// SFC『鮫亀』さめがめ「かんたん」モードの問題に対する最大スコア手順を求める。
#[derive(Debug, Parser)]
struct Cli {
    path_problem: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let board = std::fs::read_to_string(&cli.path_problem)
        .with_context(|| format!("問題ファイル {} を読めない", cli.path_problem.display()))?;
    let board: Board = board
        .parse()
        .with_context(|| format!("問題ファイル {} のパースに失敗", cli.path_problem.display()))?;

    let (score, solution) = solve_problem(board);

    println!("{score}\t{solution}");

    Ok(())
}
