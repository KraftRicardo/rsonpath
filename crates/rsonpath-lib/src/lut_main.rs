use clap::{Parser, Subcommand};
use distance_distribution_per_json::analyse_distance_distribution_per_json;
use distance_distribution_per_query::analyse_distance_distribution_per_query;
use rsonpath::lookup_table::analysis::distance_distribution_per_query;
use rsonpath::lookup_table::analysis::{distance_distribution_per_json, json_size_estimation_bits::print_estimation};
use rsonpath::lookup_table::correctness::{lut_build_correctness, lut_query_correctness};
use rsonpath::lookup_table::extra::{eval_valgrind, lut_test_hotness, query_with_lut};
use rsonpath::lookup_table::speed::{
    eval_distance_cutoff, eval_final, eval_lut_construction, eval_rq_lut, eval_rq_lut_no_lut, eval_serde,
};
use std::error::Error;

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
    // ##############
    // ## Analysis ##
    // ##############
    AnalyseDistanceDistributionPerJson {
        json_dir_path: String,
        result_dir_path: String,
    },
    AnalyseDistanceDistributionPerQuery {
        json_dir_path: String,
        result_dir_path: String,
        cutoff: usize,
    },
    EstimateIndexForJsonSize {},
    // ##############
    // # Evaluation #
    // ##############
    EvalDistanceCutoff {
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
    EvalRqLut {
        data_dir_path: String,
        result_dir_path: String,
    },
    EvalRqLutNoLut {
        data_dir_path: String,
        result_dir_path: String,
    },
    EvalSerde {
        data_dir_path: String,
        result_dir_path: String,
    },
    EvalValgrind {},
    // ###############
    // # Correctness #
    // ###############
    TestQueryCorrectness {
        data_dir_path: String,
    },
    TestBuildCorrectness {
        data_dir_path: String,
    },
    // ##############
    // ### Extra ####
    // ##############
    QueryWithLut {
        json_path: String,
        query: String,
    },
    TestHotness {},
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        // ##############
        // ## Analysis ##
        // ##############
        Commands::AnalyseDistanceDistributionPerJson {
            json_dir_path,
            result_dir_path,
        } => {
            analyse_distance_distribution_per_json(json_dir_path, result_dir_path);
        }
        Commands::AnalyseDistanceDistributionPerQuery {
            json_dir_path,
            result_dir_path,
            cutoff,
        } => {
            analyse_distance_distribution_per_query(json_dir_path, result_dir_path, *cutoff);
        }
        Commands::EstimateIndexForJsonSize {} => {
            print_estimation();
        }
        // ##############
        // # Evaluation #
        // ##############
        Commands::EvalDistanceCutoff {
            data_dir_path,
            result_dir_path,
        } => {
            eval_distance_cutoff::run(data_dir_path, result_dir_path);
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
        Commands::EvalSerde {
            data_dir_path,
            result_dir_path,
        } => {
            eval_serde::run(data_dir_path, result_dir_path);
        }
        // ###############
        // # Correctness #
        // ###############
        Commands::TestQueryCorrectness { data_dir_path } => {
            lut_query_correctness::run(data_dir_path);
        }
        Commands::TestBuildCorrectness { data_dir_path } => {
            lut_build_correctness::run(data_dir_path);
        }
        // ##############
        // ### Extra ####
        // ##############
        Commands::EvalValgrind {} => {
            eval_valgrind::run();
        }
        Commands::TestHotness {} => {
            lut_test_hotness::run();
        }
        Commands::QueryWithLut { json_path, query } => {
            query_with_lut::run(json_path, query);
        }
    }

    Ok(())
}
