use std::{
    cell::RefCell,
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

use dyl_bytecode::Instruction as ResolvedInstruction;

use crate::instruction::Instruction;

pub(crate) fn resolve_context(
    instructions: &[Instruction],
    ctxt: &LoweringContext,
) -> Vec<ResolvedInstruction> {
    instructions.iter().map(|i| i.resolve(ctxt)).collect()
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ParsingContext {
    errs: ErrorContext,
}

impl ParsingContext {
    pub(crate) fn new() -> ParsingContext {
        ParsingContext::default()
    }

    pub(crate) fn errors(&self) -> &ErrorContext {
        &self.errs
    }

    pub(crate) fn into_lowering_context(self) -> LoweringContext {
        let errs = self.errs;

        LoweringContext {
            errs,
            ..Default::default()
        }
    }

    pub(crate) fn wrap_result<T>(self, rslt: Result<T, ()>) -> PassResult<ParsingContext, T> {
        self.errs
            .emit_possible_errors(rslt)
            .map(|pass_value| (self, pass_value))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LoweringContext {
    labels: LabelContext,
    stack: StackContext,
    errs: ErrorContext,
}

impl LoweringContext {
    pub(crate) fn new() -> LoweringContext {
        LoweringContext::default()
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

    pub(crate) fn errors(&self) -> &ErrorContext {
        &self.errs
    }

    pub(crate) fn wrap_result<T>(self, res: Result<T, ()>) -> PassResult<LoweringContext, T> {
        self.errs
            .emit_possible_errors(res)
            .map(|pass_value| (self, pass_value))
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

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ErrorContext(RefCell<Vec<CompilationError>>);

impl ErrorContext {
    pub(crate) fn add(&self, e: impl Into<CompilationError>) {
        self.0.borrow_mut().push(e.into());
    }

    fn emit_possible_errors<T>(&self, rslt: Result<T, ()>) -> Result<T, CompilerPassError> {
        let errs = self.0.borrow();

        match (rslt, errs.as_slice()) {
            (Ok(v), []) => Ok(v),
            _ => {
                self.emit();
                Err(CompilerPassError(errs.len()))
            }
        }
    }

    fn emit(&self) {
        eprintln!("{}", self);
    }

    #[cfg(test)]
    fn new() -> ErrorContext {
        ErrorContext::default()
    }
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0
            .borrow()
            .iter()
            .try_for_each(|e| writeln!(f, "{}", e))
    }
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub(crate) struct CompilerPassError(usize);

impl Display for CompilerPassError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let error_word = if self.0 == 1 { "error" } else { "errors" };

        write!(f, "Compilation failed with {} {}", self.0, error_word)
    }
}

impl Error for CompilerPassError {}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CompilationError(String);

impl From<String> for CompilationError {
    fn from(input: String) -> Self {
        CompilationError(input)
    }
}

impl From<&str> for CompilationError {
    fn from(input: &str) -> Self {
        let input = input.to_owned();
        CompilationError::from(input)
    }
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}

pub(crate) trait Resolvable {
    type Output;

    fn resolve(&self, ctxt: &LoweringContext) -> Self::Output;
}

pub(crate) type PassResult<C, T> = Result<(C, T), CompilerPassError>;

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

#[cfg(test)]
mod errors {
    use super::*;

    #[test]
    fn creation_addition_display() {
        let errs = ErrorContext::new();
        errs.add("Hello");
        errs.add("World");

        let left = errs.to_string();
        let right = "Hello\nWorld\n";

        assert_eq!(left, right);
    }
}

#[cfg(test)]
mod compiler_pass_error {
    use super::*;

    #[test]
    fn singular() {
        assert_eq!(
            CompilerPassError(1).to_string(),
            "Compilation failed with 1 error"
        );
    }

    #[test]
    fn plural() {
        assert_eq!(
            CompilerPassError(2).to_string(),
            "Compilation failed with 2 errors"
        );
    }
}
