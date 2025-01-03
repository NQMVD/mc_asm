use anyhow::{bail, Result};
use itertools::Itertools;
use regex::Regex;
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Write},
    process,
};

type Code = Vec<String>;

#[derive(Debug)]
struct AssemblyError {
    message: String,
}

impl std::fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for AssemblyError {}

// Helper to create AssemblyError
fn assembly_error(msg: &str, line_num: Option<usize>, line: Option<&str>) -> AssemblyError {
    let mut formatted = format!("\x1b[31mError\x1b[0m: {}\n", msg);
    if let Some(num) = line_num {
        formatted.push_str(&format!(
            "\x1b[34m --> \x1b[0mFILE \x1b[34mline\x1b[0m {}\n",
            num
        ));
        if let Some(l) = line {
            let indent = format!("{}", num).len();
            formatted.push_str(&format!(
                "\x1b[34m{:indent$} |\n\x1b[0m",
                "",
                indent = indent
            ));
            formatted.push_str(&format!("{} |\x1b[0m {}\n", num, l));
            formatted.push_str(&format!(
                "\x1b[34m{:indent$} |\x1b[0m\n",
                "",
                indent = indent
            ));
        }
    }
    AssemblyError { message: formatted }
}

#[derive(Debug)]
struct OperandError {
    inner: AssemblyError,
}

impl std::fmt::Display for OperandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.message)
    }
}

impl Error for OperandError {}

fn operand_error(msg: &str, line_num: usize, line: Option<&str>) -> OperandError {
    OperandError {
        inner: assembly_error(msg, Some(line_num), line),
    }
}

/// old assembler
pub fn assemble_old(mut assembly_code: Code) -> Result<Code> {
    // -------------------- Constants --------------------
    const MAX_REGISTER: i32 = 1 << 4; // 2**4
    const MAX_IMMEDIATE: i32 = 255;
    const MIN_IMMEDIATE: i32 = -128;
    const MAX_OFFSET: i32 = 7;
    const MIN_OFFSET: i32 = -8;
    const MAX_ADDRESS: i32 = 1 << 10; // 2**10

    // Opcodes (same order as original)
    const OPC_NOP: i32 = 0;
    const OPC_HLT: i32 = 1;
    const OPC_ADD: i32 = 2;
    const OPC_SUB: i32 = 3;
    const OPC_NOR: i32 = 4;
    const OPC_AND: i32 = 5;
    const OPC_XOR: i32 = 6;
    const OPC_RSH: i32 = 7;
    const OPC_LDI: i32 = 8;
    const OPC_ADI: i32 = 9;
    const OPC_JMP: i32 = 10;
    const OPC_BRH: i32 = 11;
    const OPC_CAL: i32 = 12;
    const OPC_RET: i32 = 13;
    const OPC_LOD: i32 = 14;
    const OPC_STR: i32 = 15;

    // For better error messages, let’s map opcode -> name
    static OPCODE_NAMES: [&str; 16] = [
        "nop", "hlt", "add", "sub", "nor", "and", "xor", "rsh", "ldi", "adi", "jmp", "brh", "cal",
        "ret", "lod", "str",
    ];

    // -------------------- Helper Functions --------------------

    fn populate_symbols(symbols: &[&str], offset: i32) -> HashMap<String, i32> {
        let mut map = HashMap::new();
        for (i, s) in symbols.iter().enumerate() {
            map.insert(s.to_string(), i as i32 + offset);
        }
        map
    }

    fn populate_symbol_table() -> HashMap<String, i32> {
        let mut symbols = HashMap::new();

        // Add opcodes
        let opcodes = [
            "nop", "hlt", "add", "sub", "nor", "and", "xor", "rsh", "ldi", "adi", "jmp", "brh",
            "cal", "ret", "lod", "str",
        ];
        symbols.extend(populate_symbols(&opcodes, 0));

        // registers
        for i in 0..16 {
            symbols.insert(format!("r{}", i), i);
        }

        // conditions
        let conditions = vec![
            vec!["eq", "ne", "ge", "lt"],
            vec!["=", "!=", ">=", "<"],
            vec!["z", "nz", "c", "nc"],
            vec!["zero", "notzero", "carry", "notcarry"],
        ];
        for group in conditions {
            for (i, c) in group.iter().enumerate() {
                symbols.insert(c.to_string(), i as i32);
            }
        }

        // ports (offset 240)
        let ports = [
            "pixel_x",
            "pixel_y",
            "draw_pixel",
            "clear_pixel",
            "load_pixel",
            "buffer_screen",
            "clear_screen_buffer",
            "write_char",
            "buffer_chars",
            "clear_chars_buffer",
            "show_number",
            "clear_number",
            "signed_mode",
            "unsigned_mode",
            "rng",
            "controller_input",
        ];
        symbols.extend(populate_symbols(&ports, 240));

        // single characters
        let chars = " abcdefghijklmnopqrstuvwxyz.!?";
        for (i, letter) in chars.chars().enumerate() {
            let ch = letter.to_string();
            symbols.insert(format!("\"{}\"", ch), i as i32);
            symbols.insert(format!("'{}'", ch), i as i32);
        }

        symbols
    }

    fn is_definition(word: &str) -> bool {
        word == "define"
    }

    fn is_label(word: &str) -> bool {
        word.starts_with('.')
    }

    // No more reversing opcode -> string; just use numeric logic
    fn opcode_name(opcode: i32) -> &'static str {
        if (0..OPCODE_NAMES.len() as i32).contains(&opcode) {
            OPCODE_NAMES[opcode as usize]
        } else {
            "???"
        }
    }

    // Return how many operands each opcode needs
    fn operand_count_for_opcode(opcode: i32) -> usize {
        match opcode {
            // (nop, hlt, ret)
            OPC_NOP | OPC_HLT | OPC_RET => 1,

            // (jmp, cal)
            OPC_JMP | OPC_CAL => 2,

            // (rsh, ldi, adi, brh)
            OPC_RSH | OPC_LDI | OPC_ADI | OPC_BRH => 3,

            // (add, sub, nor, and, xor, lod, str)
            OPC_ADD | OPC_SUB | OPC_NOR | OPC_AND | OPC_XOR | OPC_LOD | OPC_STR => 4,

            // If it's something else out of range, let’s just say 0
            _ => 0,
        }
    }

    fn resolve(
        symbols: &HashMap<String, i32>,
        word: &str,
        line_num: usize,
        line: &str,
    ) -> Result<i32, AssemblyError> {
        // numeric literal or symbol
        if word.starts_with('-') || word.chars().next().unwrap().is_ascii_digit() {
            // parse
            // if it starts with 0x, parse as hex
            // if it starts with 0b, parse as binary
            // otherwise, parse as decimal
            let radix = if word.starts_with("0x") {
                16
            } else if word.starts_with("0b") {
                2
            } else {
                10
            };
            let val = i32::from_str_radix(
                word.trim_start_matches("0x").trim_start_matches("0b"),
                radix,
            )
            .map_err(|_| {
                assembly_error(
                    &format!("Could not parse number '{}'", word),
                    Some(line_num),
                    Some(line),
                )
            })?;
            Ok(val)
        } else {
            match symbols.get(word) {
                Some(val) => Ok(*val),
                None => Err(assembly_error(
                    &format!("Could not resolve symbol '{}'", word),
                    Some(line_num),
                    Some(line),
                )),
            }
        }
    }

    type PseudoFn = Box<dyn Fn(&[String]) -> Vec<String>>;
    // Pseudo-instructions => real instructions
    fn resolve_pseudo_instructions(words: &[String]) -> Vec<String> {
        use std::collections::HashMap;

        // Make the HashMap store trait objects
        let mut m: HashMap<&str, PseudoFn> = HashMap::new();

        m.insert(
            "cmp",
            Box::new(|w: &[String]| {
                vec![
                    "sub".to_string(),
                    w[1].clone(),
                    w[2].clone(),
                    "r0".to_string(),
                ]
            }),
        );

        m.insert(
            "mov",
            Box::new(|w: &[String]| {
                vec![
                    "add".to_string(),
                    w[1].clone(),
                    "r0".to_string(),
                    w[2].clone(),
                ]
            }),
        );

        m.insert(
            "lsh",
            Box::new(|w: &[String]| {
                vec!["add".to_string(), w[1].clone(), w[1].clone(), w[2].clone()]
            }),
        );

        m.insert(
            "inc",
            Box::new(|w: &[String]| vec!["adi".to_string(), w[1].clone(), "1".to_string()]),
        );

        m.insert(
            "dec",
            Box::new(|w: &[String]| vec!["adi".to_string(), w[1].clone(), "-1".to_string()]),
        );

        m.insert(
            "not",
            Box::new(|w: &[String]| {
                vec![
                    "nor".to_string(),
                    w[1].clone(),
                    "r0".to_string(),
                    w[2].clone(),
                ]
            }),
        );

        m.insert(
            "neg",
            Box::new(|w: &[String]| {
                vec![
                    "sub".to_string(),
                    "r0".to_string(),
                    w[1].clone(),
                    w[2].clone(),
                ]
            }),
        );

        if let Some(expander) = m.get(words[0].as_str()) {
            expander(words)
        } else {
            words.to_vec()
        }
    }

    // Validate by numeric opcode
    fn validate_operand_count(
        opcode: i32,
        words: &[i32],
        pc: usize,
        line: &str,
    ) -> Result<(), OperandError> {
        let expected = operand_count_for_opcode(opcode);
        if words.len() != expected {
            return Err(operand_error(
                &format!(
                    "Incorrect number of operands for '{}' (expected {}, got {})",
                    opcode_name(opcode),
                    expected,
                    words.len()
                ),
                pc,
                Some(line),
            ));
        }
        Ok(())
    }

    // Build machine code by numeric opcode
    fn build_machine_code(
        opcode: i32,
        words: &[i32],
        machine_code: u16,
        pc: usize,
        line: &str,
    ) -> Result<u16, OperandError> {
        let mut code = machine_code;

        // regA check
        if matches!(
            opcode,
            OPC_ADD
                | OPC_SUB
                | OPC_NOR
                | OPC_AND
                | OPC_XOR
                | OPC_RSH
                | OPC_LDI
                | OPC_ADI
                | OPC_LOD
                | OPC_STR
        ) {
            if words[1] >= MAX_REGISTER {
                return Err(operand_error(
                    &format!("Invalid reg A for '{}'", opcode_name(opcode)),
                    pc,
                    Some(line),
                ));
            }
            code |= (words[1] << 8) as u16;
        }

        // regB check
        if matches!(
            opcode,
            OPC_ADD | OPC_SUB | OPC_NOR | OPC_AND | OPC_XOR | OPC_LOD | OPC_STR
        ) {
            if words[2] >= MAX_REGISTER {
                return Err(operand_error(
                    &format!("Invalid reg B for '{}'", opcode_name(opcode)),
                    pc,
                    Some(line),
                ));
            }
            code |= (words[2] << 4) as u16;
        }

        // regC check
        if matches!(
            opcode,
            OPC_ADD | OPC_SUB | OPC_NOR | OPC_AND | OPC_XOR | OPC_RSH
        ) {
            let c = words[words.len() - 1];
            if c >= MAX_REGISTER {
                return Err(operand_error(
                    &format!("Invalid reg C for '{}'", opcode_name(opcode)),
                    pc,
                    Some(line),
                ));
            }
            code |= c as u16;
        }

        // immediate
        if matches!(opcode, OPC_LDI | OPC_ADI) {
            let imm = words[2];
            if !(MIN_IMMEDIATE..=MAX_IMMEDIATE).contains(&imm) {
                return Err(operand_error(
                    &format!("Invalid immediate value '{}'", imm),
                    pc,
                    Some(line),
                ));
            }
            code |= (imm & 0xFF) as u16;
        }

        // jump address
        if matches!(opcode, OPC_JMP | OPC_BRH | OPC_CAL) {
            let addr = words[words.len() - 1];
            if addr >= MAX_ADDRESS {
                return Err(operand_error(
                    &format!("Invalid address '{}'", addr),
                    pc,
                    Some(line),
                ));
            }
            code |= addr as u16;
        }

        // offset
        if matches!(opcode, OPC_LOD | OPC_STR) {
            let offset = words[3];
            if !(MIN_OFFSET..=MAX_OFFSET).contains(&offset) {
                return Err(operand_error(
                    &format!("Invalid offset '{}'", offset),
                    pc,
                    Some(line),
                ));
            }
            code |= (offset & 0xF) as u16;
        }

        // condition
        if opcode == OPC_BRH {
            if words[1] >= (1 << 2) {
                return Err(operand_error(
                    &format!("Invalid condition for '{}'", opcode_name(opcode)),
                    pc,
                    Some(line),
                ));
            }
            code |= (words[1] << 10) as u16;
        }

        Ok(code)
    }

    // -------------------- Main Assembler --------------------

    let comment_regex = Regex::new(r"[/#;].*")?;
    let mut lines = Vec::new();

    // Strip comments, gather lines
    for line in assembly_code {
        let cleaned = comment_regex.replace(&line, "");
        let trimmed = cleaned.trim();
        if !trimmed.is_empty() {
            lines.push(trimmed.to_lowercase());
        }
    }

    let mut symbols = populate_symbol_table();

    // First pass: definitions & labels
    let mut pc: i32 = 0;
    let mut instructions: Vec<(usize, Vec<String>)> = vec![];

    for (line_num, line) in lines.iter().enumerate() {
        let tokens: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
        if tokens.is_empty() {
            continue;
        }
        if is_definition(&tokens[0]) {
            // define <sym> <val>
            if tokens.len() >= 3 {
                if let Ok(val) = i32::from_str_radix(
                    tokens[2].trim_start_matches("0x"),
                    if tokens[2].contains("0x") { 16 } else { 10 },
                ) {
                    symbols.insert(tokens[1].clone(), val);
                }
            }
        } else if is_label(&tokens[0]) {
            // label
            symbols.insert(tokens[0].clone(), pc);
            if tokens.len() > 1 {
                pc += 1;
                instructions.push((line_num, tokens[1..].to_vec()));
            }
        } else {
            pc += 1;
            instructions.push((line_num, tokens));
        }
    }

    // Second pass: generate machine code
    let mut result_machine_code: Code = Vec::new();

    for (pc_index, (line_num, tokens)) in instructions.iter().enumerate() {
        // pseudo-instructions
        let tokens = resolve_pseudo_instructions(tokens);

        // handle optional offset for lod/str
        let mut tokens = tokens.clone();
        if (tokens[0] == "lod" || tokens[0] == "str") && tokens.len() == 3 {
            tokens.push("0".to_string());
        }

        // space special case
        if tokens.len() >= 2 {
            let last = tokens.last().unwrap();
            let second_last = &tokens[tokens.len() - 2];
            if (last == "'" || last == "\"") && (second_last == "'" || second_last == "\"") {
                // replace them with "' '"
                tokens.pop();
                tokens.pop();
                tokens.push("' '".to_string());
            }
        }

        // opcode int
        let opcode_num = resolve(&symbols, &tokens[0], *line_num, &lines[*line_num])?;

        // shift opcode into top 4 bits
        let machine_code: u16 = ((opcode_num as u16) << 12) & 0xFFFF;

        // convert all tokens
        let resolved: Vec<i32> = tokens
            .iter()
            .map(|t| resolve(&symbols, t, *line_num, &lines[*line_num]))
            .collect::<Result<Vec<_>, _>>()?;

        // validate operand count by numeric opcode
        validate_operand_count(opcode_num, &resolved, pc_index, &lines[*line_num])?;

        // build
        let code = build_machine_code(
            opcode_num,
            &resolved,
            machine_code,
            pc_index,
            &lines[*line_num],
        )?;

        // write 16-bit binary
        // writeln!(result_machine_code, "{:016b}", code)?;
        result_machine_code.push(format!("{:016b}", code));
    }

    Ok(result_machine_code)
}

/// new assembler
pub fn assemble_new(mut assembly_code: Code) -> Result<Code> {
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
