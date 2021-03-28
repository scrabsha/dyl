use anyhow::{anyhow, bail, ensure, Context, Result};

use dyl_bytecode::Instruction;

use crate::{runnable::RunStatus, value::Value};
use crate::runnable::Runnable;

pub(crate) struct Interpreter {
    stack: Stack,
    code: Vec<Instruction>,
}

impl Interpreter {
    pub(crate) fn from_bytecode(code: &[u8]) -> Result<Interpreter> {
        let code = Instruction::from_bytes(code).context("Failed to decode input")?;

        Ok(Interpreter::from_instructions(code))
    }

    pub(crate) fn from_instructions(code: Vec<Instruction>) -> Interpreter {
        let stack = Stack::new();
        Interpreter { stack, code }
    }

    pub(crate) fn run(&mut self) -> Result<Value> {
        let mut ip = 0;
        
        let final_value = loop {
            match self.run_single(ip)? {
                RunStatus::ContinueToNext => ip += 1,
                RunStatus::ContinueTo(idx) => ip = idx,
                RunStatus::Stop(val) => break val,
            }
        };

        Ok(final_value)
    }

    fn run_single(&mut self, ip: u32) -> Result<RunStatus> {
        let instr = self
            .code
            .get(ip as usize)
            .ok_or_else(|| anyhow!("Failed to read instruction at index `{}`", ip))?
            .clone();

        instr.run(ip, &mut self.stack)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Stack(Vec<Value>);

impl Stack {
    fn new() -> Stack {
        Stack(Vec::new())
    }

    pub(crate) fn push_integer(&mut self, n: i32) {
        let v = Value::Integer(n);
        self.push_value(v);
    }

    pub(crate) fn pop_integer(&mut self) -> Result<i32> {
        self.pop()
            .and_then(Value::try_into_integer)
            .context("Failed to pop an integer from the stack")
    }

    pub(crate) fn pop(&mut self) -> Result<Value> {
        self.0.pop().ok_or_else(|| anyhow!("Empty stack found"))
    }

    pub(crate) fn push_value(&mut self, v: Value) {
        self.0.push(v);
    }

    pub(crate) fn full_stop_value(&self) -> Result<&Value> {
        match self.0.as_slice() {
            [unique_value] => Ok(unique_value),
            [] => bail!("Found empty stack at the end of the program"),
            _ => bail!("Expected single-element in the stack at the end of the program"),
        }
    }

    pub(crate) fn copy_value(&mut self, idx: u16) -> Result<()> {
        ensure!(!self.0.is_empty(), "Out-of-bound stack access");

        let idx = self.0.len() - 1 - idx as usize;
        let value = self
            .0
            .get(idx)
            .ok_or_else(|| anyhow!("Out-of-bound stack access"))?
            .clone();

        self.0.push(value);

        Ok(())
    }

    pub(crate) fn push_instruction_pointer(&mut self, idx: u32) {
        let value = Value::InstructionPointer(idx);
        self.0.push(value);
    }

    pub(crate) fn get_instruction_pointer_at_offset(&mut self, idx: u16) -> Result<u32> {
        self.get_at_offset(idx)?
            .clone()
            .try_into_instruction_pointer()
    }

    pub(crate) fn get_at_offset(&mut self, idx: u16) -> Result<&Value> {
        ensure!(!self.0.is_empty(), "Out-of-bound stack access");

        let idx = self.0.len() - 1 - idx as usize;

        self.0
            .get(idx)
            .ok_or_else(|| anyhow!("Out-of-bound stack access"))
    }

    pub(crate) fn truncate(&mut self, idx: u16) -> Result<()> {
        ensure!(!self.0.is_empty(), "Out-of-bound stack access");

        let idx = self.0.len() - idx as usize;
        self.0.truncate(idx);

        Ok(())
    }

    pub(crate) fn replace(&mut self, offset: u16, val: Value) -> Result<()> {
        let idx = self.0.len() - offset as usize;

        let dest = self
            .0
            .get_mut(idx)
            .ok_or_else(|| anyhow!("Out-of-bound stack access"))?;

        *dest = val;

        Ok(())
    }
}
