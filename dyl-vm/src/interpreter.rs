use dyl_bytecode::Instruction;

use crate::value::Value;

pub(crate) struct Interpreter {
    stack: Stack,
    code: Vec<u8>,
    ip: usize,
}

impl Interpreter {
    pub(crate) fn from_bytecode(code: Vec<u8>) -> Interpreter {
        let stack = Stack::new();
        let ip = 0;

        Interpreter { stack, ip, code }
    }

    pub(crate) fn run(&mut self) {
        while self.ip != usize::MAX {
            self.run_single();
        }
    }

    fn run_single(&mut self) {
        let code_tail = self.code.split_at(self.ip).1;
        let (instr, len) = Instruction::decode(code_tail).unwrap().0;

        self.ip += len;

        match instr {
            Instruction::AddI => self.run_add_i(),
            Instruction::PushI(val) => self.run_push_i(val),
            Instruction::FullStop => self.run_full_stop(),
            Instruction::PushC(chr) => self.run_push_c(chr),
        }
    }

    fn run_add_i(&mut self) {
        let lhs = self.stack.pop_integer().unwrap();
        let rhs = self.stack.pop_integer().unwrap();

        let sum = lhs + rhs;
        self.stack.push_integer(sum);
    }

    fn run_push_i(&mut self, i: i32) {
        self.stack.push_integer(i);
    }

    fn run_full_stop(&mut self) {
        self.ip = usize::MAX;

        if let Err(msg) = self.stack.full_stop_value() {
            eprintln!("{}", msg)
        }
    }

    fn run_push_c(&mut self, chr: char) {
        self.stack.push_char(chr);
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Stack(Vec<Value>);

impl Stack {
    fn new() -> Stack {
        Stack(Vec::new())
    }

    fn push_integer(&mut self, n: i32) {
        let v = Value::Integer(n);
        self.push_value(v);
    }

    fn push_char(&mut self, c: char) {
        let v = Value::Char(c);
        self.push_value(v);
    }

    fn pop_integer(&mut self) -> Option<i32> {
        let top = self.0.pop()?.try_into_integer();
        match top {
            Ok(n) => Some(n),
            Err(v) => {
                self.push_value(v);
                None
            }
        }
    }

    fn pop_char(&mut self) -> Option<char> {
        let top = self.0.pop()?.try_into_char();
        match top {
            Ok(c) => Some(c),
            Err(v) => {
                self.push_value(v);
                None
            }
        }
    }

    fn push_value(&mut self, v: Value) {
        self.0.push(v);
    }

    fn full_stop_value(&self) -> Result<Value, &'static str> {
        match self.0.as_slice() {
            [unique_value] => Ok(unique_value.clone()),
            [] => Err("Program exited with empty stack"),
            _ => Err("Program exited with non-unique stack value"),
        }
    }
}