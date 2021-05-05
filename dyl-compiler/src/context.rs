use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};

use anyhow::Result;

use dyl_bytecode::Instruction as ResolvedInstruction;

use crate::instruction::Instruction;

pub(crate) fn resolve_context(
    instructions: &[Instruction],
    ctxt: &Context,
) -> Result<Vec<ResolvedInstruction>> {
    instructions.iter().map(|i| i.resolve(ctxt)).collect()
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Context {
    labels: LabelContext,
    stack: StackContext,
}

impl Context {
    pub(crate) fn new() -> Context {
        Context::default()
    }

    pub(crate) fn labels(&self) -> &LabelContext {
        &self.labels
    }

    pub(crate) fn labels_mut(&mut self) -> &mut LabelContext {
        &mut self.labels
    }

    pub(crate) fn stack(&self) -> &StackContext {
        &self.stack
    }

    pub(crate) fn stack_mut(&mut self) -> &mut StackContext {
        &mut self.stack
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LabelContext(Vec<Option<u32>>);

impl LabelContext {
    pub(crate) fn new_anonymous(&mut self) -> u32 {
        let tmp = self.0.len();
        self.0.push(None);
        tmp as u32
    }

    pub(crate) fn set_position(
        &mut self,
        label_id: u32,
        pos: u32,
    ) -> Result<(), LabelDefinitionError> {
        match self.0.get_mut(label_id as usize) {
            Some(slot @ Option::None) => {
                *slot = Some(pos);
                Ok(())
            }
            None => Err(LabelDefinitionError::UnknownLabel),
            Some(Some(addr)) => Err(LabelDefinitionError::AlreadyDefined(*addr)),
        }
    }

    pub(crate) fn resolve(&self, label_id: u32) -> Result<u32, LabelResolutionError> {
        self.0
            .get(label_id as usize)
            .ok_or(LabelResolutionError::UnknownLabel)?
            .ok_or(LabelResolutionError::UnknownLabelPosition)
    }

    #[cfg(test)]
    fn new() -> LabelContext {
        LabelContext::default()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum LabelDefinitionError {
    AlreadyDefined(u32),
    UnknownLabel,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum LabelResolutionError {
    UnknownLabel,
    UnknownLabelPosition,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StackContext(Vec<String>);

impl StackContext {
    pub(crate) fn push_anonymous(&mut self) {
        self.0.push(String::new());
    }

    pub(crate) fn name_top_anonymous(&mut self, name: String) -> Result<(), AnonymousNamingError> {
        let top = self
            .0
            .last_mut()
            .ok_or(AnonymousNamingError::NoTopVariable)?;

        if top.is_empty() {
            *top = name;
            Ok(())
        } else {
            Err(AnonymousNamingError::NotAnonymous)
        }
    }

    pub(crate) fn resolve(&self, name: &str) -> Option<u16> {
        self.0
            .iter()
            .rev()
            .enumerate()
            .find_map(|(depth, var_name)| var_name.eq(name).then(|| depth as u16))
    }

    pub(crate) fn new_subcontext(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn drop_subcontext(&mut self, new_top: usize) {
        self.0.truncate(new_top);
    }

    pub(crate) fn pop_top_anonymous(&mut self) -> Result<(), AnonymousPoppingError> {
        self.0
            .pop()
            .ok_or(AnonymousPoppingError::EmptyStack)?
            .is_empty()
            .then(|| ())
            .ok_or(AnonymousPoppingError::NotAnonymous)
    }

    #[cfg(test)]
    fn new() -> StackContext {
        StackContext::default()
    }

    #[cfg(test)]
    fn push_variable(&mut self, name: String) {
        self.0.push(name)
    }

    #[cfg(test)]
    pub(crate) fn depth(&self) -> usize {
        self.0.len()
    }

    #[cfg(test)]
    pub(crate) fn top(&self) -> Option<&str> {
        self.0.last().map(AsRef::as_ref)
    }

    #[cfg(test)]
    pub(crate) fn push_named(&mut self, name: String) {
        self.0.push(name)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum AnonymousNamingError {
    NoTopVariable,
    NotAnonymous,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum AnonymousPoppingError {
    EmptyStack,
    NotAnonymous,
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
        let mut ctxt = LabelContext::new();
        let (a, b, c) = (
            ctxt.new_anonymous(),
            ctxt.new_anonymous(),
            ctxt.new_anonymous(),
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
        let mut ctxt = LabelContext::new();
        let a = ctxt.new_anonymous();
        assert!(ctxt.set_position(a, 101).is_ok());
    }

    #[test]
    fn set_label_position_when_already_defined() {
        let mut ctxt = LabelContext::new();
        let a = ctxt.new_anonymous();
        ctxt.set_position(a, 101).unwrap();

        assert_eq!(
            ctxt.set_position(a, 13),
            Err(LabelDefinitionError::AlreadyDefined(101))
        );
    }

    #[test]
    fn resolve_anonymous_defined() {
        let mut ctxt = LabelContext::new();
        let a = ctxt.new_anonymous();
        ctxt.set_position(a, 42).unwrap();

        assert_eq!(ctxt.resolve(a), Ok(42));
    }

    #[test]
    fn resolve_anonymous_undefined() {
        let mut ctxt = LabelContext::new();
        let a = ctxt.new_anonymous();

        assert_eq!(
            ctxt.resolve(a),
            Err(LabelResolutionError::UnknownLabelPosition)
        );
    }
}

#[cfg(test)]
mod variables {
    use super::*;

    #[test]
    fn define_and_resolve_on_top() {
        let mut ctxt = StackContext::new();
        ctxt.push_variable("foo".to_owned());

        assert_eq!(ctxt.resolve("foo"), Some(0));
    }

    #[test]
    fn define_and_resolve_not_top() {
        let mut ctxt = StackContext::new();
        ctxt.push_variable("foo".to_owned());
        ctxt.push_variable("bar".to_owned());

        assert_eq!(ctxt.resolve("foo"), Some(1));
    }

    #[test]
    fn shadowing_simple() {
        let mut ctxt = StackContext::new();
        ctxt.push_variable("foo".to_owned());
        ctxt.push_variable("foo".to_owned());

        assert_eq!(ctxt.resolve("foo"), Some(0));
    }

    #[test]
    fn cross_sub_context_shadowing() {
        let mut ctxt = StackContext::new();
        ctxt.push_variable("foo".to_owned());
        ctxt.push_variable("bar".to_owned());

        let outer = ctxt.new_subcontext();

        ctxt.push_variable("foo".to_owned());
        assert_eq!(ctxt.resolve("foo"), Some(0));

        ctxt.drop_subcontext(outer);
        assert_eq!(ctxt.resolve("foo"), Some(1));
    }

    #[test]
    fn push_anonymous_increases_depth() {
        let mut ctxt = StackContext::new();
        ctxt.push_variable("foo".to_owned());
        ctxt.push_anonymous();

        assert_eq!(ctxt.resolve("foo"), Some(1));
    }

    #[test]
    fn drop_top_anonymous_decreases_depth() {
        let mut ctxt = StackContext::new();
        ctxt.push_variable("foo".to_owned());
        ctxt.push_anonymous();

        ctxt.pop_top_anonymous().unwrap();

        assert_eq!(ctxt.resolve("foo"), Some(0));
    }

    #[test]
    fn drop_top_anonymous_fails_when_not_anonymous() {
        let mut ctxt = StackContext::new();
        ctxt.push_variable("foo".to_owned());

        assert_eq!(
            ctxt.pop_top_anonymous(),
            Err(AnonymousPoppingError::NotAnonymous)
        );
    }

    #[test]
    fn pop_top_anonymous_fails_when_empty() {
        let mut ctxt = StackContext::new();

        assert_eq!(
            ctxt.pop_top_anonymous(),
            Err(AnonymousPoppingError::EmptyStack)
        );
    }

    #[test]
    fn name_top_anonymous_working() {
        let mut ctxt = StackContext::new();
        ctxt.push_anonymous();
        ctxt.name_top_anonymous("foo".to_owned()).unwrap();

        assert_eq!(ctxt.resolve("foo"), Some(0));
    }

    #[test]
    fn name_top_anonymous_empty_stack() {
        let mut ctxt = StackContext::new();
        assert_eq!(
            ctxt.name_top_anonymous("foo".to_owned()),
            Err(AnonymousNamingError::NoTopVariable)
        );
    }

    #[test]
    fn name_top_anonymous_already_named() {
        let mut ctxt = StackContext::new();
        ctxt.push_variable("foo".to_owned());

        assert_eq!(
            ctxt.name_top_anonymous("bar".to_owned()),
            Err(AnonymousNamingError::NotAnonymous)
        );
    }
}
