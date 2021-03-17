use interpreter::Interpreter;

mod interpreter;
mod value;

fn main() {
    let ops = vec![0, 0, 0, 0, 41, 0, 0, 0, 0, 1, 1, 2];
    Interpreter::from_bytecode(ops).run();
}
