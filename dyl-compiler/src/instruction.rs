use anyhow::Result;

use crate::context::{Context, Resolvable};
use dyl_bytecode::operations as resolved_operations;
use dyl_bytecode::Instruction as ResolvedInstruction;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Instruction {
    PushI(PushI),
    AddI(AddI),
    FStop(FStop),
    Neg(Neg),
}

macro_rules! map_instruction {
    ($instruction:ident, |$name:ident| $do:expr) => {
        match $instruction {
            Instruction::PushI($name) => $do,
            Instruction::AddI($name) => $do,
            Instruction::FStop($name) => $do,
            Instruction::Neg($name) => $do,
        }
    };
}

macro_rules! impl_from_variants {
    ($( $variant:ident ),* $(,)? ) => {
        $(
            impl From<$variant> for Instruction {
                fn from(input: $variant) -> Instruction {
                    Instruction::$variant(input)
                }
            }
        )*
    };
}

impl_from_variants! { PushI, AddI, FStop, Neg }

impl Instruction {
    pub(crate) fn push_i(i: i32) -> Instruction {
        Instruction::PushI(PushI(i))
    }

    pub(crate) fn add_i() -> Instruction {
        Instruction::AddI(AddI)
    }

    pub(crate) fn f_stop() -> Instruction {
        Instruction::FStop(FStop)
    }

    pub(crate) fn neg() -> Instruction {
        Instruction::Neg(Neg)
    }
}

impl Resolvable for Instruction {
    type Output = ResolvedInstruction;

    fn resolve(&self, ctxt: &Context) -> Result<Self::Output> {
        map_instruction!(self, |instruction| instruction
            .resolve(ctxt)
            .map(Into::into))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct PushI(pub i32);

impl Resolvable for PushI {
    type Output = resolved_operations::PushI;

    fn resolve(&self, _ctxt: &Context) -> Result<Self::Output> {
        Ok(resolved_operations::PushI(self.0))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct AddI;

impl Resolvable for AddI {
    type Output = resolved_operations::AddI;

    fn resolve(&self, _ctxt: &Context) -> Result<Self::Output> {
        Ok(resolved_operations::AddI)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct FStop;

impl Resolvable for FStop {
    type Output = resolved_operations::FStop;

    fn resolve(&self, _ctxt: &Context) -> Result<Self::Output> {
        Ok(resolved_operations::FStop)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Neg;

impl Resolvable for Neg {
    type Output = resolved_operations::Neg;

    fn resolve(&self, _ctxt: &Context) -> Result<Self::Output> {
        Ok(resolved_operations::Neg)
    }
}
