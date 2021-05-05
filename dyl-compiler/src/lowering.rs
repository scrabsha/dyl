use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use crate::{
    ast::{Addition, Binding, Bindings, ExprKind, Ident, If, Integer, Multiplication, Subtraction},
    context::Context,
    instruction::Instruction,
};

pub(crate) fn lower_ast(ast: &ExprKind) -> (Vec<Instruction>, Context, Result<(), LoweringError>) {
    let mut tmp = Vec::new();
    let mut ctxt = Context::new();

    let lowering_status = ast.lower(&mut tmp, &mut ctxt).map_err(|_| LoweringError);

    tmp.push(Instruction::f_stop());

    (tmp, ctxt, lowering_status)
}

trait Lowerable {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) -> LoweringResult;
}

type LoweringResult = Result<(), LoweringError>;

impl Lowerable for ExprKind {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) -> LoweringResult {
        match self {
            ExprKind::Addition(e) => e.lower(collector, ctxt),
            ExprKind::Integer(e) => e.lower(collector, ctxt),
            ExprKind::Subtraction(e) => e.lower(collector, ctxt),
            ExprKind::If(e) => e.lower(collector, ctxt),
            ExprKind::Multiplication(e) => e.lower(collector, ctxt),
            ExprKind::Bindings(e) => e.lower(collector, ctxt),
            ExprKind::Ident(e) => e.lower(collector, ctxt),
        }
    }
}

impl Lowerable for Integer {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) -> LoweringResult {
        let instr = Instruction::push_i(self.value());
        collector.push(instr);
        ctxt.stack_mut().push_anonymous();

        Ok(())
    }
}

impl Lowerable for Addition {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) -> LoweringResult {
        let left_exp = self.left().lower(collector, ctxt);
        let right_exp = self.right().lower(collector, ctxt);

        let instr = Instruction::add_i();
        collector.push(instr);
        ctxt.stack_mut().pop_top_anonymous().unwrap();

        left_exp.and(right_exp)
    }
}

impl Lowerable for Subtraction {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) -> LoweringResult {
        let left_exp = self.left().lower(collector, ctxt);
        let right_exp = self.right().lower(collector, ctxt);

        let instructions = [Instruction::neg(), Instruction::add_i()];

        collector.extend_from_slice(&instructions);
        ctxt.stack_mut().pop_top_anonymous().unwrap();

        left_exp.and(right_exp)
    }
}

impl Lowerable for Multiplication {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) -> LoweringResult {
        let left_exp = self.left().lower(collector, ctxt);
        let right_exp = self.right().lower(collector, ctxt);
        collector.push(Instruction::mul());
        ctxt.stack_mut().pop_top_anonymous().unwrap();

        left_exp.and(right_exp)
    }
}

impl Lowerable for If {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) -> LoweringResult {
        let condition_exp = self.condition().lower(collector, ctxt);

        let consequent_start = ctxt.labels_mut().new_anonymous();
        let alt_start = ctxt.labels_mut().new_anonymous();
        let consequent_end = ctxt.labels_mut().new_anonymous();

        let cond = Instruction::cond_jmp(consequent_start, alt_start, consequent_start);
        let goto_end = Instruction::goto(consequent_end);

        collector.push(cond);
        ctxt.stack_mut().pop_top_anonymous().unwrap();

        ctxt.labels_mut()
            .set_position(consequent_start, collector.len() as u32)
            .unwrap();

        let branches_subcontext = ctxt.stack().new_subcontext();

        let consequent_exp = self.consequent().lower(collector, ctxt);

        collector.push(goto_end);

        ctxt.stack_mut().drop_subcontext(branches_subcontext);

        ctxt.labels_mut()
            .set_position(alt_start, collector.len() as u32)
            .unwrap();

        let alternative_exp = self.alternative().lower(collector, ctxt);

        ctxt.stack_mut().drop_subcontext(branches_subcontext);
        ctxt.stack_mut().push_anonymous();

        ctxt.labels_mut()
            .set_position(consequent_end, collector.len() as u32)
            .unwrap();

        condition_exp.and(consequent_exp).and(alternative_exp)
    }
}

impl Lowerable for Bindings {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) -> LoweringResult {
        let subcontext_id = ctxt.stack().new_subcontext();
        let defines_exp = self
            .defines()
            .iter()
            .map(|b| b.lower(collector, ctxt))
            .fold(Ok(()), Result::and);

        let ending_exp = self.ending_expression().lower(collector, ctxt);

        let len = self.defines().len() as u16;

        collector.push(Instruction::pop_copy(len));
        collector.push(Instruction::pop(len - 1));

        ctxt.stack_mut().drop_subcontext(subcontext_id);
        ctxt.stack_mut().push_anonymous();

        defines_exp.and(ending_exp)
    }
}

impl Lowerable for Binding {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) -> LoweringResult {
        let value_exp = self.value().lower(collector, ctxt);
        ctxt.stack_mut()
            .name_top_anonymous(self.name().to_owned())
            .unwrap();

        value_exp
    }
}

impl Lowerable for Ident {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) -> LoweringResult {
        let stack_offset = match ctxt.stack().resolve(self.name()) {
            Some(offset) => offset,
            None => {
                ctxt.errors()
                    .add(format!("Undefined variable `{}`", self.name()));

                ctxt.stack_mut().push_anonymous();

                return Err(LoweringError);
            }
        };

        // TODO: convert stack index to u16
        collector.push(Instruction::push_copy(stack_offset as u16));

        ctxt.stack_mut().push_anonymous();

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct LoweringError;

impl Display for LoweringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Lowering failed")
    }
}

impl Error for LoweringError {}

#[cfg(test)]
fn lower_expr(expr: &impl Lowerable) -> (Vec<Instruction>, Context) {
    let mut collector = Vec::new();
    let mut ctxt = Context::new();

    expr.lower(&mut collector, &mut ctxt).unwrap();

    (collector, ctxt)
}

#[cfg(test)]
#[test]
fn lowering_can_fail() {
    let ast = ExprKind::ident("undefined".to_owned());
    let mut collector = Vec::new();
    let mut ctxt = Context::new();

    assert!(ast.lower(&mut collector, &mut ctxt).is_err());
}

#[cfg(test)]
mod integer {
    use super::*;

    #[test]
    fn lower_42() {
        let expr = Integer::new(42);
        let (left, _) = lower_expr(&expr);

        assert_eq!(left, [Instruction::push_i(42)]);
    }
}

#[cfg(test)]
mod addition {
    use super::*;

    fn simple_addition() -> ExprKind {
        ExprKind::addition(ExprKind::integer(40), ExprKind::integer(2))
    }

    #[test]
    fn generated_instructions() {
        let (left, _) = lower_expr(&simple_addition());

        assert_eq!(
            left,
            [
                Instruction::push_i(40),
                Instruction::push_i(2),
                Instruction::add_i(),
            ]
        );
    }

    #[test]
    fn stack_effects() {
        let (_, ctxt) = lower_expr(&simple_addition());

        assert_eq!(ctxt.stack().depth(), 1);
        assert!(ctxt.stack().top().unwrap().is_empty());
    }
}

#[cfg(test)]
mod multiplication {
    use super::*;

    fn simple_multiplication() -> ExprKind {
        ExprKind::multiplication(ExprKind::integer(7), ExprKind::integer(6))
    }

    #[test]
    fn generated_instructions() {
        let (left, _) = lower_expr(&simple_multiplication());

        assert_eq!(
            left,
            [
                Instruction::push_i(7),
                Instruction::push_i(6),
                Instruction::mul(),
            ]
        )
    }

    #[test]
    fn stack_effects() {
        let (_, ctxt) = lower_expr(&simple_multiplication());

        assert_eq!(ctxt.stack().depth(), 1);
        assert!(ctxt.stack().top().unwrap().is_empty());
    }
}

#[cfg(test)]
mod subtraction {
    use super::*;

    fn simple_subtraction() -> ExprKind {
        ExprKind::subtraction(ExprKind::integer(43), ExprKind::integer(1))
    }

    #[test]
    fn generated_instructions() {
        let (left, _) = lower_expr(&simple_subtraction());

        assert_eq!(
            left,
            [
                Instruction::push_i(43),
                Instruction::push_i(1),
                Instruction::neg(),
                Instruction::add_i(),
            ],
        );
    }

    #[test]
    fn stack_effects() {
        let (_, ctxt) = lower_expr(&simple_subtraction());

        assert_eq!(ctxt.stack().depth(), 1);
        assert!(ctxt.stack().top().unwrap().is_empty());
    }
}

#[cfg(test)]
mod if_ {
    use super::*;

    fn simple_if() -> ExprKind {
        ExprKind::if_(
            ExprKind::integer(1),
            ExprKind::integer(42),
            ExprKind::integer(-1),
        )
    }

    #[test]
    fn generated_instructions() {
        let (left, _) = lower_expr(&simple_if());

        assert_eq!(
            left,
            [
                Instruction::push_i(1),
                Instruction::cond_jmp(0, 1, 0),
                Instruction::push_i(42),
                Instruction::goto(2),
                Instruction::push_i(-1),
            ],
        );
    }

    #[test]
    fn label_effects() {
        let (_, ctxt) = lower_expr(&simple_if());

        assert_eq!(ctxt.labels().resolve(0).unwrap(), 2);
        assert_eq!(ctxt.labels().resolve(1).unwrap(), 4);
        assert_eq!(ctxt.labels().resolve(2).unwrap(), 5);
    }

    #[test]
    fn stack_effects() {
        let (_, ctxt) = lower_expr(&simple_if());

        assert_eq!(ctxt.stack().depth(), 1);
        assert!(ctxt.stack().top().unwrap().is_empty());
    }
}

#[cfg(test)]
mod bindings {
    use super::*;

    fn simple_bindings() -> ExprKind {
        ExprKind::single_binding(
            "foo".to_owned(),
            ExprKind::integer(101),
            ExprKind::integer(42),
        )
    }

    #[test]
    fn generated_instructions() {
        let (bytecode, _) = lower_expr(&simple_bindings());

        assert_eq!(
            bytecode,
            [
                Instruction::push_i(101),
                Instruction::push_i(42),
                Instruction::pop_copy(1),
                Instruction::pop(0),
            ]
        );
    }

    #[test]
    fn stack_effects() {
        let (_, ctxt) = lower_expr(&simple_bindings());

        assert_eq!(ctxt.stack().depth(), 1);
        assert!(ctxt.stack().top().unwrap().is_empty());
    }

    #[test]
    fn recovers_from_error() {
        let expr = ExprKind::bindings(
            vec![
                Binding::new("a".to_owned(), ExprKind::ident("b".to_owned())),
                Binding::new("c".to_owned(), ExprKind::ident("d".to_owned())),
            ],
            ExprKind::ident("e".to_owned()),
        );
        let mut ctxt = Context::new();
        let mut instructions = Vec::new();

        let rslt = expr.lower(&mut instructions, &mut ctxt);

        assert!(rslt.is_err());
        assert_eq!(
            ctxt.errors().to_string(),
            "Undefined variable `b`\nUndefined variable `d`\nUndefined variable `e`\n"
        )
    }
}

#[cfg(test)]
mod binding {
    use super::*;

    fn simple_binding() -> Binding {
        Binding::new("foo".to_owned(), ExprKind::integer(101))
    }

    #[test]
    fn generated_instructions() {
        let (bytecode, _) = lower_expr(&simple_binding());

        assert_eq!(bytecode, [Instruction::push_i(101)]);
    }

    #[test]
    fn stack_effects() {
        let (_, ctxt) = lower_expr(&simple_binding());

        assert_eq!(ctxt.stack().depth(), 1);
        assert_eq!(ctxt.stack().top().unwrap(), "foo");
    }
}

#[cfg(test)]
mod ident {
    use super::*;

    fn simple_ident() -> ExprKind {
        ExprKind::ident("foo".to_owned())
    }

    fn lower_simple_ident() -> (Vec<Instruction>, Context) {
        let mut ctxt = Context::new();
        ctxt.stack_mut().push_named("foo".to_owned());
        ctxt.stack_mut().push_named("bar".to_owned());

        let mut instructions = Vec::new();

        simple_ident().lower(&mut instructions, &mut ctxt).unwrap();

        (instructions, ctxt)
    }

    #[test]
    fn generated_instructions() {
        let (bytecode, _) = lower_simple_ident();

        assert_eq!(bytecode, [Instruction::push_copy(1)]);
    }

    #[test]
    fn stack_effects() {
        let (_, ctxt) = lower_simple_ident();

        assert_eq!(ctxt.stack().depth(), 3);
        assert!(ctxt.stack().top().unwrap().is_empty());
    }

    #[test]
    fn fails_when_not_found() {
        let exp = ExprKind::ident("foo".to_owned());
        let mut ctxt = Context::new();
        let mut instructions = Vec::new();

        let rslt = exp.lower(&mut instructions, &mut ctxt);

        assert!(rslt.is_err());
        assert_eq!(ctxt.stack().depth(), 1);
    }

    #[test]
    fn emits_when_not_found() {
        let exp = ExprKind::ident("undefined".to_owned());
        let mut ctxt = Context::new();
        let mut instructions = Vec::new();

        let rslt = exp.lower(&mut instructions, &mut ctxt);

        assert!(rslt.is_err());
        assert_eq!(
            ctxt.errors().to_string(),
            "Undefined variable `undefined`\n"
        );
    }
}
