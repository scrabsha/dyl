use anyhow::{anyhow, bail, Context, Result};

use std::{
    error::Error,
    fmt::{self, Display},
};

use crate::Instruction;

impl Instruction {
    pub fn from_bytes(mut input: &[u8]) -> Result<Vec<Instruction>> {
        let mut instrs = Vec::new();
        let mut idx = 0;

        while !input.is_empty() {
            let ((instr, len), tail) = Instruction::decode(input)
                .with_context(|| format!("Failed to read instruction at byte {:#06x}", idx))?;

            instrs.push(instr);
            idx += len;
            input = tail;
        }

        Ok(instrs)
    }

    pub fn decode(input: InputStream) -> DecodingResult {
        let (op, input) = Instruction::pump_one(input)?;

        match op {
            0 => Instruction::decode_push_i(input).context("Failed to decode `push_i` instruction"),
            1 => Ok(((Instruction::AddI, 1), input)),
            2 => Ok(((Instruction::FullStop, 1), input)),
            3 => Instruction::decode_push_c(input).context("Failed to decode `push_c` instruction"),
            4 => Instruction::decode_copy_v(input).context("Failed to decode `copy_v` instruction"),
            5 => Instruction::decode_call(input).context("Failed to decode `call` instruction"),
            6 => Instruction::decode_ret_w(input).context("Failed to decode `ret_w` instruction"),
            7 => Instruction::decode_ret(input).context("Failed to decode `ret` instruction"),
            8 => Instruction::decode_res_v(input).context("Failed to decode `res_v` instruction"),
            9 => Instruction::decode_copy_v_s(input)
                .context("Failed to decode `copy_v_s` instruction"),

            op => bail!(DecodingError::UnknownOpcode(op)),
        }
    }

    fn decode_push_i(input: InputStream) -> DecodingResult {
        let (val, input) =
            Instruction::pump_four(input).context("Failed to get integer to push")?;
        let instr = Instruction::PushI(val as i32);

        Ok(((instr, 5), input))
    }

    fn decode_push_c(input: InputStream) -> DecodingResult {
        let (val, input) = Instruction::pump_four(input).context("Failed to get char to push")?;
        let chr = std::char::from_u32(val)
            .with_context(|| format!("Failed to convert {:#x} to char", val))?;
        let instr = Instruction::PushC(chr);

        Ok(((instr, 5), input))
    }

    fn decode_copy_v(input: InputStream) -> DecodingResult {
        let (idx, input) =
            Instruction::pump_four(input).context("Failed to get stack address to copy")?;
        let instr = Instruction::CopyV(idx);

        Ok(((instr, 5), input))
    }

    fn decode_call(input: InputStream) -> DecodingResult {
        let (code_pointer, input) =
            Instruction::pump_four(input).context("Failed to get function address to call")?;
        let instr = Instruction::Call(code_pointer);

        Ok(((instr, 5), input))
    }

    fn decode_ret_w(input: InputStream) -> DecodingResult {
        let (value_offset, input) =
            Instruction::pump_four(input).context("Failed to get value address to return")?;
        let (pointer_offset, input) = Instruction::pump_four(input)
            .context("Failed to get function pointer to return back")?;
        let instr = Instruction::RetW {
            value_offset,
            pointer_offset,
        };

        Ok(((instr, 9), input))
    }

    fn decode_ret(input: InputStream) -> DecodingResult {
        let (return_offset, input) =
            Instruction::pump_four(input).context("Failed to get return offset")?;
        let (pointer_offset, input) = Instruction::pump_four(input)
            .context("Failed to get function pointer to return back")?;
        let instr = Instruction::Ret {
            return_offset,
            pointer_offset,
        };

        Ok(((instr, 9), input))
    }

    fn decode_res_v(input: InputStream) -> DecodingResult {
        let (offset_to_reserve, input) = Instruction::pump_four(input)
            .context("Failed to get the amount of space to reserve")?;
        let instr = Instruction::ResV(offset_to_reserve);

        Ok(((instr, 5), input))
    }

    fn decode_copy_v_s(input: InputStream) -> DecodingResult {
        let (offset, input) =
            Instruction::pump_four(input).context("Failed to get copy destination")?;
        let instr = Instruction::CopyVS(offset);

        Ok(((instr, 5), input))
    }

    fn pump_one(input: InputStream) -> TmpDecodingResult<u8> {
        match input {
            [fst, rest @ ..] => Ok((*fst, rest)),
            _ => Err(anyhow!(DecodingError::UnexpectedEOF))
                .context("Failed to get one byte from input"),
        }
    }

    fn pump_four(input: InputStream) -> TmpDecodingResult<u32> {
        match input {
            [fst, snd, trd, fth, rest @ ..] => {
                let val = u32::from_be_bytes([*fst, *snd, *trd, *fth]);
                Ok((val, rest))
            }
            _ => Err(anyhow!(DecodingError::UnexpectedEOF))
                .context("Failed to get four bytes from input"),
        }
    }
}

pub type InputStream<'a> = &'a [u8];

pub type DecodingResult<'a> = TmpDecodingResult<'a, (Instruction, usize)>;

pub type TmpDecodingResult<'a, T> = Result<(T, InputStream<'a>)>;

#[derive(Clone, Debug, PartialEq)]
pub enum DecodingError {
    UnknownOpcode(u8),
    UnexpectedEOF,
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodingError::UnexpectedEOF => write!(f, "Unexpected EOF"),
            DecodingError::UnknownOpcode(id) => write!(f, "Unknown opcode: `{}`", id),
        }
    }
}

impl Error for DecodingError {}
