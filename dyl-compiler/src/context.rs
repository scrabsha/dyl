use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};

use anyhow::{anyhow, bail, ensure, Result};

use dyl_bytecode::Instruction as ResolvedInstruction;

use crate::instruction::Instruction;

pub(crate) fn resolve_context(
    instructions: &[Instruction],
    ctxt: &Context,
) -> Result<Vec<ResolvedInstruction>> {
    instructions.iter().map(|i| i.resolve(ctxt)).collect()
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Context {
    labels: Vec<Option<u32>>,
    variables: Vec<String>,
}

impl Context {
    pub(crate) fn new() -> Context {
        Context {
            labels: Vec::new(),
            variables: Vec::new(),
        }
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

    pub(crate) fn add_anonymous_variable(&mut self) {
        self.variables.push(String::new());
    }

    pub(crate) fn name_last_anonymous(&mut self, name: String) -> Result<()> {
        let stack_top = self.variables_stack_top_mut()?;

        ensure!(
            stack_top.is_empty(),
            NonAnonymousVariable(stack_top.clone())
        );

        *stack_top = name;

        Ok(())
    }

    pub(crate) fn resolve_variable(&self, name: &str) -> Option<u32> {
        self.traverse_stack()
            .enumerate()
            .find_map(|(idx, def_name)| name.eq(def_name).then(|| idx as u32))
    }

    pub(crate) fn new_subcontext(&self) -> u32 {
        self.variables.len() as u32
    }

    pub(crate) fn drop_subcontext(&mut self, id: u32) {
        self.variables.truncate(id as usize)
    }

    pub(crate) fn drop_anonymous_variable(&mut self) -> Result<()> {
        let top = self
            .variables
            .pop()
            .ok_or_else(|| anyhow!(EmptyVariableStack))?;

        ensure!(top.is_empty(), NonAnonymousVariable(top));

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn variables_stack_top(&self) -> Result<&str> {
        self.variables
            .last()
            .map(String::as_str)
            .ok_or_else(|| anyhow!(EmptyVariableStack))
    }

    #[cfg(test)]
    pub(crate) fn add_variable(&mut self, name: String) {
        self.variables.push(name);
    }

    fn variables_stack_top_mut(&mut self) -> Result<&mut String> {
        self.variables
            .last_mut()
            .ok_or_else(|| anyhow!(EmptyVariableStack))
    }

    fn traverse_stack(&self) -> impl Iterator<Item = &str> {
        self.variables.iter().rev().map(String::as_str)
    }
}

#[cfg(test)]
impl Context {
    pub(crate) fn variables_stack_len(&self) -> usize {
        self.variables.len()
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

#[derive(Clone, Debug, PartialEq)]
struct NonAnonymousVariable(String);

impl Display for NonAnonymousVariable {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(
            f,
            "Excepted anonymous variable, found named variable `{}`",
            self.0
        )
    }
}

impl Error for NonAnonymousVariable {}

#[derive(Copy, Clone, Debug, PartialEq)]
struct EmptyVariableStack;

impl Display for EmptyVariableStack {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "Empty variable stack")
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
        ctxt.set_label_position(a, 42).unwrap();

        assert_eq!(ctxt.resolve(a).unwrap(), 42);
    }

    #[test]
    fn resolve_anonymous_undefined() {
        let mut ctxt = Context::new();
        let a = ctxt.new_anonymous_label();

        assert!(ctxt.resolve(a).is_err());
    }
}

#[cfg(test)]
mod variables {
    use super::*;

    #[test]
    fn define_and_resolve_on_top() {
        let mut ctxt = Context::new();
        ctxt.add_variable("foo".to_owned());

        assert_eq!(ctxt.resolve_variable("foo"), Some(0));
    }

    #[test]
    fn define_and_resolve_not_top() {
        let mut ctxt = Context::new();
        ctxt.add_variable("foo".to_owned());
        ctxt.add_variable("bar".to_owned());

        assert_eq!(ctxt.resolve_variable("foo"), Some(1));
    }

    #[test]
    fn shadowing_simple() {
        let mut ctxt = Context::new();
        ctxt.add_variable("foo".to_owned());
        ctxt.add_variable("foo".to_owned());

        assert_eq!(ctxt.resolve_variable("foo"), Some(0));
    }

    #[test]
    fn cross_sub_context_shadowing() {
        let mut ctxt = Context::new();
        ctxt.add_variable("foo".to_owned());
        ctxt.add_variable("bar".to_owned());

        let outer = ctxt.new_subcontext();

        ctxt.add_variable("foo".to_owned());
        assert_eq!(ctxt.resolve_variable("foo"), Some(0));

        ctxt.drop_subcontext(outer);
        assert_eq!(ctxt.resolve_variable("foo"), Some(1));
    }

    #[test]
    fn anonymous_variable_increases_depth() {
        let mut ctxt = Context::new();
        ctxt.add_variable("foo".to_owned());
        ctxt.add_anonymous_variable();

        assert_eq!(ctxt.resolve_variable("foo"), Some(1));
    }

    #[test]
    fn drop_anonymous_variable_decreases_depth() {
        let mut ctxt = Context::new();
        ctxt.add_variable("foo".to_owned());
        ctxt.add_anonymous_variable();

        ctxt.drop_anonymous_variable().unwrap();

        assert_eq!(ctxt.resolve_variable("foo"), Some(0));
    }

    #[test]
    fn drop_anonymous_variable_fails_when_not_anonymous() {
        let mut ctxt = Context::new();
        ctxt.add_variable("foo".to_owned());

        assert!(ctxt.drop_anonymous_variable().is_err());
    }

    #[test]
    fn name_last_anonymous_working() {
        let mut ctxt = Context::new();
        ctxt.add_anonymous_variable();
        ctxt.name_last_anonymous("foo".to_owned()).unwrap();

        assert_eq!(ctxt.resolve_variable("foo"), Some(0));
    }

    #[test]
    fn name_last_anonymous_empty_stack() {
        let mut ctxt = Context::new();
        assert!(ctxt.name_last_anonymous("foo".to_owned()).is_err());
    }

    #[test]
    fn name_last_anonymous_already_named() {
        let mut ctxt = Context::new();
        ctxt.add_variable("foo".to_owned());

        assert!(ctxt.name_last_anonymous("bar".to_owned()).is_err());
    }
}
