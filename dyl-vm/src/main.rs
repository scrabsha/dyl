use anyhow::Result;

use dyl_bytecode::display::disassemble;
use interpreter::Interpreter;

mod interpreter;
mod value;

fn main() -> Result<()> {
    #[rustfmt::skip]
    let ops = vec![
        // res_v 1
        8, 0, 0, 0, 1,

        // Call 11
        5, 0, 0, 0, 11,

        // fstop
        2,

        // (at offset 11) push_i 42
        0, 0, 0, 0, 42,
        
        // copy_s_v 1
        9, 0, 0, 0, 2,

        // ret 1 0
        7, 0, 0, 0, 1, 0, 0, 0, 0,
    ];

    println!("Program disassembly:");
    disassemble(ops.as_slice()).unwrap();

    println!("Program execution:");
    let exit_value = Interpreter::from_bytecode(ops).run()?;

    println!("Program exited with value `{}`", exit_value);

    Ok(())
}
