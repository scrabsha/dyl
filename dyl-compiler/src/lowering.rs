use crate::{
    ast::{
        Addition, Binding, Bindings, Bool, ExprKind, Function, Ident, If, Integer, Multiplication,
        Program, Subtraction,
    },
    context::{CompilerPassError, LoweringContext},
    instruction::Instruction,
};

pub(crate) fn lower_ast(
    ast: &Program,
    mut ctxt: LoweringContext,
) -> Result<(LoweringContext, Vec<Instruction>), CompilerPassError> {
    let mut tmp = Vec::new();

    let lowering_rslt = ast.lower(&mut tmp, &mut ctxt).map(|()| tmp);

    ctxt.wrap_result(lowering_rslt)
}

trait Lowerable {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut LoweringContext)
        -> LoweringResult;
}

type LoweringResult = Result<(), ()>;

impl Lowerable for Program {
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
        let main_fn_data = self
            .functions()
            .iter()
            .enumerate()
            .find(|(_, f)| f.name() == "main");

        if main_fn_data.is_none() {
            ctxt.errors().add("No `main` function found");
        }

        // We want to lower the main function first, so that the instructions
        // to be executed start at offset 0.
        //
        // In the future, we may prefer starting with a `push_i 0; call main;`
        // directive at the beginning of the bytecode. This should be done next.

        // We don't perform early return because we want to catch as much
        // lowering errors as possible.
        let main_fn_lowering = main_fn_data
            .ok_or(())
            .and_then(|(_, node)| node.lower(collector, ctxt));

        if main_fn_lowering.is_ok() {
            // We must remove the final `pop_copy` and `ret` instruction, as
            // the main function does not return the way other functions do.
            collector.truncate(collector.len() - 2);

            // For the same reason, we must add the full stop instruction.
            let full_stop = Instruction::f_stop();
            collector.push(full_stop);
        }

        let idx_to_avoid = main_fn_data.map(|(idx, _)| idx);

        let rslt = self
            .functions()
            .iter()
            .enumerate()
            .filter_map(|(idx, function)| {
                if Some(idx) != idx_to_avoid {
                    Some(function)
                } else {
                    None
                }
            })
            .map(|function| function.lower(collector, ctxt))
            .fold(Ok(()), Result::and)
            .and(main_fn_lowering);

        rslt
    }
}

impl Lowerable for Function {
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
        ctxt.labels_mut()
            .new_named(self.name().to_string(), collector.len() as u32);

        self.body().lower(collector, ctxt)?;

        let rslt_copy_instr = Instruction::pop_copy(1);
        let return_instr = Instruction::ret();

        collector.extend([rslt_copy_instr, return_instr]);

        Ok(())
    }
}

impl Lowerable for ExprKind {
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
        match self {
            ExprKind::Addition(e) => e.lower(collector, ctxt),
            ExprKind::Integer(e) => e.lower(collector, ctxt),
            ExprKind::Subtraction(e) => e.lower(collector, ctxt),
            ExprKind::If(e) => e.lower(collector, ctxt),
            ExprKind::Multiplication(e) => e.lower(collector, ctxt),
            ExprKind::Bindings(e) => e.lower(collector, ctxt),
            ExprKind::Ident(e) => e.lower(collector, ctxt),
            ExprKind::Bool(e) => e.lower(collector, ctxt),
        }
    }
}

impl Lowerable for Integer {
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
        let instr = Instruction::push_i(self.value());
        collector.push(instr);
        ctxt.stack_mut().push_anonymous();

        Ok(())
    }
}

impl Lowerable for Addition {
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
        let left_exp = self.left().lower(collector, ctxt);
        let right_exp = self.right().lower(collector, ctxt);

        let instr = Instruction::add_i();
        collector.push(instr);
        ctxt.stack_mut().pop_top_anonymous().unwrap();

        left_exp.and(right_exp)
    }
}

impl Lowerable for Subtraction {
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
        let left_exp = self.left().lower(collector, ctxt);
        let right_exp = self.right().lower(collector, ctxt);

        let instructions = [Instruction::neg(), Instruction::add_i()];

        collector.extend_from_slice(&instructions);
        ctxt.stack_mut().pop_top_anonymous().unwrap();

        left_exp.and(right_exp)
    }
}

impl Lowerable for Multiplication {
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
        let left_exp = self.left().lower(collector, ctxt);
        let right_exp = self.right().lower(collector, ctxt);
        collector.push(Instruction::mul());
        ctxt.stack_mut().pop_top_anonymous().unwrap();

        left_exp.and(right_exp)
    }
}

impl Lowerable for If {
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
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
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
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
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
        let value_exp = self.value().lower(collector, ctxt);
        ctxt.stack_mut()
            .name_top_anonymous(self.name().to_owned())
            .unwrap();

        value_exp
    }
}

impl Lowerable for Ident {
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
        let stack_offset = match ctxt.stack().resolve(self.name()) {
            Some(offset) => offset,
            None => {
                ctxt.errors()
                    .add(format!("Undefined variable `{}`", self.name()));

                ctxt.stack_mut().push_anonymous();

                return Err(());
            }
        };

        collector.push(Instruction::push_copy(stack_offset));

        ctxt.stack_mut().push_anonymous();

        Ok(())
    }
}

impl Lowerable for Bool {
    fn lower(
        &self,
        collector: &mut Vec<Instruction>,
        ctxt: &mut LoweringContext,
    ) -> LoweringResult {
        let num = if self.value() { 1 } else { 0 };

        collector.push(Instruction::push_i(num));
        ctxt.stack_mut().push_anonymous();

        Ok(())
    }
}

#[cfg(test)]
fn lower(expr: &impl Lowerable) -> (Vec<Instruction>, LoweringContext) {
    let mut collector = Vec::new();
    let mut ctxt = LoweringContext::new();

    if expr.lower(&mut collector, &mut ctxt).is_err() {
        println!("{}", ctxt.errors());
        panic!("called `Result::unwrap()` on an `Err` value: ()")
    }

    (collector, ctxt)
}

#[cfg(test)]
#[test]
fn lowering_can_fail() {
    let ast = ExprKind::ident("undefined".to_owned());
    let mut collector = Vec::new();
    let mut ctxt = LoweringContext::new();

    assert!(ast.lower(&mut collector, &mut ctxt).is_err());
}

#[cfg(test)]
mod program {
    use crate::inline_program;

    use super::*;

    #[test]
    fn simplest_test() {
        let program: Program = inline_program! { fn main() { 42 } };
        let (instrs, _) = lower(&program);

        assert_eq!(instrs, [Instruction::push_i(42), Instruction::f_stop()]);
    }

    #[test]
    fn main_is_lowered_first() {
        let program: Program = inline_program! {
            fn ___() { 41 }
            fn main() { 42 }
        };

        let (instrs, _) = lower(&program);

        assert!(instrs.starts_with(&[Instruction::push_i(42), Instruction::f_stop()]));
    }

    #[test]
    fn main_instrs_are_removed() {
        let program: Program = inline_program! { fn main() { 42 } };
        let (instrs, _) = lower(&program);

        assert!(!instrs.ends_with(&[Instruction::ret()]));
    }
}

#[cfg(test)]
mod function {
    use crate::inline_fn;

    use super::*;

    #[test]
    fn body_is_lowered() {
        let f: Function = inline_fn! { fn f() { 42 } };
        let (instrs, _) = lower(&f);

        assert_eq!(
            instrs,
            [
                Instruction::push_i(42),
                Instruction::pop_copy(1),
                Instruction::ret()
            ]
        );
    }

    #[test]
    fn label_is_added() {
        let f: Function = inline_fn! { fn foo() { 42 } };
        let (_, ctxt) = lower(&f);

        assert!(ctxt.labels().resolve_named("foo").is_ok());
    }
}

#[cfg(test)]
mod integer {
    use crate::inline_expr;

    use super::*;

    #[test]
    fn lower_42() {
        let expr: ExprKind = inline_expr! { 42 };
        let (left, _) = lower(&expr);

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
        let (left, _) = lower(&simple_addition());

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
        let (_, ctxt) = lower(&simple_addition());

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
        let (left, _) = lower(&simple_multiplication());

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
        let (_, ctxt) = lower(&simple_multiplication());

        assert_eq!(ctxt.stack().depth(), 1);
        assert!(ctxt.stack().top().unwrap().is_empty());
    }
}

#[cfg(test)]
mod subtraction {
    use crate::inline_expr;

    use super::*;

    fn simple_subtraction() -> ExprKind {
        inline_expr! { 43 - 1 }
    }

    #[test]
    fn generated_instructions() {
        let (left, _) = lower(&simple_subtraction());

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
        let (_, ctxt) = lower(&simple_subtraction());

        assert_eq!(ctxt.stack().depth(), 1);
        assert!(ctxt.stack().top().unwrap().is_empty());
    }
}

#[cfg(test)]
mod if_ {
    use crate::inline_expr;

    use super::*;

    fn simple_if() -> ExprKind {
        inline_expr! { if 1 { 42 } else { -1 } }
    }

    #[test]
    fn generated_instructions() {
        let (left, _) = lower(&simple_if());

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
        let (_, ctxt) = lower(&simple_if());

        assert_eq!(ctxt.labels().resolve_anonymous(0).unwrap(), 2);
        assert_eq!(ctxt.labels().resolve_anonymous(1).unwrap(), 4);
        assert_eq!(ctxt.labels().resolve_anonymous(2).unwrap(), 5);
    }

    #[test]
    fn stack_effects() {
        let (_, ctxt) = lower(&simple_if());

        assert_eq!(ctxt.stack().depth(), 1);
        assert!(ctxt.stack().top().unwrap().is_empty());
    }
}

#[cfg(test)]
mod bindings {
    use crate::inline_expr;

    use super::*;

    fn simple_bindings() -> ExprKind {
        inline_expr! {
            {
                let foo = 101;
                42
            }
        }
    }

    #[test]
    fn generated_instructions() {
        let (bytecode, _) = lower(&simple_bindings());

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
        let (_, ctxt) = lower(&simple_bindings());

        assert_eq!(ctxt.stack().depth(), 1);
        assert!(ctxt.stack().top().unwrap().is_empty());
    }

    #[test]
    fn recovers_from_error() {
        let expr: ExprKind = inline_expr! {
            {
                let a = b;
                let c = d;
                e
            }
        };
        let mut ctxt = LoweringContext::new();
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
        let (bytecode, _) = lower(&simple_binding());

        assert_eq!(bytecode, [Instruction::push_i(101)]);
    }

    #[test]
    fn stack_effects() {
        let (_, ctxt) = lower(&simple_binding());

        assert_eq!(ctxt.stack().depth(), 1);
        assert_eq!(ctxt.stack().top().unwrap(), "foo");
    }
}

#[cfg(test)]
mod ident {
    use crate::inline_expr;

    use super::*;

    fn simple_ident() -> ExprKind {
        inline_expr! { foo }
    }

    fn lower_simple_ident() -> (Vec<Instruction>, LoweringContext) {
        let mut ctxt = LoweringContext::new();
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
        let exp: ExprKind = inline_expr! { foo };
        let mut ctxt = LoweringContext::new();
        let mut instructions = Vec::new();

        let rslt = exp.lower(&mut instructions, &mut ctxt);

        assert!(rslt.is_err());
        assert_eq!(ctxt.stack().depth(), 1);
    }

    #[test]
    fn emits_when_not_found() {
        let expr: ExprKind = inline_expr! { undefined };
        let mut ctxt = LoweringContext::new();
        let mut instructions = Vec::new();

        let rslt = expr.lower(&mut instructions, &mut ctxt);

        assert!(rslt.is_err());
        assert_eq!(
            ctxt.errors().to_string(),
            "Undefined variable `undefined`\n"
        );
    }
}

#[cfg(test)]
mod bool {
    use crate::inline_expr;

    use super::*;

    fn simple_bool() -> ExprKind {
        inline_expr! { true }
    }

    fn lower_simple_bool() -> (Vec<Instruction>, LoweringContext) {
        let mut ctxt = LoweringContext::new();
        let mut collector = Vec::new();

        simple_bool().lower(&mut collector, &mut ctxt).unwrap();

        (collector, ctxt)
    }

    #[test]
    fn generated_instructions() {
        let (bytecode, _) = lower_simple_bool();
        assert_eq!(bytecode, [Instruction::push_i(1)]);
    }

    #[test]
    fn stack_effects() {
        let (_, ctxt) = lower_simple_bool();

        assert_eq!(ctxt.stack().depth(), 1);
        assert!(ctxt.stack().top().unwrap().is_empty());
    }
}
