use anyhow::{anyhow, Context, Result};

use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FResult},
};

use crate::Instruction;

pub(crate) const AVAILABLE_DECODERS: [Decoder; 11] = [
    PushI::decode_and_wrap,
    AddI::decode_and_wrap,
    FStop::decode_and_wrap,
    PushCopy::decode_and_wrap,
    Call::decode_and_wrap,
    Ret::decode_and_wrap,
    ResV::decode_and_wrap,
    PopCopy::decode_and_wrap,
    Goto::decode_and_wrap,
    CondJmp::decode_and_wrap,
    Neg::decode_and_wrap,
];

pub(crate) type Decoder = fn(&[u8]) -> Result<(Instruction, usize, &[u8])>;

pub(crate) trait Operation: Sized + Into<Instruction> {
    const ID: usize;
    const SIZE: usize;
    const DISPLAY_NAME: &'static str;

    fn decode(input: &[u8]) -> Result<(Self, &[u8])>;

    fn decode_and_wrap(input: &[u8]) -> Result<(Instruction, usize, &[u8])> {
        Self::decode(input)
            .with_context(|| format!("Failed to decode `{}`", Self::DISPLAY_NAME))
            .map(|(op, tail)| (op.into(), Self::SIZE, tail))
    }
}

macro_rules! next_id {
    ($t:ident) => {
        <$t as Operation>::ID + 1
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PushI(pub i32);

impl Operation for PushI {
    const ID: usize = 0;
    const SIZE: usize = 5;
    const DISPLAY_NAME: &'static str = "push_i";

    fn decode(input: &[u8]) -> Result<(Self, &[u8])> {
        let (i, input) = pump_four(input).context("Failed to get integer to push")?;
        let instr = PushI(i as i32);

        Ok((instr, input))
    }
}

impl Display for PushI {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "push_i {}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AddI;

impl Operation for AddI {
    const ID: usize = next_id![PushI];
    const SIZE: usize = 1;
    const DISPLAY_NAME: &'static str = "add_i";

    fn decode(input: &[u8]) -> Result<(Self, &[u8])> {
        let instr = AddI;

        Ok((instr, input))
    }
}

impl Display for AddI {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "add_i")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FStop;

impl Operation for FStop {
    const ID: usize = next_id![AddI];
    const SIZE: usize = 1;
    const DISPLAY_NAME: &'static str = "f_stop";

    fn decode(input: &[u8]) -> Result<(Self, &[u8])> {
        let instr = FStop;

        Ok((instr, input))
    }
}

impl Display for FStop {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "f_stop")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PushCopy(pub u16);

impl Operation for PushCopy {
    const ID: usize = next_id![FStop];
    const SIZE: usize = 3;
    const DISPLAY_NAME: &'static str = "push_copy";

    fn decode(input: &[u8]) -> Result<(Self, &[u8])> {
        let (idx, input) = pump_two(input).context("Failed to get stack offset to copy")?;
        let instr = PushCopy(idx);

        Ok((instr, input))
    }
}

impl Display for PushCopy {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "push_copy {}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Call(pub u32);

impl Operation for Call {
    const ID: usize = next_id![PushCopy];
    const SIZE: usize = 5;
    const DISPLAY_NAME: &'static str = "call";

    fn decode(input: &[u8]) -> Result<(Self, &[u8])> {
        let (idx, input) = pump_four(input).context("Failed to get function address to call")?;
        let instr = Call(idx);

        Ok((instr, input))
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "call {}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ret {
    pub shrink_offset: u16,
    pub ip_offset: u16,
}

impl Operation for Ret {
    const ID: usize = next_id![Call];
    const SIZE: usize = 5;
    const DISPLAY_NAME: &'static str = "ret";

    fn decode(input: &[u8]) -> Result<(Self, &[u8])> {
        let (shrink_offset, input) = pump_two(input).context("Failed to get new stack top")?;
        let (ip_offset, input) =
            pump_two(input).context("Failed to get instruction pointer to return to")?;
        let instr = Ret {
            shrink_offset,
            ip_offset,
        };

        Ok((instr, input))
    }
}

impl Display for Ret {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "ret {} {}", self.shrink_offset, self.ip_offset)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResV(pub u16);

impl Operation for ResV {
    const ID: usize = next_id![Ret];
    const SIZE: usize = 3;
    const DISPLAY_NAME: &'static str = "res_v";

    fn decode(input: &[u8]) -> Result<(Self, &[u8])> {
        let (amount_to_reserve, input) =
            pump_two(input).context("Failed to get amount of space to reserve")?;
        let instr = ResV(amount_to_reserve);

        Ok((instr, input))
    }
}

impl Display for ResV {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "res_v {}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopCopy(pub u16);

impl Operation for PopCopy {
    const ID: usize = next_id![ResV];
    const SIZE: usize = 3;
    const DISPLAY_NAME: &'static str = "pop_copy";

    fn decode(input: &[u8]) -> Result<(Self, &[u8])> {
        let (offset_to_replace, input) =
            pump_two(input).context("Failed to get copy destination")?;
        let instr = PopCopy(offset_to_replace);

        Ok((instr, input))
    }
}

impl Display for PopCopy {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "pop_copy {}", self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Goto(pub u32);

impl Operation for Goto {
    const ID: usize = next_id![PopCopy];
    const SIZE: usize = 5;
    const DISPLAY_NAME: &'static str = "goto";

    fn decode(input: &[u8]) -> Result<(Self, &[u8])> {
        let (addr, rest) = pump_four(input).context("Failed to get goto destination")?;
        let instr = Goto(addr);

        Ok((instr, rest))
    }
}

impl Display for Goto {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "goto {}", self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CondJmp {
    pub negative_addr: u32,
    pub null_addr: u32,
    pub positive_addr: u32,
}

impl Operation for CondJmp {
    const ID: usize = next_id![Goto];
    const SIZE: usize = 13;
    const DISPLAY_NAME: &'static str = "cond_branch";

    fn decode(input: &[u8]) -> Result<(Self, &[u8])> {
        let (negative_addr, tail) =
            pump_four(input).context("Failed to get negative branch address")?;
        let (null_addr, tail) = pump_four(tail).context("Failed to get null branch address")?;
        let (positive_addr, tail) =
            pump_four(tail).context("Failed to get positive branch address")?;

        let instr = CondJmp {
            negative_addr,
            null_addr,
            positive_addr,
        };

        Ok((instr, tail))
    }
}

impl Display for CondJmp {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(
            f,
            "cond_jmp {}, {}, {}",
            self.negative_addr, self.negative_addr, self.positive_addr
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Neg;

impl Operation for Neg {
    const ID: usize = next_id![CondJmp];
    const SIZE: usize = 1;
    const DISPLAY_NAME: &'static str = "neg";

    fn decode(input: &[u8]) -> Result<(Self, &[u8])> {
        let instr = Neg;

        Ok((instr, input))
    }
}

impl Display for Neg {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "neg")
    }
}

pub(crate) fn pump_one(input: &[u8]) -> Result<(u8, &[u8])> {
    match input {
        [fst, rest @ ..] => Ok((*fst, rest)),
        _ => {
            Err(anyhow!(DecodingError::UnexpectedEof)).context("Failed to get one byte from input")
        }
    }
}

fn pump_two(input: &[u8]) -> Result<(u16, &[u8])> {
    match input {
        [fst, snd, rest @ ..] => {
            let val = u16::from_be_bytes([*fst, *snd]);
            Ok((val, rest))
        }
        _ => {
            Err(anyhow!(DecodingError::UnexpectedEof)).context("Failed to get two bytes from input")
        }
    }
}

fn pump_four(input: &[u8]) -> Result<(u32, &[u8])> {
    match input {
        [fst, snd, trd, fth, rest @ ..] => {
            let val = u32::from_be_bytes([*fst, *snd, *trd, *fth]);
            Ok((val, rest))
        }
        _ => Err(anyhow!(DecodingError::UnexpectedEof))
            .context("Failed to get four bytes from input"),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DecodingError {
    UnknownOpcode(u8),
    UnexpectedEof,
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            DecodingError::UnexpectedEof => write!(f, "Unexpected EOF"),
            DecodingError::UnknownOpcode(id) => write!(f, "Unknown opcode: `{}`", id),
        }
    }
}

impl Error for DecodingError {}

#[cfg(test)]
mod id_tests {
    use super::*;

    macro_rules! assert_correct_id {
        ($ty:ident) => {
            assert_eq!(
                AVAILABLE_DECODERS[$ty::ID] as usize,
                $ty::decode_and_wrap as usize
            );
        };
    }

    #[test]
    fn decoder_id_matches_operation_id() {
        assert_correct_id!(PushI);
        assert_correct_id!(AddI);
        assert_correct_id!(FStop);
        assert_correct_id!(PushCopy);
        assert_correct_id!(Call);
        assert_correct_id!(Ret);
        assert_correct_id!(ResV);
        assert_correct_id!(PopCopy);
        assert_correct_id!(Goto);
        assert_correct_id!(CondJmp);
        assert_correct_id!(Neg);
    }
}
