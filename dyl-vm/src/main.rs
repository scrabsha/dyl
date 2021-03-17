use dyl_bytecode::display::disassemble;
use interpreter::Interpreter;

mod interpreter;
mod value;

fn main() {
    let ops = vec![
        // pushi 40
        0, 0, 0, 0, 40,
        // pushi 1
        0, 0, 0, 0, 1,
        // pushi 1
        0, 0, 0, 0, 1,
        // addi
        1,
        // addi
        1,
        // fstop
        2
    ];

    disassemble(ops.as_slice()).unwrap();
    Interpreter::from_bytecode(ops).run();
}
