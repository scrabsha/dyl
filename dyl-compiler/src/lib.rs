use anyhow::{Context, Result};
use dyl_bytecode::Instruction;
use std::path::Path;

mod ast;
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

    let ast = parser::parse_input(content.as_str()).context("Failed to parse program")?;

    let instructions = lowering::lower_ast(&ast);
    let output = Instruction::encode_multiple(&instructions);

    io::write_bytecode(o, output.as_slice()).context("Failed to write output bytecode")?;

    Ok(())
}
