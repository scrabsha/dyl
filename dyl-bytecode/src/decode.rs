use anyhow::{Context, Result};

use crate::operations::{self, AVAILABLE_DECODERS};
use crate::{operations::DecodingError, Instruction};

impl Instruction {
    pub fn from_bytes(mut input: &[u8]) -> Result<Vec<Instruction>> {
        let mut instrs = Vec::new();
        let mut idx = 0;

        while !input.is_empty() {
            let (instr, len, tail) = Instruction::decode(input)
                .with_context(|| format!("Failed to read instruction at byte {:#06x}", idx))?;

            instrs.push(instr);
            idx += len;
            input = tail;
        }

        Ok(instrs)
    }

    pub fn decode(input: &[u8]) -> Result<(Instruction, usize, &[u8])> {
        let (op, input) = operations::pump_one(input)?;

        AVAILABLE_DECODERS
            .get(op as usize)
            .ok_or(DecodingError::UnknownOpcode(op))?(input)
    }
}
