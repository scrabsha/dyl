use anyhow::Result;

use std::fmt::{Display, Formatter, Result as FResult};

use crate::{decode::DecodingError, Instruction};

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            Instruction::PushI(val) => write!(f, "push_i {}", val),
            Instruction::AddI => write!(f, "add_i"),
            Instruction::FullStop => write!(f, "f_stop"),
            Instruction::PushC(chr) => write!(f, "push_c {}", chr),
            Instruction::CopyV(idx) => write!(f, "copy_val {}", idx),
            Instruction::Call(offset) => write!(f, "call {}", offset),
            Instruction::Return {
                pointer_offset,
                value_offset,
            } => write!(f, "ret {} {}", value_offset, pointer_offset),
        }
    }
}

pub fn disassemble(mut bytecode: &[u8]) -> Result<()> {
    let mut instrs = Vec::new();
    let mut idx = 0;
    while !bytecode.is_empty() {
        let ((instr, len), tail) = Instruction::decode(bytecode)?;
        bytecode = tail;
        instrs.push((idx, instr));
        idx += len;
    }

    for (pos, instr) in instrs {
        println!("{:#06x}: {}", pos, instr);
    }

    Ok(())
}
