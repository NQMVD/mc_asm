#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_assignments)]

use std::{fs, io::Lines};

use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use itertools::*;
use paris::Logger;
use serde::{Deserialize, Serialize};

mod assembler;
mod generator;

use assembler::{assemble_new, assemble_old};

type Code = Vec<String>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Full,
    Assemble,
    Generate,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Mode to run in
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Assembles the file
    Assemble {
        /// file to assemble
        as_file: String,

        /// optional output file
        mc_file: Option<String>,
    },
    /// Generates a .schem file
    Generate {
        /// file to generate
        mc_file: String,

        /// optional output file
        schem_file: Option<String>,
    },
    /// Assembles and generates
    Full {
        /// file to assemble and generate
        as_file: String,

        /// optional output file
        schem_file: Option<String>,
    },
}

fn load_file(file_name: &str) -> Result<Code> {
    // check if file exists
    if !fs::exists(file_name)? {
        bail!("File does not exist");
    }
    let contents = fs::read_to_string(file_name)?;
    let lines = contents.lines().map(|line| line.to_string()).collect();
    Ok(lines)
}

fn main() -> Result<()> {
    let mut log = Logger::new();
    let cli = Cli::parse();

    match cli.command {
        Commands::Assemble { as_file, mc_file } => {
            log.info("Assembling...");

            if !as_file.ends_with(".as") {
                log.error("File must be a .as file");
                return Ok(());
            }

            // borrow because we might need it again
            let assembly_code = load_file(&as_file)?;

            // DEBUG
            log.info("File contents:");

            for line in &assembly_code {
                println!("> {line}");
            }
            println!();
            // END DEBUG

            // move to consume
            let machine_code = assemble_old(assembly_code)?;

            log.success("Assembled!");

            // DEBUG
            for line in machine_code {
                println!("= {line}");
            }
            println!();
            // END DEBUG

            Ok(())
        }
        Commands::Generate {
            mc_file,
            schem_file,
        } => {
            log.info("Generating...");

            if !mc_file.ends_with(".mc") {
                log.error("File must be a .mc file");
                return Ok(());
            }

            // borrow because we might need it again
            let machine_code = load_file(&mc_file)?;

            // DEBUG
            log.info("File contents:");

            for line in &machine_code {
                println!("> {line}");
            }
            println!();
            // END DEBUG

            Ok(())
        }
        Commands::Full {
            as_file,
            schem_file,
        } => {
            log.info("Full mode...");

            let assembly_code = load_file(&as_file)?;
            Ok(())
        }
    }
}
