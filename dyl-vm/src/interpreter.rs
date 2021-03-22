use anyhow::{anyhow, bail, Context, Result};

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

    pub(crate) fn run(&mut self) -> Result<()> {
        while self.ip != usize::MAX {
            self.run_single()?;
        }

        Ok(())
    }

    fn run_single(&mut self) -> Result<()> {
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

    fn run_add_i(&mut self) -> Result<()> {
        let lhs = self.stack.pop_integer()?;
        let rhs = self.stack.pop_integer()?;

        let sum = lhs + rhs;
        self.stack.push_integer(sum);

        Ok(())
    }

    fn run_push_i(&mut self, i: i32) -> Result<()> {
        self.stack.push_integer(i);
        Ok(())
    }

    fn run_full_stop(&mut self) -> Result<()> {
        self.ip = usize::MAX;

        let v = self.stack.full_stop_value()?;
        println!("Program exited with `{}` result", v);
        Ok(())
    }

    fn run_push_c(&mut self, chr: char) -> Result<()> {
        self.stack.push_char(chr);
        Ok(())
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

    fn pop_integer(&mut self) -> Result<i32> {
        self.pop()?
            .try_into_integer()
            .context("While pop-ing from stack")
    }

    fn pop_char(&mut self) -> Result<char> {
        self.pop()?
            .try_into_char()
            .context("While pop-ing from stack")
    }

    fn pop(&mut self) -> Result<Value> {
        self.0.pop().ok_or_else(|| anyhow!("Empty stack found"))
    }

    fn push_value(&mut self, v: Value) {
        self.0.push(v);
    }

    fn full_stop_value(&self) -> Result<Value> {
        match self.0.as_slice() {
            [unique_value] => Ok(unique_value.clone()),
            [] => bail!("Found empty stack at the end of the program"),
            _ => bail!("Expected single-element in the stack at the end of the program"),
        }
    }
}
