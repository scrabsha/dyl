use anyhow::Result;

use dyl_bytecode::display::disassemble;
use interpreter::Interpreter;

mod interpreter;
mod value;

fn main() -> Result<()> {
    #[rustfmt::skip]
    let ops = vec![
        // push_c a
        3, 0, 0, 0, 97,

        // fstop
        // 2

        // add_i
        // 1
    ];

    disassemble(ops.as_slice()).unwrap();

    let exit_value = Interpreter::from_bytecode(ops).run()?;

    println!("Program exited with value `{}`", exit_value);

    Ok(())
}
