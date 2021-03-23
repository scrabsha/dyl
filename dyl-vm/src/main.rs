use anyhow::Result;

use dyl_bytecode::display::disassemble;
use interpreter::Interpreter;

mod interpreter;
mod value;

fn main() -> Result<()> {
    #[rustfmt::skip]
    let ops = vec![
        // Call 6
        5, 0, 0, 0, 6,

        // fstop
        2,

        // (at offset 6) push_i 42
        0, 0, 0, 0, 42,

        // ret 0 1
        6, 0, 0, 0, 0, 0, 0, 0, 1
    ];

    disassemble(ops.as_slice()).unwrap();

    let exit_value = Interpreter::from_bytecode(ops).run()?;

    println!("Program exited with value `{}`", exit_value);

    Ok(())
}
