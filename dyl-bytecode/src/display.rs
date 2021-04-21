use anyhow::Result;

use std::fmt::{Display, Formatter, Result as FResult};

use crate::Instruction;

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            Instruction::PushI(op) => op.fmt(f),
            Instruction::AddI(op) => op.fmt(f),
            Instruction::FStop(op) => op.fmt(f),
            Instruction::PushCopy(op) => op.fmt(f),
            Instruction::Ret(op) => op.fmt(f),
            Instruction::Call(op) => op.fmt(f),
            Instruction::ResV(op) => op.fmt(f),
            Instruction::PopCopy(op) => op.fmt(f),
            Instruction::Goto(op) => op.fmt(f),
            Instruction::CondJmp(op) => op.fmt(f),
            Instruction::Neg(op) => op.fmt(f),
            Instruction::Mul(op) => op.fmt(f),
        }
    }
}

pub fn disassemble(mut bytecode: &[u8]) -> Result<()> {
    let mut instrs = Vec::new();
    let mut idx = 0;
    while !bytecode.is_empty() {
        let (instr, len, tail) = Instruction::decode(bytecode)?;
        bytecode = tail;
        instrs.push((idx, instr));
        idx += len;
    }

    for (pos, instr) in instrs {
        println!("{:#06x}: {}", pos, instr);
    }

    Ok(())
}
