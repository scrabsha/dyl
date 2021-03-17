use std::convert::TryInto;

use dyl_bytecode::Instruction;

use crate::value::Value;

pub(crate) struct Interpreter {
    stack: Vec<Value>,
    code: Vec<u8>,
    ip: usize,
}

impl Interpreter {
    pub(crate) fn from_bytecode(code: Vec<u8>) -> Interpreter {
        let stack = Vec::new();
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
        }
    }

    fn run_add_i(&mut self) {
        let lhs = self.pop_stack().unwrap().try_into_integer().unwrap();
        let rhs = self.pop_stack().unwrap().try_into_integer().unwrap();

        let rslt = Value::Integer(lhs + rhs);
        self.stack.push(rslt);
    }

    fn run_push_i(&mut self, val: i32) {
        let i = Value::Integer(val);
        self.stack.push(i);
    }

    fn run_full_stop(&mut self) {
        let v = self.pop_stack().unwrap();
        println!("Final value: {:?}", v);
        self.ip = usize::MAX;
    }

    fn pop_stack(&mut self) -> Option<Value> {
        self.stack.pop()
    }
}
