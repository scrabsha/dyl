use anyhow::{Context, Result};

use dyl_bytecode::{
    operations::{AddI, Call, CondJmp, FStop, Goto, Neg, PopCopy, PushCopy, PushI, ResV, Ret},
    Instruction,
};

use crate::{interpreter::Stack, value::Value};

use std::cmp::Ordering;

pub(crate) trait Runnable {
    fn run(&self, ip: u32, s: &mut Stack) -> Result<RunStatus>;
}

impl Runnable for Instruction {
    fn run(&self, ip: u32, s: &mut Stack) -> Result<RunStatus> {
        match self {
            Instruction::PushI(op) => op.run(ip, s).context("Failed to run `push_i` instruction"),
            Instruction::AddI(op) => op.run(ip, s).context("Failed to run `add_i` instruction"),
            Instruction::FStop(op) => op.run(ip, s).context("Failed to run `f_stop`"),
            Instruction::PushCopy(op) => op
                .run(ip, s)
                .context("Failed to run `push_copy` instruction"),
            Instruction::Call(op) => op.run(ip, s).context("Failed to run `call` instruction"),
            Instruction::Ret(op) => op.run(ip, s).context("Failed to run `ret` instruction"),
            Instruction::ResV(op) => op.run(ip, s).context("Failed to run `res_v` instruction"),
            Instruction::PopCopy(op) => op
                .run(ip, s)
                .context("Failed to run `pop_copy` instruction"),
            Instruction::Goto(op) => op.run(ip, s).context("Failed to run `goto` instruction"),
            Instruction::CondJmp(op) => op
                .run(ip, s)
                .context("Failed to run `cond_jmp` instruction"),
            Instruction::Neg(op) => op.run(ip, s).context("Failed to run `neg` instruction"),
        }
    }
}

impl Runnable for PushI {
    fn run(&self, _ip: u32, s: &mut Stack) -> Result<RunStatus> {
        let n = self.0;
        s.push_integer(n);

        Ok(RunStatus::ContinueToNext)
    }
}

impl Runnable for AddI {
    fn run(&self, _ip: u32, s: &mut Stack) -> Result<RunStatus> {
        let lhs = s
            .pop_integer()
            .context("Failed to get integer left-hand-side-value")?;
        let rhs = s
            .pop_integer()
            .context("Failed to get integer right-hand-side value")?;

        s.push_integer(lhs + rhs);

        Ok(RunStatus::ContinueToNext)
    }
}

impl Runnable for FStop {
    fn run(&self, _ip: u32, s: &mut Stack) -> Result<RunStatus> {
        let final_value = s.full_stop_value()?.clone();

        Ok(RunStatus::Stop(final_value))
    }
}

impl Runnable for PushCopy {
    fn run(&self, _ip: u32, s: &mut Stack) -> Result<RunStatus> {
        let idx = self.0;
        s.copy_value(idx)?;

        Ok(RunStatus::ContinueToNext)
    }
}

impl Runnable for Call {
    fn run(&self, ip: u32, s: &mut Stack) -> Result<RunStatus> {
        s.push_instruction_pointer(ip + 1);
        Ok(RunStatus::ContinueTo(self.0))
    }
}

impl Runnable for Ret {
    fn run(&self, _ip: u32, s: &mut Stack) -> Result<RunStatus> {
        let initial_offset = s
            .get_instruction_pointer_at_offset(self.ip_offset)
            .context("Failed to get return address")?;

        s.truncate(self.shrink_offset)
            .context("Failed to resize stack")?;

        Ok(RunStatus::ContinueTo(initial_offset))
    }
}

impl Runnable for ResV {
    fn run(&self, _ip: u32, s: &mut Stack) -> Result<RunStatus> {
        let ResV(offset) = self;

        for _ in 0..*offset {
            s.push_integer(0);
        }

        Ok(RunStatus::ContinueToNext)
    }
}

impl Runnable for PopCopy {
    fn run(&self, _ip: u32, s: &mut Stack) -> Result<RunStatus> {
        let PopCopy(offset) = self;

        let v = s.pop().context("Failed to get value to copy")?;

        s.replace(*offset, v)
            .context("Failed to replace stack value")?;

        Ok(RunStatus::ContinueToNext)
    }
}

impl Runnable for Goto {
    fn run(&self, _ip: u32, _s: &mut Stack) -> Result<RunStatus> {
        let dest = self.0;
        Ok(RunStatus::ContinueTo(dest))
    }
}

impl Runnable for CondJmp {
    fn run(&self, _ip: u32, s: &mut Stack) -> Result<RunStatus> {
        let i = s
            .pop_integer()
            .context("Failed to get conditional jump offset")?;

        Ok(match i.cmp(&0) {
            Ordering::Less => RunStatus::ContinueTo(self.negative_addr),
            Ordering::Equal => RunStatus::ContinueTo(self.null_addr),
            Ordering::Greater => RunStatus::ContinueTo(self.positive_addr),
        })
    }
}

impl Runnable for Neg {
    fn run(&self, _ip: u32, s: &mut Stack) -> Result<RunStatus> {
        let i = s.pop_integer().context("Failed to get integer to negate")?;
        s.push_integer(-i);

        Ok(RunStatus::ContinueToNext)
    }
}

pub(crate) enum RunStatus {
    ContinueToNext,
    ContinueTo(u32),
    Stop(Value),
}
