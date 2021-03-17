use std::fmt::{Display, Formatter, Result as FResult};

use crate::{Instruction, decode::DecodingError};

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            Instruction::PushI(val) => write!(f, "push_i {}", val),
            Instruction::AddI => write!(f, "add_i"),
            Instruction::FullStop => write!(f, "f_stop"),
        }
    }
}

pub fn disassemble(mut bytecode: &[u8]) -> Result<(), DecodingError> {
    let mut instrs = Vec::new();
    while !bytecode.is_empty() {
        let ((instr, _), tail) = Instruction::decode(bytecode)?;
        bytecode = tail;
        instrs.push(instr);
    }

    for instr in instrs {
        println!("{}", instr);
    }

    Ok(())
}