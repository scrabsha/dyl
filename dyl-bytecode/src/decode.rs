use crate::Instruction;

impl Instruction {
    pub fn decode(input: InputStream) -> DecodingResult {
        let (op, input) = Instruction::pump_one(input)?;
        
        match op {
            0 => Instruction::decode_push_i(input),
            1 => Instruction::decode_sum_i(input),
            op => Err(DecodingError::UnknownOpcode(op)),
        }
    }

    fn decode_push_i(input: InputStream) -> DecodingResult {
        let (val, input) = Instruction::pump_four(input)?;
        let instr = Instruction::PushI(val);

        Ok((instr, input))
    }

    fn decode_sum_i(input: InputStream) -> DecodingResult {
        let instr = Instruction::AddI;

        Ok((instr, input))
    }

    fn pump_one(input: InputStream) -> TmpDecodingResult<i8> {
        match input {
            [fst, rest @ .. ] => Ok((*fst, rest)),
            _ => Err(DecodingError::UnexpectedEOF),
        }
    }

    fn pump_four(input: InputStream) -> TmpDecodingResult<i32> {
        match input {
            [fst, snd, trd, fth, rest @ .. ] => {
                let val = group_to_i32(*fst, *snd, *trd, *fth);
                Ok((val, rest))
            },
            _ => Err(DecodingError::UnexpectedEOF),
        }
    }
}

pub type InputStream<'a> = &'a [i8];

pub type DecodingResult<'a> = TmpDecodingResult<'a, Instruction>;

pub type TmpDecodingResult<'a, T> = Result<(T, &'a [i8]), DecodingError>;

#[derive(Clone, Debug, PartialEq)]
pub enum DecodingError {
    UnknownOpcode(i8),
    UnexpectedEOF,
}

fn group_to_i32(a: i8, b: i8, c: i8, d: i8) -> i32 {
    (a as i32) << 24 + (b as i32) << 16 + (c as i32) << 8 + (d as i32)
}