use dyl_bytecode::operations as resolved_operations;
use dyl_bytecode::Instruction as ResolvedInstruction;

use crate::context::{LoweringContext, Resolvable};

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Instruction {
    PushI(PushI),
    AddI(AddI),
    Mul(Mul),
    FStop(FStop),
    Neg(Neg),
    CondJmp(CondJmp),
    Goto(Goto),
    PopCopy(PopCopy),
    Pop(Pop),
    PushCopy(PushCopy),
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
            Instruction::PopCopy($name) => $do,
            Instruction::Pop($name) => $do,
            Instruction::PushCopy($name) => $do,
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

impl_from_variants! { PushI, AddI, FStop, Neg, CondJmp, Goto, Mul, PopCopy, Pop, PushCopy }

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

    pub(crate) fn pop_copy(offset: u16) -> Instruction {
        Instruction::PopCopy(PopCopy(offset))
    }

    pub(crate) fn pop(offset: u16) -> Instruction {
        Instruction::Pop(Pop(offset))
    }

    pub(crate) fn push_copy(offset: u16) -> Instruction {
        Instruction::PushCopy(PushCopy(offset))
    }
}

impl Resolvable for Instruction {
    type Output = ResolvedInstruction;

    fn resolve(&self, ctxt: &LoweringContext) -> Self::Output {
        map_instruction!(self, |instruction| instruction.resolve(ctxt).into())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct PushI(pub i32);

impl Resolvable for PushI {
    type Output = resolved_operations::PushI;

    fn resolve(&self, _ctxt: &LoweringContext) -> Self::Output {
        resolved_operations::PushI(self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct AddI;

impl Resolvable for AddI {
    type Output = resolved_operations::AddI;

    fn resolve(&self, _ctxt: &LoweringContext) -> Self::Output {
        resolved_operations::AddI
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Mul;

impl Resolvable for Mul {
    type Output = resolved_operations::Mul;

    fn resolve(&self, _ctxt: &LoweringContext) -> Self::Output {
        resolved_operations::Mul
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct FStop;

impl Resolvable for FStop {
    type Output = resolved_operations::FStop;

    fn resolve(&self, _ctxt: &LoweringContext) -> Self::Output {
        resolved_operations::FStop
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Neg;

impl Resolvable for Neg {
    type Output = resolved_operations::Neg;

    fn resolve(&self, _ctxt: &LoweringContext) -> Self::Output {
        resolved_operations::Neg
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct CondJmp(pub u32, pub u32, pub u32);

impl Resolvable for CondJmp {
    type Output = resolved_operations::CondJmp;

    fn resolve(&self, ctxt: &LoweringContext) -> Self::Output {
        let CondJmp(neg, null, pos) = *self;

        let negative_addr = ctxt
            .labels()
            .resolve(neg)
            .expect("Failed to resolve negative address value");

        let null_addr = ctxt
            .labels()
            .resolve(null)
            .expect("Failed to resolve null address value");

        let positive_addr = ctxt
            .labels()
            .resolve(pos)
            .expect("Failed to resolve positive address value");

        resolved_operations::CondJmp {
            negative_addr,
            null_addr,
            positive_addr,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Goto(pub u32);

impl Resolvable for Goto {
    type Output = resolved_operations::Goto;

    fn resolve(&self, ctxt: &LoweringContext) -> Self::Output {
        let dest = ctxt
            .labels()
            .resolve(self.0)
            .expect("Failed to resolve goto destination");

        resolved_operations::Goto(dest)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct PopCopy(pub u16);

impl Resolvable for PopCopy {
    type Output = resolved_operations::PopCopy;

    fn resolve(&self, _ctxt: &LoweringContext) -> Self::Output {
        resolved_operations::PopCopy(self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Pop(pub u16);

impl Resolvable for Pop {
    type Output = resolved_operations::Pop;

    fn resolve(&self, _ctxt: &LoweringContext) -> Self::Output {
        resolved_operations::Pop(self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct PushCopy(pub u16);

impl Resolvable for PushCopy {
    type Output = resolved_operations::PushCopy;

    fn resolve(&self, _ctxt: &LoweringContext) -> Self::Output {
        resolved_operations::PushCopy(self.0)
    }
}
