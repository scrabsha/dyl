use std::{
    error::Error,
    fmt::{self, Display},
};

use crate::Instruction;

impl Instruction {
    pub fn decode(input: InputStream) -> DecodingResult {
        let (op, input) = Instruction::pump_one(input)?;

        match op {
            0 => Instruction::decode_push_i(input),
            1 => Ok(((Instruction::AddI, 1), input)),
            2 => Ok(((Instruction::FullStop, 1), input)),
            3 => Instruction::decode_push_c(input),
            4 => Instruction::decode_copy_v(input),
            5 => Instruction::decode_call(input),
            6 => Instruction::decode_ret(input),

            op => Err(DecodingError::UnknownOpcode(op)),
        }
    }

    fn decode_push_i(input: InputStream) -> DecodingResult {
        let (val, input) = Instruction::pump_four(input)?;
        let instr = Instruction::PushI(val as i32);

        Ok(((instr, 5), input))
    }

    fn decode_push_c(input: InputStream) -> DecodingResult {
        let (val, input) = Instruction::pump_four(input)?;
        let chr = std::char::from_u32(val).unwrap();
        let instr = Instruction::PushC(chr);

        Ok(((instr, 5), input))
    }

    fn decode_copy_v(input: InputStream) -> DecodingResult {
        let (idx, input) = Instruction::pump_four(input)?;
        let instr = Instruction::CopyV(idx);

        Ok(((instr, 5), input))
    }

    fn decode_call(input: InputStream) -> DecodingResult {
        let (code_pointer, input) = Instruction::pump_four(input)?;
        let instr = Instruction::Call(code_pointer);

        Ok(((instr, 5), input))
    }

    fn decode_ret(input: InputStream) -> DecodingResult {
        let (value_offset, input) = Instruction::pump_four(input)?;
        let (pointer_offset, input) = Instruction::pump_four(input)?;
        let instr = Instruction::Return {
            value_offset,
            pointer_offset,
        };

        Ok(((instr, 9), input))
    }

    fn pump_one(input: InputStream) -> TmpDecodingResult<u8> {
        match input {
            [fst, rest @ ..] => Ok((*fst, rest)),
            _ => Err(DecodingError::UnexpectedEOF),
        }
    }

    fn pump_four(input: InputStream) -> TmpDecodingResult<u32> {
        match input {
            [fst, snd, trd, fth, rest @ ..] => {
                let val = u32::from_be_bytes([*fst, *snd, *trd, *fth]);
                Ok((val, rest))
            }
            _ => Err(DecodingError::UnexpectedEOF),
        }
    }
}

pub type InputStream<'a> = &'a [u8];

pub type DecodingResult<'a> = TmpDecodingResult<'a, (Instruction, usize)>;

pub type TmpDecodingResult<'a, T> = Result<(T, InputStream<'a>), DecodingError>;

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
