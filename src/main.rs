#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_assignments)]

use std::{fs, io::Lines};

use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use itertools::*;
use paris::{error, info, success, warn, Logger};
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

        /// optional old flag
        #[clap(long)]
        old: bool,
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
    /// Remove all .mc files
    Clean,
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

fn write_to_file(file_name: String, code: Code) -> Result<()> {
    // then check if the file already exists
    // if it does, warn the user but continue
    // if it doesn't, write the file
    if fs::exists(&file_name)? {
        warn!("File already exists, overwriting...");
    }

    let contents = code.join("\n");
    fs::write(file_name, contents)?;
    Ok(())
}

fn main() -> Result<()> {
    #[allow(unused_mut)]
    let mut log = Logger::new();
    let cli = Cli::parse();

    match cli.command {
        Commands::Assemble {
            as_file,
            mc_file,
            old,
        } => {
            info!("Assembling...");

            if !as_file.ends_with(".as") {
                error!("File must be a .as file");
                return Ok(());
            }

            // borrow because we might need it again
            let assembly_code = load_file(&as_file)?;

            // move to consume
            // let machine_code = assemble_old(assembly_code)?;
            let machine_code = if old {
                assemble_old(assembly_code)?
            } else {
                assemble_new(assembly_code)?
            };

            success!("Assembled!");

            // write to file
            // first check if output file name is provided
            // if not, use the same name as the input file
            // but with a .mc extension
            let mc_file = mc_file.unwrap_or_else(|| as_file.replace(".as", ".mc"));
            write_to_file(mc_file, machine_code)?;

            success!("Wrote to file!");

            Ok(())
        }
        Commands::Generate {
            mc_file,
            schem_file,
        } => {
            info!("Generating...");

            if !mc_file.ends_with(".mc") {
                error!("File must be a .mc file");
                return Ok(());
            }

            // borrow because we might need it again
            let machine_code = load_file(&mc_file)?;

            // DEBUG
            info!("File contents:");

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
            info!("Full mode...");

            let assembly_code = load_file(&as_file)?;
            Ok(())
        }
        Commands::Clean => {
            info!("Cleaning...");

            // get all files in the current directory
            let files = fs::read_dir(".")?;

            // filter out all .mc files
            let mc_files = files
                .filter_map(|file| {
                    let file = file.ok()?;
                    let file_name = file.file_name();
                    let file_name = file_name.to_string_lossy();
                    if file_name.ends_with(".mc") {
                        Some(file_name.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();

            // delete all .mc files
            for file in &mc_files {
                fs::remove_file(file)?;
            }

            success!("Cleaned! ({} file(s) removed)", mc_files.len());

            Ok(())
        }
    }
}
