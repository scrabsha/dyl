use anyhow::Result;

use std::fmt::{Display, Formatter, Result as FResult};

use crate::Instruction;

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            Instruction::PushI(val) => write!(f, "push_i {}", val),
            Instruction::AddI => write!(f, "add_i"),
            Instruction::FullStop => write!(f, "f_stop"),
            Instruction::PushC(chr) => write!(f, "push_c {}", chr),
            Instruction::CopyV(idx) => write!(f, "copy_val {}", idx),
            Instruction::Ret {
                return_offset,
                pointer_offset,
            } => write!(f, "ret {} {}", return_offset, pointer_offset),
            Instruction::Call(offset) => write!(f, "call {}", offset),
            Instruction::RetW {
                pointer_offset,
                value_offset,
            } => write!(f, "ret_w {} {}", value_offset, pointer_offset),
            Instruction::ResV(offset) => write!(f, "res_v {}", offset),
            Instruction::CopyVS(offset) => write!(f, "copy_v_s {}", offset),
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
