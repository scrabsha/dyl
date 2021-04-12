use anyhow::{Context, Result};

use dyl_bytecode::{
    operations::{AddI, Call, CondJmp, FStop, Goto, Neg, PopCopy, PushCopy, PushI, ResV, Ret},
    Instruction,
};

use crate::{interpreter::RunningInterpreterState, value::Value};

use std::cmp::Ordering;

pub(crate) trait Runnable {
    fn run(&self, state: RunningInterpreterState) -> Result<RunStatus>;
}

impl Runnable for Instruction {
    fn run(&self, state: RunningInterpreterState) -> Result<RunStatus> {
        match self {
            Instruction::PushI(op) => op.run(state).context("Failed to run `push_i` instruction"),
            Instruction::AddI(op) => op.run(state).context("Failed to run `add_i` instruction"),
            Instruction::FStop(op) => op.run(state).context("Failed to run `f_stop`"),
            Instruction::PushCopy(op) => op
                .run(state)
                .context("Failed to run `push_copy` instruction"),
            Instruction::Call(op) => op.run(state).context("Failed to run `call` instruction"),
            Instruction::Ret(op) => op.run(state).context("Failed to run `ret` instruction"),
            Instruction::ResV(op) => op.run(state).context("Failed to run `res_v` instruction"),
            Instruction::PopCopy(op) => op
                .run(state)
                .context("Failed to run `pop_copy` instruction"),
            Instruction::Goto(op) => op.run(state).context("Failed to run `goto` instruction"),
            Instruction::CondJmp(op) => op
                .run(state)
                .context("Failed to run `cond_jmp` instruction"),
            Instruction::Neg(op) => op.run(state).context("Failed to run `neg` instruction"),
        }
    }
}

impl Runnable for PushI {
    fn run(&self, mut state: RunningInterpreterState) -> Result<RunStatus> {
        let n = self.0;
        state.stack_mut().push_integer(n);

        Ok(state.continue_to_next().into())
    }
}

impl Runnable for AddI {
    fn run(&self, mut state: RunningInterpreterState) -> Result<RunStatus> {
        let lhs = state
            .stack_mut()
            .pop_integer()
            .context("Failed to get integer left-hand-side-value")?;
        let rhs = state
            .stack_mut()
            .pop_integer()
            .context("Failed to get integer right-hand-side value")?;

        state.stack_mut().push_integer(lhs + rhs);

        Ok(state.continue_to_next().into())
    }
}

impl Runnable for FStop {
    fn run(&self, state: RunningInterpreterState) -> Result<RunStatus> {
        let final_value = state
            .stack()
            .full_stop_value()
            .context("Failed to get final value")?
            .clone();

        Ok(RunStatus::Stop(final_value))
    }
}

impl Runnable for PushCopy {
    fn run(&self, mut state: RunningInterpreterState) -> Result<RunStatus> {
        let idx = self.0;
        state.stack_mut().copy_value(idx)?;

        Ok(state.continue_to_next().into())
    }
}

impl Runnable for Call {
    fn run(&self, mut state: RunningInterpreterState) -> Result<RunStatus> {
        let jump_addr = self.0;
        let next_addr = state.ip() + 1;
        state.stack_mut().push_instruction_pointer(next_addr);
        Ok(state.continue_to(jump_addr).into())
    }
}

impl Runnable for Ret {
    fn run(&self, mut state: RunningInterpreterState) -> Result<RunStatus> {
        let initial_offset = state
            .stack_mut()
            .get_instruction_pointer_at_offset(self.ip_offset)
            .context("Failed to get return address")?;

        state
            .stack_mut()
            .truncate(self.shrink_offset)
            .context("Failed to resize stack")?;

        Ok(state.continue_to(initial_offset).into())
    }
}

impl Runnable for ResV {
    fn run(&self, mut state: RunningInterpreterState) -> Result<RunStatus> {
        let ResV(offset) = self;

        for _ in 0..*offset {
            state.stack_mut().push_integer(0);
        }

        Ok(state.continue_to_next().into())
    }
}

impl Runnable for PopCopy {
    fn run(&self, mut state: RunningInterpreterState) -> Result<RunStatus> {
        let PopCopy(offset) = self;

        let v = state
            .stack_mut()
            .pop()
            .context("Failed to get value to copy")?;

        state
            .stack_mut()
            .replace(*offset, v)
            .context("Failed to replace stack value")?;

        Ok(state.continue_to_next().into())
    }
}

impl Runnable for Goto {
    fn run(&self, state: RunningInterpreterState) -> Result<RunStatus> {
        let dest = self.0;
        Ok(state.continue_to(dest).into())
    }
}

impl Runnable for CondJmp {
    fn run(&self, mut state: RunningInterpreterState) -> Result<RunStatus> {
        let i = state
            .stack_mut()
            .pop_integer()
            .context("Failed to get conditional jump offset")?;

        Ok(match i.cmp(&0) {
            Ordering::Less => state.continue_to(self.negative_addr),
            Ordering::Equal => state.continue_to(self.null_addr),
            Ordering::Greater => state.continue_to(self.positive_addr),
        }
        .into())
    }
}

impl Runnable for Neg {
    fn run(&self, mut state: RunningInterpreterState) -> Result<RunStatus> {
        let i = state
            .stack_mut()
            .pop_integer()
            .context("Failed to get integer to negate")?;
        state.stack_mut().push_integer(-i);

        Ok(state.continue_to_next().into())
    }
}

pub(crate) enum RunStatus {
    Continue(RunningInterpreterState),
    Stop(Value),
}

impl From<RunningInterpreterState> for RunStatus {
    fn from(state: RunningInterpreterState) -> RunStatus {
        RunStatus::Continue(state)
    }
}
