use anyhow::{anyhow, bail, ensure, Context, Result};

use dyl_bytecode::Instruction;

use crate::value::Value;

pub(crate) struct Interpreter {
    stack: Stack,
    code: Vec<u8>,
    state: InterpreterState,
}

impl Interpreter {
    pub(crate) fn from_bytecode(code: Vec<u8>) -> Interpreter {
        let stack = Stack::new();
        let state = InterpreterState::beginning();

        Interpreter { stack, state, code }
    }

    pub(crate) fn run(&mut self) -> Result<Value> {
        while self.state.is_running() {
            self.run_single()?;
        }

        let final_value = self.state.finished_value().unwrap().clone();

        Ok(final_value)
    }

    fn run_single(&mut self) -> Result<()> {
        let ip = self
            .state
            .instruction_pointer()
            .expect("attempt to call run_single on a finished interpreter");

        let code_tail = self.code.split_at(ip).1;
        let (instr, len) = Instruction::decode(code_tail)
            .with_context(|| format!("Failed to decode instruction at offset {:#x}", ip))?
            .0;

        self.state.increment_instruction_pointer(len)?;

        match instr {
            Instruction::AddI => self.run_add_i()?,
            Instruction::PushI(val) => self.run_push_i(val),
            Instruction::FullStop => self.run_full_stop()?,
            Instruction::PushC(chr) => self.run_push_c(chr),
            Instruction::CopyV(idx) => self.run_copy_v(idx)?,
        }

        Ok(())
    }

    fn run_add_i(&mut self) -> Result<()> {
        let (lhs, rhs) = self.pop_pair_i().context("Failed to run `add_i`")?;

        let sum = lhs + rhs;
        self.stack.push_integer(sum);

        Ok(())
    }

    fn pop_pair_i(&mut self) -> Result<(i32, i32)> {
        let lhs = self
            .stack
            .pop_integer()
            .context("Failed to get left-hand-side integer")?;

        let rhs = self
            .stack
            .pop_integer()
            .context("Failed to get right-hand-side integer")?;

        Ok((lhs, rhs))
    }

    fn run_push_i(&mut self, i: i32) {
        self.stack.push_integer(i);
    }

    fn run_full_stop(&mut self) -> Result<()> {
        let v = self.stack.full_stop_value()?;
        self.state = InterpreterState::Finished(v);

        Ok(())
    }

    fn run_push_c(&mut self, chr: char) {
        self.stack.push_char(chr);
    }

    fn run_copy_v(&mut self, idx: u32) -> Result<()> {
        self.stack.copy_value(idx)
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
        self.pop()
            .and_then(Value::try_into_integer)
            .context("Failed to pop an integer from the stack")
    }

    fn pop_char(&mut self) -> Result<char> {
        self.pop()
            .and_then(Value::try_into_char)
            .context("Failed to pop a char form the stack")
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

    fn copy_value(&mut self, idx: u32) -> Result<()> {
        ensure!(!self.0.is_empty(), "Out-of-bound stack access");

        let idx = self.0.len() - 1 - idx as usize;
        let value_to_copy = self
            .0
            .get(idx)
            .ok_or_else(|| anyhow!("Out-of-bound stack access"))?
            .clone();

        self.0.push(value_to_copy);

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
enum InterpreterState {
    Running(usize),
    Finished(Value),
}

impl InterpreterState {
    fn beginning() -> InterpreterState {
        InterpreterState::Running(0)
    }

    fn is_running(&self) -> bool {
        matches!(self, InterpreterState::Running(_))
    }

    fn finished_value(&self) -> Option<&Value> {
        match self {
            InterpreterState::Finished(v) => Some(v),
            _ => None,
        }
    }

    fn increment_instruction_pointer(&mut self, idx: usize) -> Result<()> {
        match self {
            InterpreterState::Running(ip) => *ip += idx,
            InterpreterState::Finished(_) => {
                bail!("Attempt to increment instruction pointer on a finished interpreter")
            }
        };

        Ok(())
    }

    fn instruction_pointer(&self) -> Option<usize> {
        match self {
            InterpreterState::Running(ip) => Some(*ip),
            InterpreterState::Finished(_) => None,
        }
    }
}
