use dyl_bytecode::display::disassemble;
use interpreter::Interpreter;

mod interpreter;
mod value;

fn main() {
    let ops = vec![
        // push_c a
        3, 0, 0, 0, 97,
        // fstop
        2
    ];

    disassemble(ops.as_slice()).unwrap();
    Interpreter::from_bytecode(ops).run();
}
