use crate::Instruction;

impl Instruction {
    pub fn decode(input: InputStream) -> DecodingResult {
        let (op, input) = Instruction::pump_one(input)?;

        match op {
            0 => Instruction::decode_push_i(input),
            1 => Ok((Instruction::AddI, input)),
            op => Err(DecodingError::UnknownOpcode(op)),
        }
    }

    fn decode_push_i(input: InputStream) -> DecodingResult {
        let (val, input) = Instruction::pump_four(input)?;
        let instr = Instruction::PushI(val as i32);

        Ok((instr, input))
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

pub type DecodingResult<'a> = TmpDecodingResult<'a, Instruction>;

pub type TmpDecodingResult<'a, T> = Result<(T, InputStream<'a>), DecodingError>;

#[derive(Clone, Debug, PartialEq)]
pub enum DecodingError {
    UnknownOpcode(u8),
    UnexpectedEOF,
}
