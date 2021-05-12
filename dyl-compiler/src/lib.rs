use std::path::Path;

use anyhow::{Context, Result};

use dyl_bytecode::Instruction;

mod ast;
mod context;
mod instruction;
mod io;
mod lowering;
mod parser;

pub fn compile<PA, PB>(i: PA, o: PB) -> Result<()>
where
    PA: AsRef<Path>,
    PB: AsRef<Path>,
{
    let content = io::read_program(i.as_ref())
        .with_context(|| format!("Failed to read input file `{}`", i.as_ref().display()))?;

    let (ctxt, ast) = parser::parse_input(content.as_str())?;

    let ctxt = ctxt.into_lowering_context();

    let (ctxt, instructions) = lowering::lower_ast(&ast, ctxt)?;

    let instructions = context::resolve_context(instructions.as_slice(), &ctxt);

    let output = Instruction::encode_multiple(&instructions);

    io::write_bytecode(o, output.as_slice()).context("Failed to write output bytecode")?;

    Ok(())
}

pub fn bytecode_from_program<P>(path: P) -> Result<Vec<Instruction>>
where
    P: AsRef<Path>,
{
    let content = io::read_program(path.as_ref())
        .with_context(|| format!("Failed to read input file `{}`", path.as_ref().display()))?;

    let (ctxt, ast) = parser::parse_input(content.as_str())?;

    let ctxt = ctxt.into_lowering_context();

    let (ctxt, instructions) = lowering::lower_ast(&ast, ctxt)?;

    let final_instructions = context::resolve_context(instructions.as_slice(), &ctxt);

    Ok(final_instructions)
}
