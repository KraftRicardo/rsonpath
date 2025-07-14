use clap::{Parser, Subcommand};
use rsonpath::evaluation::{eval_legacy, eval_optimal};
use std::error::Error;

#[derive(Parser)]
#[command(
    name = "Measure skip time of rsonpath",
    about = "Code to measure the skip time of rsonpath-original so I can compare it with rsonpath-lut."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    EvalLegacy {
        data_dir_path: String,
        result_dir_path: String,
    },
    EvalOptimal {
        data_dir_path: String,
        result_dir_path: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::EvalLegacy {
            data_dir_path,
            result_dir_path,
        } => {
            eval_legacy::run(data_dir_path, result_dir_path);
        }
        Commands::EvalOptimal {
            data_dir_path,
            result_dir_path,
        } => {
            eval_optimal::run(data_dir_path, result_dir_path);
        }
    }

    Ok(())
}
