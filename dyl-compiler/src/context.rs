use anyhow::{anyhow, bail, Result};

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
    labels: Vec<Option<u32>>,
}

impl Context {
    pub(crate) fn new() -> Context {
        Context { labels: Vec::new() }
    }

    pub(crate) fn new_anonymous_label(&mut self) -> u32 {
        let tmp = self.labels.len();
        self.labels.push(None);
        tmp as u32
    }

    pub(crate) fn set_label_position(&mut self, label_id: u32, label_pos: u32) -> Result<()> {
        match self.labels.get_mut(label_id as usize) {
            Some(val @ None) => *val = Some(label_pos),
            Some(Some(previous_label)) => {
                bail!(DuplicateLabelPosition(label_id, *previous_label, label_pos))
            }
            _ => bail!(UnresolvedLabel(label_id)),
        }

        Ok(())
    }

    pub(crate) fn resolve(&self, label_id: u32) -> Result<u32> {
        self.labels
            .get(label_id as usize)
            .copied()
            .flatten()
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

#[cfg(test)]
mod anonymous_labels {
    use super::*;

    #[test]
    fn grow_continuously() {
        let mut ctxt = Context::new();
        let (a, b, c) = (
            ctxt.new_anonymous_label(),
            ctxt.new_anonymous_label(),
            ctxt.new_anonymous_label(),
        );

        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(c, 2);
    }
}

#[cfg(test)]
mod labels {
    use super::*;

    #[test]
    fn set_label_position_when_undefined() {
        let mut ctxt = Context::new();
        let a = ctxt.new_anonymous_label();
        assert!(ctxt.set_label_position(a, 101).is_ok());
    }

    #[test]
    fn set_label_position_when_already_defined() {
        let mut ctxt = Context::new();
        let a = ctxt.new_anonymous_label();
        ctxt.set_label_position(a, 101).unwrap();

        assert!(ctxt.set_label_position(a, 13).is_err());
    }

    #[test]
    fn resolve_anonymous_defined() {
        let mut ctxt = Context::new();
        let a = ctxt.new_anonymous_label();
        ctxt.set_label_position(a, 42);

        assert_eq!(ctxt.resolve(a).unwrap(), 42);
    }

    #[test]
    fn resolve_anonymous_undefined() {
        let mut ctxt = Context::new();
        let a = ctxt.new_anonymous_label();

        assert!(ctxt.resolve(a).is_err());
    }
}
