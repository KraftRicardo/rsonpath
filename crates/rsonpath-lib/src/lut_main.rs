use clap::{Parser, Subcommand};
use rsonpath::lookup_table::analysis::distance_distribution_per_query;
use rsonpath::lookup_table::extra::query_with_lut::query_with_lut;
use rsonpath::lookup_table::extra::sichash_test_data_generator;
use rsonpath::lookup_table::speed::lut_hot::test_hotness;
use rsonpath::lookup_table::speed::{
    eval_final, eval_lut_construction, eval_rq_lut, eval_rq_lut_cutoffs, eval_rq_lut_no_lut, eval_serde,
    lut_query_correctness, lut_skip_evaluation,
};
use rsonpath::lookup_table::{
    analysis::{distance_distribution, json_size_estimation_bits::print_estimation},
    performance::{self, EVAL_DIR},
};
use std::{error::Error, fs, path::Path};

#[derive(Parser)]
#[command(
    name = "LUT applications",
    about = "List of all commands related to the analysis, testing and performance evaluation of LUT related experiments."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Query {
        json_path: String,
        query: String,
    },
    AnalyseDistanceDistribution {
        json_dir_path: String,
        result_dir_path: String,
    },
    AnalyseDistanceDistributionPerQuery {
        json_dir_path: String,
        result_dir_path: String,
    },
    EvalSerde {
        data_dir_path: String,
        result_dir_path: String,
    },
    EvalRqLut {
        data_dir_path: String,
        result_dir_path: String,
    },
    EvalRqLutNoLut {
        data_dir_path: String,
        result_dir_path: String,
    },
    EvalFinal {
        data_dir_path: String,
        result_dir_path: String,
    },
    EvalLutConstruction {
        data_dir_path: String,
        result_dir_path: String,
    },

    /// Run performance tests
    Performance {
        /// Path to the input JSON folder
        json_dir: String,
        /// Path to the output directory
        out_dir: String,
    },
    Skip {},
    TestQuery {},
    /// Create the test data used in this project: https://github.com/KraftRicardo/test-SicHash
    Sichash {
        json_dir: String,
        out_dir: String,
    },
    Cutoff {
        data_dir_path: String,
        base_dir_path: String,
    },
    Analysis {
        json_folder_path: String,
    },
    Hot {},
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Query { json_path, query } => {
            query_with_lut(json_path, query);
        }
        Commands::AnalyseDistanceDistribution {
            json_dir_path,
            result_dir_path,
        } => {
            distance_distribution::run(json_dir_path, result_dir_path);
        }
        Commands::AnalyseDistanceDistributionPerQuery {
            json_dir_path,
            result_dir_path,
        } => {
            distance_distribution_per_query::run(json_dir_path, result_dir_path);
        }
        Commands::EvalSerde {
            data_dir_path,
            result_dir_path,
        } => {
            eval_serde::run(data_dir_path, result_dir_path);
        }
        Commands::EvalRqLut {
            data_dir_path,
            result_dir_path,
        } => {
            eval_rq_lut::run(data_dir_path, result_dir_path);
        }
        Commands::EvalRqLutNoLut {
            data_dir_path,
            result_dir_path,
        } => {
            eval_rq_lut_no_lut::run(data_dir_path, result_dir_path);
        }
        Commands::EvalFinal {
            data_dir_path,
            result_dir_path,
        } => {
            eval_final::run(data_dir_path, result_dir_path);
        }
        Commands::EvalLutConstruction {
            data_dir_path,
            result_dir_path,
        } => {
            eval_lut_construction::run(data_dir_path, result_dir_path);
        }

        Commands::Analysis { json_folder_path: _ } => {
            // create_json_size_csv(json_folder_path);
            print_estimation();
        }
        Commands::Performance { json_dir, out_dir } => {
            check_if_dir_exists(json_dir);
            create_folder_setup(out_dir)?;
            let csv_dir = format!("{}/{}", out_dir, "performance");

            performance::performance_test(json_dir, &csv_dir);
        }
        Commands::Skip {} => {
            lut_skip_evaluation::skip_evaluation();
        }

        Commands::TestQuery {} => {
            lut_query_correctness::test_build_and_queries();
        }
        Commands::Sichash { json_dir, out_dir } => {
            check_if_dir_exists(json_dir);
            create_folder_setup(out_dir)?;
            let csv_dir = format!("{}/{}", out_dir, "performance");

            sichash_test_data_generator::generate_test_data_for_sichash(json_dir, &csv_dir);
        }
        Commands::Cutoff {
            data_dir_path,
            base_dir_path: result_dir_path,
        } => {
            eval_rq_lut_cutoffs::evaluate(data_dir_path, result_dir_path);
        }
        Commands::Hot {} => {
            test_hotness();
        }
    }

    Ok(())
}

/// Creates the required folder structure if it does not exist.
fn create_folder_setup(dir_name: &str) -> std::io::Result<()> {
    let dirs = [
        dir_name,
        &format!("{}/performance", dir_name),
        &format!("{}/performance/{}", dir_name, EVAL_DIR),
        &format!("{}/test_data", dir_name),
    ];

    for dir in &dirs {
        let path = Path::new(dir);
        if !path.exists() {
            fs::create_dir_all(path)?;
            println!("Created directory: {}", dir);
        }
    }

    Ok(())
}

fn check_if_dir_exists(path: &str) {
    if fs::metadata(path).is_err() {
        panic!("Error: The provided folder '{}' does not exist.", path);
    } else if !Path::new(path).is_dir() {
        panic!("Error: The provided folder '{}' is not a directory.", path);
    }
}
