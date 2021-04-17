use anyhow::{anyhow, Result};

use dyl_bytecode::Instruction as ResolvedInstruction;

use crate::instruction::Instruction;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};

pub(crate) fn resolve_context(
    instructions: &[Instruction],
    ctxt: &Context,
) -> Result<Vec<ResolvedInstruction>> {
    instructions.into_iter().map(|i| i.resolve(ctxt)).collect()
}

pub(crate) struct Context {
    next_id: u32,
    labels: HashMap<u32, u32>,
}

impl Context {
    pub(crate) fn new() -> Context {
        Context {
            next_id: 0,
            labels: HashMap::new(),
        }
    }

    pub(crate) fn new_anonymous_label(&mut self) -> u32 {
        let tmp = self.next_id;
        self.next_id += 1;
        tmp
    }

    pub(crate) fn set_label_position(&mut self, label_id: u32, label_pos: u32) -> Result<()> {
        if let Some(previous_label) = self.labels.insert(label_id, label_pos) {
            Err(anyhow!(DuplicateLabelPosition(
                label_id,
                previous_label,
                label_pos
            )))
        } else {
            Ok(())
        }
    }

    pub(crate) fn resolve(&self, label_id: u32) -> Result<u32> {
        self.labels
            .get(&label_id)
            .copied()
            .ok_or_else(|| anyhow!(UnresolvedLabel(label_id)))
    }
}

pub(crate) trait Resolvable {
    type Output;

    fn resolve(&self, ctxt: &Context) -> Result<Self::Output>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct UnresolvedLabel(u32);

impl Display for UnresolvedLabel {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "Label `{}` is used but never declared", self.0)
    }
}

impl Error for UnresolvedLabel {}

#[derive(Copy, Clone, Debug, PartialEq)]
struct DuplicateLabelPosition(u32, u32, u32);

impl Display for DuplicateLabelPosition {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(
            f,
            "Label `{}` is positioned at `{}` and at `{}`",
            self.0, self.1, self.2
        )
    }
}
