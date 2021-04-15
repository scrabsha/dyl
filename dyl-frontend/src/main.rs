use anyhow::Result;

fn main() -> Result<()> {
    let bytecode = dyl_compiler::bytecode_from_program("main.dyl")?;

    dyl_vm::run_program(bytecode)?;

    Ok(())
}
