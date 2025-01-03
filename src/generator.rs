use anyhow::{bail, Result};

type Code = Vec<String>;

pub fn generate(machine_code: Code) -> Result<()> {
    // Generate 1024 xz positions
    let mem_start_pos = [-4, -1, 2];
    let pos_list = Vec::new();

    for i in 0..2 {
        for j in 0..32 {}
    }

    Ok(())
}
