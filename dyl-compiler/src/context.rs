use anyhow::Result;

use dyl_bytecode::Instruction as ResolvedInstruction;

use crate::instruction::Instruction;

pub(crate) fn resolve_context(
    instructions: &[Instruction],
    ctxt: &Context,
) -> Result<Vec<ResolvedInstruction>> {
    instructions.into_iter().map(|i| i.resolve(ctxt)).collect()
}

pub(crate) struct Context;

impl Context {
    pub(crate) fn new() -> Context {
        Context
    }
}

pub(crate) trait Resolvable {
    type Output;

    fn resolve(&self, ctxt: &Context) -> Result<Self::Output>;
}
