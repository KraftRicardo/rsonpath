use clap::{Parser, Subcommand};
use std::{error::Error, fs, path::Path};

#[derive(Parser)]
#[command(
    name = "LUT Performance Tool",
    about = "A tool for evaluating performance and distances."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Hello {},
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Hello {  } => {
            say_hello();
        }
    }

    Ok(())
}

fn say_hello(){
    println!("Hello, world!");
}