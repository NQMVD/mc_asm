use anyhow::{bail, Result};
use mc_schem::*;

type Code = Vec<String>;
type BlockList = Vec<(Vec<i32>, String)>;

fn set_block(pos: [i32; 3], block: String) {
    // println!("setblock {} {} {} {}", pos[0], pos[1], pos[2], block);
    let schem_shape = [64, 1, 64];

    let mut schem = Schematic::new();

    let mut region = Region::new();
    region.reshape(&schem_shape);
    region.fill_with(&Block::air());
    region.name = "main".to_string();
    schem.regions.push(region);

    let mut md = MetaDataIR::default();
    md.mc_data_version = DataVersion::Java_1_12_2 as i32;
    schem.metadata = md;
}

pub fn generate(machine_code: Code) -> Result<()> {
    // Generate 1024 xz positions
    let mem_start_pos = [-4, -1, 2];
    let mut pos_list = Vec::new();

    for i in 0..2 {
        for j in 0..32 {
            let mut pos = mem_start_pos;

            if i == 1 {
                pos[0] -= 2;
            }

            pos[2] += 2 * j;
            if j >= 16 {
                pos[2] += 4;
            }

            for k in 0..16 {
                pos_list.push(pos.clone());

                if k % 2 == 0 {
                    pos[0] -= 7;
                    pos[2] += if j < 16 { 1 } else { -1 };
                } else {
                    pos[0] -= 7;
                    pos[2] -= if j < 16 { 1 } else { -1 };
                }
            }
        }
    }

    // write instructions to each postition

    // first fill the lines vec with lines of 16 zeros until it reaches 1024 lines
    let mut lines = Vec::new();
    for _ in 0..1024 {
        lines.push("0000000000000000".to_string());
    }
    // then write the machine code to the lines
    for (i, code) in machine_code.iter().enumerate() {
        lines[i] = code.to_string();
    }

    for (adress, line) in lines.iter().enumerate() {
        let face = if adress < 512 { "east" } else { "west" };
        let mut pos = pos_list[adress].clone();

        let byte1 = &line[0..4];
        let byte2 = &line[4..8];

        for (i, char) in byte1.chars().enumerate() {
            let block = if char == '0' {
                "minecraft:purple_wool".to_string()
            } else {
                format!("minecraft:repeater[facing={face}]")
            };
            set_block(pos, block);
            pos[1] -= 2;
        }

        pos[1] -= 2;

        for (i, char) in byte2.chars().enumerate() {
            let block = if char == '0' {
                "minecraft:purple_wool".to_string()
            } else {
                format!("minecraft:repeater[facing={face}]")
            };
            set_block(pos, block);
            pos[1] -= 2;
        }

        // reset program counter
        let pc_start_pos = [-21, -1, -16];
        let mut pos = pc_start_pos;

        for i in 0..10 {
            set_block(
                pos,
                "minecraft:repeater[facing=north,locked=true,powered=false]".to_string(),
            );
            pos[1] -= 2;
        }

        // reset call stack
        let push_start_pos = [-9, -1, -22];
        let pull_start_pos = [-8, -1, -21];

        for i in 0..16 {
            pos = push_start_pos.clone();
            pos[1] -= i * 3;
            for _ in 0..10 {
                set_block(
                    pos,
                    "minecraft:repeater[facing=south,locked=true,powered=false]".to_string(),
                );
                pos[1] -= 2;
            }
        }
        for i in 0..16 {
            pos = pull_start_pos.clone();
            pos[1] -= i * 3;
            for _ in 0..10 {
                set_block(
                    pos,
                    "minecraft:repeater[facing=north,locked=true,powered=false]".to_string(),
                );
                pos[1] -= 2;
            }
        }

        // reset flags
        let flag_start_pos = [-26, -17, -60];
        let mut pos = flag_start_pos.clone();

        set_block(
            pos,
            "minecraft:repeater[facing=west,locked=true,powered=false]".to_string(),
        );
        pos[2] -= 4;
        set_block(
            pos,
            "minecraft:repeater[facing=west,locked=true,powered=false]".to_string(),
        );

        // reset data mem

        let data_start_pos = [-47, -3, -9];
        let mut pos_list_north = Vec::new();

        for i in 0..4 {
            let mut pos = data_start_pos.clone();
            pos[2] -= 16 * i;
            for j in 0..16 {
                pos_list_north.push(pos.clone());
                pos[0] -= 2;
                if j % 2 == 0 {
                    pos[2] += 1;
                } else {
                    pos[2] -= 1;
                }
            }

            let mut pos = data_start_pos.clone();
            pos[2] -= 16 * i;
            pos[0] -= 36;
            pos[1] += 1;
            for j in 0..16 {
                pos_list_north.push(pos.clone());
                pos[0] -= 2;
                if j % 2 == 0 {
                    pos[2] += 1;
                } else {
                    pos[2] -= 1;
                }
            }
        }

        // setblock at last 3 positions of pos list north
        for pos in pos_list_north.iter().skip(pos_list_north.len() - 3) {
            let mut x = pos.clone();
            for _ in 0..8 {
                set_block(
                    x,
                    "minecraft:repeater[facing=north,locked=true,powered=false]".to_string(),
                );
                x[1] -= 2;
            }
        }

        for pos in pos_list_north {
            let mut x = pos.clone();
            for _ in 0..8 {
                set_block(
                    x,
                    "minecraft:repeater[facing=north,locked=true,powered=false]".to_string(),
                );
                x[1] -= 2;
            }
        }

        // reset registers
        let reg_start_pos = [-35, -3, -12];
        let mut pos_list_east = Vec::new();

        let mut pos = reg_start_pos.clone();
        for i in 0..15 {
            pos_list_east.push(pos.clone());
            pos[0] -= 2;
            if i % 2 == 0 {
                pos[2] -= 1;
            } else {
                pos[2] += 1;
            }
        }

        for pos in pos_list_east {
            let mut x = pos.clone();
            for _ in 0..8 {
                set_block(
                    x,
                    "minecraft:repeater[facing=east,locked=true,powered=false]".to_string(),
                );
                x[1] -= 2;
            }
            let mut x = pos.clone();
            x[0] += 2;
            for _ in 0..8 {
                set_block(
                    x,
                    "minecraft:repeater[facing=west,locked=true,powered=false]".to_string(),
                );
                x[1] -= 2;
            }
        }
    }

    Ok(())
}
