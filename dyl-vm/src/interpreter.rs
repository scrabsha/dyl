use anyhow::{anyhow, bail, ensure, Context, Result};

use dyl_bytecode::Instruction;

use crate::value::Value;

pub(crate) struct Interpreter {
    stack: Stack,
    code: Vec<Instruction>,
    state: InterpreterState,
}

impl Interpreter {
    pub(crate) fn from_bytecode(code: &[u8]) -> Result<Interpreter> {
        let stack = Stack::new();
        let state = InterpreterState::beginning();
        let code = Instruction::from_bytes(code).context("Failed to decode input")?;

        Ok(Interpreter { stack, state, code })
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

        let instr = self
            .code
            .get(ip)
            .ok_or_else(|| anyhow!("Failed to read instruction at index `{}`", ip))?
            .clone();

        self.state.increment_instruction_pointer()?;

        match instr {
            Instruction::AddI => self.run_add_i().context("Failed to run `add_i`")?,
            Instruction::PushI(val) => self.run_push_i(val),
            Instruction::FullStop => self.run_full_stop().context("Failed to run `f_stop`")?,
            Instruction::PushC(chr) => self.run_push_c(chr),
            Instruction::CopyV(idx) => self.run_copy_v(idx).context("Failed to run `copy_v`")?,
            Instruction::Call(idx) => self.run_call(idx),
            Instruction::Ret {
                return_offset,
                pointer_offset,
            } => self
                .run_ret(return_offset, pointer_offset)
                .context("Failed to run `ret`")?,
            Instruction::RetW {
                pointer_offset,
                value_offset,
            } => self
                .run_ret_w(pointer_offset, value_offset)
                .context("Failed to run `ret_w`")?,
            Instruction::ResV(offset) => self.run_res_v(offset),
            Instruction::CopyVS(offset) => self
                .run_copy_v_s(offset)
                .context("Failed to run `copy_v_s`")?,
        }

        Ok(())
    }

    fn run_add_i(&mut self) -> Result<()> {
        let (lhs, rhs) = self.pop_pair_i()?;

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
        self.state = InterpreterState::Finished(v.clone());

        Ok(())
    }

    fn run_push_c(&mut self, chr: char) {
        self.stack.push_char(chr);
    }

    fn run_copy_v(&mut self, idx: u32) -> Result<()> {
        self.stack.copy_value(idx).context("Failed to run `copy_v`")
    }

    fn run_call(&mut self, idx: u32) {
        let ip = self
            .state
            .instruction_pointer()
            .expect("run_call called on a finished interpreter");

        self.stack.push_instruction_pointer(ip as u32);
        self.state.replace_instruction_pointer(idx);
    }

    fn run_ret_w(&mut self, pointer_offset: u32, value_offset: u32) -> Result<()> {
        let initial_offset = self
            .stack
            .get_instruction_pointer_at_offset(pointer_offset)
            .context("Failed to get return address")?;

        let value_to_replace = self
            .stack
            .get_at_offset(value_offset)
            .context("Failed to get return value")?
            .clone();

        self.stack
            .replace_instruction_pointer(pointer_offset, value_to_replace)
            .context("Failed to replace stack pointer")?;

        self.stack
            .truncate(pointer_offset)
            .context("Failed to resize stack")?;

        self.state.replace_instruction_pointer(initial_offset);

        Ok(())
    }

    fn run_ret(&mut self, return_offset: u32, pointer_offset: u32) -> Result<()> {
        let initial_offset = self
            .stack
            .get_instruction_pointer_at_offset(pointer_offset)
            .context("Failed to get return address")?;

        self.stack
            .truncate(return_offset)
            .context("Failed to resize stack")?;

        self.state.replace_instruction_pointer(initial_offset);

        Ok(())
    }

    fn run_res_v(&mut self, offset: u32) {
        for _ in 0..offset {
            self.stack.push_integer(0);
        }
    }

    fn run_copy_v_s(&mut self, offset: u32) -> Result<()> {
        let v = self.stack.pop().context("Failed to get value to copy")?;

        self.stack
            .replace(offset, v)
            .context("Failed to replace stack value")?;

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

    fn full_stop_value(&self) -> Result<&Value> {
        match self.0.as_slice() {
            [unique_value] => Ok(unique_value),
            [] => bail!("Found empty stack at the end of the program"),
            _ => bail!("Expected single-element in the stack at the end of the program"),
        }
    }

    fn copy_value(&mut self, idx: u32) -> Result<()> {
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

    fn push_instruction_pointer(&mut self, idx: u32) {
        let value = Value::InstructionPointer(idx);
        self.0.push(value);
    }

    fn get_instruction_pointer_at_offset(&mut self, idx: u32) -> Result<u32> {
        self.get_at_offset(idx)?
            .clone()
            .try_into_instruction_pointer()
    }

    fn get_at_offset(&mut self, idx: u32) -> Result<&Value> {
        ensure!(!self.0.is_empty(), "Out-of-bound stack access");

        let idx = self.0.len() - 1 - idx as usize;

        self.0
            .get(idx)
            .ok_or_else(|| anyhow!("Out-of-bound stack access"))
    }

    fn replace_instruction_pointer(&mut self, idx: u32, v: Value) -> Result<()> {
        ensure!(!self.0.is_empty(), "Out-of-bound stack access");

        let idx = self.0.len() - 1 - idx as usize;

        let value_to_replace = self
            .0
            .get_mut(idx)
            .ok_or_else(|| anyhow!("Out-of-bound stack access"))?;

        ensure!(
            value_to_replace.is_instruction_pointer(),
            "Instruction pointer not found"
        );

        *value_to_replace = v;

        Ok(())
    }

    fn truncate(&mut self, idx: u32) -> Result<()> {
        ensure!(!self.0.is_empty(), "Out-of-bound stack access");

        let idx = self.0.len() - idx as usize;
        self.0.truncate(idx);

        Ok(())
    }

    fn replace(&mut self, offset: u32, val: Value) -> Result<()> {
        let idx = self.0.len() - offset as usize;

        let dest = self
            .0
            .get_mut(idx)
            .ok_or_else(|| anyhow!("Out-of-bound stack access"))?;

        *dest = val;

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

    fn increment_instruction_pointer(&mut self) -> Result<()> {
        match self {
            InterpreterState::Running(ip) => *ip += 1,
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

    fn replace_instruction_pointer(&mut self, idx: u32) {
        *self = InterpreterState::Running(idx as usize);
    }
}
