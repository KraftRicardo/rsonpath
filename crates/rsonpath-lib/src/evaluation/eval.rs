use clap::{Parser, Subcommand};
use rsonpath::evaluation::{eval_legacy, eval_legacy_skip_time};
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
    EvalLegacyEmptyListOptOff {
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
            eval_legacy::evaluate_rq_query_speed(data_dir_path, result_dir_path, true);
        }
        Commands::EvalLegacyEmptyListOptOff {
            data_dir_path,
            result_dir_path,
        } => {
            eval_legacy::evaluate_rq_query_speed(data_dir_path, result_dir_path, false);
        }
        Commands::EvalOptimal {
            data_dir_path,
            result_dir_path,
        } => {
            eval_legacy_skip_time::run(data_dir_path, result_dir_path);
        }
    }

    Ok(())
}
