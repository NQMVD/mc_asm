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

type Code = Vec<String>;

fn load_file(file_name: &str) -> Result<Code> {
    // check if file exists
    if !fs::exists(file_name)? {
        bail!("File does not exist");
    }
    let contents = fs::read_to_string(file_name)?;
    let lines = contents.lines().map(|line| line.to_string()).collect();
    Ok(lines)
}

fn clean_up(code: Code) -> Code {
    let comment_symbol = "//";
    let mut cleaned_code: Code = Vec::new();

    // remove comments and empty lines
    cleaned_code = code
        .iter()
        .filter(|line| !line.is_empty() && !line.starts_with(comment_symbol))
        .map_into()
        .collect::<Code>();

    cleaned_code = cleaned_code
        .iter()
        .map(|line| {
            if line.contains(comment_symbol) {
                let parts = line.split(comment_symbol).collect::<Vec<&str>>();
                parts[0].trim().to_string()
            } else {
                line.trim().to_string()
            }
        })
        .collect::<Code>();

    cleaned_code
}

fn validate_syntax(code: Code) -> Result<()> {
    // validate operands
    // validate operand count
    Ok(())
}

fn assemble(mut assembly_code: Code) -> Result<Code> {
    let mut machine_code: Code = Vec::new();

    assembly_code = clean_up(assembly_code);

    // assembly_code.iter().for_each(|line| println!("{line}"));

    // validate syntax

    // assembly_code.iter().for_each(|line| {
    //     let mut tokens = line.split_whitespace();
    //     let instruction = tokens.next().unwrap();
    //     let args = tokens.collect::<Vec<&str>>();

    //     match instruction {
    //         "mov" => {
    //             let dest = args[0];
    //             let src = args[1];
    //             machine_code.push(format!("mov {}, {}", dest, src));
    //         }
    //         "nop" => {
    //             machine_code.push("nop".to_string());
    //         }
    //         _ => {
    //             machine_code.push(line.to_string());
    //         }
    //     }
    // });

    machine_code = assembly_code;

    Ok(machine_code)
}

fn generate() -> Result<()> {
    Ok(())
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

            let assembly_code = load_file(&as_file)?;

            log.info("File contents:");

            for line in &assembly_code {
                println!("> {line}");
            }
            println!();

            let machine_code = assemble(assembly_code)?;

            log.success("Assembled!");

            for line in machine_code {
                println!("= {line}");
            }

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

            let machine_code = load_file(&cli.file_name)?;

            log.info("File contents:");

            for line in &machine_code {
                println!("> {line}");
            }
            println!();

            Ok(())
        }
        Commands::Full {
            as_file,
            schem_file,
        } => {
            log.info("Full mode...");

            let assembly_code = load_file(&cli.file_name)?;
            Ok(())
        }
    }
}
