# How to use the crate

    cargo install mdbook
    cargo install rsonpath
    cargo install --list

## Run the crate for example

    rq '$.name' ./.a_lut_tests/test_data/kB_1/john_119.json
    rq '$.cfgs.*.ID' ./ricardo/test_data/memory_test/mini/pokemonCfgs.json
    rq -count "$.cfgs[1].Name" .a_lut_tests/test_data/MB_15/pokemon_\(6MB\).json 

## Set Up

    cargo install just

## How to Build & Run

    just build
    just init

## Reset

    git submodule deinit --all -f
    just init

### Cheatsheet

- `just b` &ndash; build the binary in debug mode;
- `just r *ARGS` &ndash; run the debug binary with given arguments, e.g. `just r '$.a.b' -v`
- `just v` &ndash; verify that the lib and bin compile (with `cargo check`);
- `just t` &ndash; run the fast unit tests of the library.

### Tests

- `just gen-tests` generates the test in the folders called "generated"
- `just test-engine` runs the end-to-end tests (for correctness)
- `just test` to execute all tests in the project. This includes real dataset end-to-end tests,
  so might take a minute or so.
- `just t` command runs only unit tests, which is very quick,
- `just doctest` runs doctests.

## Install conda environment for plotting the lut statistics

    conda env create -f environment.yml

## Run lut code for example with

    cargo run --bin lut --release -- distances .a_lut_tests/test_data/GB_25 .a_lut_tests
    cargo run --bin lut --release -- distances .a_lut_tests/test_data/MB_1 .a_lut_tests/analysis/distance_distribution_short/
    cargo run --bin lut --release -- sichash .a_lut_tests/test_data/MB_100 .a_lut_tests
    cargo run --bin lut --release -- query $.person.spouse.person.phoneNumber[*] .a_lut_tests/test_data/kB_1/john_big.json
    cargo run --bin lut --release -- performance .a_lut_tests/test_data/MB_1 .a_lut_tests
    cargo run --bin lut --release -- skip
    cargo run --bin lut --release -- skip-count
    cargo run --bin lut --release -- test-query
    cargo run --bin lut --release -- cutoff
    cargo run --bin lut --release -- analysis .a_lut_tests/test_data/GB_1

### If you want flags

    cargo run --bin lut -F empty-list-opt  --release -- skip-count

## Gen and run tests

    just gen-tests
    just test-engine

## Run the tests with

    Define the environment variables first. This forces nosimd as used classifier:
        // no SIMD
        export RSONPATH_UNSAFE_FORCE_SIMD="nosimd;slow_quotes;slow_popcnt"
        // SIMD
        export RSONPATH_UNSAFE_FORCE_SIMD="avx2;fast_quotes;fast_popcnt"
        // remove it
        unset RSONPATH_UNSAFE_FORCE_SIMD

    Run with:
        cargo test --test lut_build_tests
        cargo test --test lut_query_tests

    Examples:
        // now works with no-simd and simd
        cargo test -p rsonpath-test --tests -q -- --test-threads=10
        // long
        just test-engine
        // even longer
        just test-full
        just verify
        cargo test -p rsonpath-test generated::atomic_after_list
        
        cargo test --test lut_query_tests -- query_john_big_log --nocapture | rg "(tail_skipping|lut_query_tests)"
        cargo test --test lut_query_tests -- query_john_big_log --nocapture | rg "(tail_skipping|lut_query_tests)" --passthru
        cargo test --test lut_query_tests -- query_error_1 --nocapture | rg "(tail_skipping|lut_query_tests)"

## How to benchmark

    export RSONPATH_UNSAFE_FORCE_SIMD="avx2;fast_quotes;fast_popcnt"
    cd crates/rsonpath-benchmarks/
    just init
    // just bench main
    just bench lut_benches

## SSH connection

ssh gienieczko@xeon0.db.in.tum.de -x -t -A
scp -r gienieczko@xeon0.db.in.tum.de:~/src/rsonpath/ricardo-cutoff-results .
scp -r gienieczko@xeon0.db.in.tum.de:~/src/rsonpath/final-results .
scp -r gienieczko@xeon0.db.in.tum.de:~/src/rsonpath/plot-results .
scp -r gienieczko@xeon0.db.in.tum.de:~/src/rsonpath/final-results-2 .
scp -r gienieczko@xeon0.db.in.tum.de:~/src/rsonpath/a.final-results-5 .
scp -r gienieczko@xeon0.db.in.tum.de:~/src/rsonpath/final_results-8 .
scp -r gienieczko@xeon0.db.in.tum.de:~/src/rsonpath/final-results-10 .
cd src/rsonpath
git pull
tmux attach 
  ctrl+b, d                to exit
  ctrl+c                   to kill
  ctrl+b, [                to move 

## TODO

Priority: 1
- Math with T_OPTIMAL being dependent of cutoff NOW COMPARE to rq-lut-cutoff-0 and rq-lut-cutoff-512

Priority: 2
- FIND PAIRS part of build time
