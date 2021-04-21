use anyhow::{Context as AnyContext, Result};

use crate::context::{Context, Resolvable};
use dyl_bytecode::operations as resolved_operations;
use dyl_bytecode::Instruction as ResolvedInstruction;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Instruction {
    PushI(PushI),
    AddI(AddI),
    Mul(Mul),
    FStop(FStop),
    Neg(Neg),
    CondJmp(CondJmp),
    Goto(Goto),
}

macro_rules! map_instruction {
    ($instruction:ident, |$name:ident| $do:expr) => {
        match $instruction {
            Instruction::PushI($name) => $do,
            Instruction::AddI($name) => $do,
            Instruction::FStop($name) => $do,
            Instruction::Neg($name) => $do,
            Instruction::CondJmp($name) => $do,
            Instruction::Goto($name) => $do,
            Instruction::Mul($name) => $do,
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

impl_from_variants! { PushI, AddI, FStop, Neg, CondJmp, Goto, Mul }

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

    pub(crate) fn cond_jmp(negative: u32, null: u32, positive: u32) -> Instruction {
        Instruction::CondJmp(CondJmp(negative, null, positive))
    }

    pub(crate) fn goto(addr: u32) -> Instruction {
        Instruction::Goto(Goto(addr))
    }

    pub(crate) fn mul() -> Instruction {
        Instruction::Mul(Mul)
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
pub(crate) struct Mul;

impl Resolvable for Mul {
    type Output = resolved_operations::Mul;

    fn resolve(&self, _ctxt: &Context) -> Result<Self::Output> {
        Ok(resolved_operations::Mul)
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct CondJmp(pub u32, pub u32, pub u32);

impl Resolvable for CondJmp {
    type Output = resolved_operations::CondJmp;

    fn resolve(&self, ctxt: &Context) -> Result<Self::Output> {
        let CondJmp(neg, null, pos) = *self;

        let negative_addr = ctxt
            .resolve(neg)
            .with_context(|| format!("Failed to resolve negative address value of id `{}`", neg))?;

        let null_addr = ctxt
            .resolve(null)
            .with_context(|| format!("Failed to resolve null address value of id `{}`", null))?;

        let positive_addr = ctxt
            .resolve(pos)
            .with_context(|| format!("Failed to resolve positive address value of id `{}`", pos))?;

        Ok(resolved_operations::CondJmp {
            negative_addr,
            null_addr,
            positive_addr,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Goto(pub u32);

impl Resolvable for Goto {
    type Output = resolved_operations::Goto;

    fn resolve(&self, ctxt: &Context) -> Result<Self::Output> {
        let dest = ctxt
            .resolve(self.0)
            .context("Failed to get goto destination")?;

        Ok(resolved_operations::Goto(dest))
    }
}
