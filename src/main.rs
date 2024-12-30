#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_assignments)]

use std::{fs, io::Lines};

use anyhow::Result;
use clap::{Parser, ValueEnum};
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
struct Cli {
    /// Assemble, Generate, Full(both)
    #[arg(value_enum)]
    mode: Mode,

    /// the file to use
    file_name: String,
}

type Code = Vec<String>;

fn load_file(file_name: &str) -> Result<Code> {
    let contents = fs::read_to_string(file_name)?;
    let lines = contents.lines().map(|line| line.to_string()).collect();
    Ok(lines)
}

fn assemble(mut assembly_code: Code) -> Result<Code> {
    let comment_symbol = "//";
    let mut machine_code: Code = Vec::new();

    // clean up: remove comments and empty lines
    assembly_code = assembly_code
        .iter()
        .filter(|line| !line.is_empty() && !line.starts_with(comment_symbol))
        .map_into()
        .collect::<Code>();

    assembly_code = assembly_code
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

    assembly_code.iter().for_each(|line| println!("{line}"));

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
    //         "add" => {
    //             let dest = args[0];
    //             let src = args[1];
    //             machine_code.push(format!("add {}, {}", dest, src));
    //         }
    //         "sub" => {
    //             let dest = args[0];
    //             let src = args[1];
    //             machine_code.push(format!("sub {}, {}", dest, src));
    //         }
    //         "mul" => {
    //             let dest = args[0];
    //             let src = args[1];
    //             machine_code.push(format!("mul {}, {}", dest, src));
    //         }
    //         "div" => {
    //             let dest = args[0];
    //             let src = args[1];
    //             machine_code.push(format!("div {}, {}", dest, src));
    //         }
    //         "jmp" => {
    //             let label = args[0];
    //             machine_code.push(format!("jmp {}", label));
    //         }
    //         "je" => {
    //             let label = args[0];
    //             machine_code.push(format!("je {}", label));
    //         }
    //         "jne" => {
    //             let label = args[0];
    //             machine_code.push(format!("jne {}", label));
    //         }
    //         "jg" => {
    //             let label = args[0];
    //             machine_code.push(format!("jg {}", label));
    //         }
    //         "jge" => {
    //             let label = args[0];
    //             machine_code.push(format!("jge {}", label));
    //         }
    //         "jl" => {
    //             let label = args[0];
    //             machine_code.push(format!("jl {}", label));
    //         }
    //         "jle" => {
    //             let label = args[0];
    //             machine_code.push(format!("jle {}", label));
    //         }
    //         "call" => {
    //             let label = args[0];
    //             machine_code.push(format!("call {}", label));
    //         }
    //         "ret" => {
    //             machine_code.push("ret".to_string());
    //         }
    //         "push" => {
    //             let src = args[0];
    //             machine_code.push(format!("push {}", src));
    //         }
    //         "pop" => {
    //             let dest = args[0];
    //             machine_code.push(format!("pop {}", dest));
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

    log.info(format!("mode: {:?}", cli.mode));
    log.info(format!("file: {:?}", cli.file_name));

    if cli.mode == Mode::Generate || cli.mode == Mode::Full {
        log.error("Not yet implemented...");
        return Ok(());
    }

    let assembly_code = load_file(&cli.file_name)?;

    log.info("File contents:");

    for ele in &assembly_code {
        println!("{ele}");
    }
    println!();

    let machine_code = assemble(assembly_code.clone())?;

    log.success("Assembled!");

    for ele in machine_code {
        println!("{ele}");
    }

    Ok(())
}
