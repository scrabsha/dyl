use anyhow::Result;

use dyl_bytecode::display::disassemble;
use dyl_bytecode::Instruction;
use interpreter::Interpreter;

mod interpreter;
mod runnable;
mod value;

#[cfg(test)]
mod tests;

pub fn run_program(bytecode: Vec<Instruction>) -> Result<()> {
    let return_value = Interpreter::from_instructions(bytecode).run()?;
    println!("{}", return_value);

    Ok(())
}
