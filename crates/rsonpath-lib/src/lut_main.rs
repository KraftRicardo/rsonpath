use clap::{Parser, Subcommand};
use distance_distribution::analyse_distance_distribution;
use distance_distribution_per_query::analyse_distance_distribution_per_query;
use rsonpath::lookup_table::analysis::distance_distribution_per_query;
use rsonpath::lookup_table::final_results::{
    deprecated::eval_legacy, deprecated::eval_optimal, eval_final, eval_lut_construction, eval_rq_lut,
    eval_rq_lut_cutoffs, eval_rq_lut_no_lut, eval_serde,
};
use rsonpath::lookup_table::performance::lut_hot::test_hotness;
use rsonpath::lookup_table::performance::lut_query_correctness;
use rsonpath::lookup_table::{
    analysis::{distance_distribution, json_size_estimation_bits::print_estimation},
    performance::{self, lut_skip_evaluation, EVAL_DIR},
    pokemon_test_data_generator,
    query_with_lut::query_with_lut,
    sichash_test_data_generator::{self, SICHASH_DATA_DIR},
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
        /// Path to the folder containing JSON files
        json_dir: String,
        /// Path to the output directory where the results are saved
        out_dir: String,
    },
    Pokemon {
        json_path: String,
    },
    Cutoff {
        data_dir_path: String,
        base_dir_path: String,
    },
    Analysis {
        json_folder_path: String,
    },
    Hot {},
    EvalSerde {
        data_dir_path: String,
        result_dir_path: String,
    },
    EvalRqLegacy {
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
    EvalTOptimal {
        data_dir_path: String,
        result_dir_path: String,
    },
    EvalLutConstruction {
        data_dir_path: String,
        result_dir_path: String,
    },
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
            analyse_distance_distribution(json_dir_path, result_dir_path);
        }
        Commands::AnalyseDistanceDistributionPerQuery {
            json_dir_path,
            result_dir_path,
        } => {
            analyse_distance_distribution_per_query(json_dir_path, result_dir_path);
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
        Commands::Pokemon { json_path } => {
            pokemon_test_data_generator::generate_bigger_version(json_path);
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
        Commands::EvalSerde {
            data_dir_path,
            result_dir_path,
        } => {
            eval_serde::evaluate(data_dir_path, result_dir_path);
        }
        Commands::EvalRqLegacy {
            data_dir_path,
            result_dir_path,
        } => {
            eval_legacy::evaluate(data_dir_path, result_dir_path);
        }
        Commands::EvalRqLut {
            data_dir_path,
            result_dir_path,
        } => {
            eval_rq_lut::evaluate(data_dir_path, result_dir_path);
        }
        Commands::EvalRqLutNoLut {
            data_dir_path,
            result_dir_path,
        } => {
            eval_rq_lut_no_lut::evaluate(data_dir_path, result_dir_path);
        }
        Commands::EvalFinal {
            data_dir_path,
            result_dir_path,
        } => {
            eval_final::evaluate(data_dir_path, result_dir_path);
        }
        Commands::EvalTOptimal {
            data_dir_path,
            result_dir_path,
        } => {
            eval_optimal::evaluate(data_dir_path, result_dir_path);
        }
        Commands::EvalLutConstruction {
            data_dir_path,
            result_dir_path,
        } => {
            eval_lut_construction::evaluate(data_dir_path, result_dir_path);
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
        &format!("{}/performance/{}", dir_name, SICHASH_DATA_DIR),
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
