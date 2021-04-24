use crate::{
    ast::{Addition, Binding, Bindings, ExprKind, Ident, If, Integer, Multiplication, Subtraction},
    context::Context,
    instruction::Instruction,
};

pub(crate) fn lower_ast(ast: &ExprKind) -> (Vec<Instruction>, Context) {
    let mut tmp = Vec::new();
    let mut ctxt = Context::new();

    ast.lower(&mut tmp, &mut ctxt);

    tmp.push(Instruction::f_stop());

    (tmp, ctxt)
}

trait Lowerable {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context);
}

impl Lowerable for ExprKind {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
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
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
        let instr = Instruction::push_i(self.value());
        collector.push(instr);
        ctxt.add_anonymous_variable();
    }
}

impl Lowerable for Addition {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
        self.left().lower(collector, ctxt);
        self.right().lower(collector, ctxt);

        let instr = Instruction::add_i();
        collector.push(instr);
        ctxt.drop_anonymous_variable().unwrap();
    }
}

impl Lowerable for Subtraction {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
        self.left().lower(collector, ctxt);
        self.right().lower(collector, ctxt);

        let instructions = [Instruction::neg(), Instruction::add_i()];

        collector.extend_from_slice(&instructions);
        ctxt.drop_anonymous_variable().unwrap();
    }
}

impl Lowerable for Multiplication {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
        self.left().lower(collector, ctxt);
        self.right().lower(collector, ctxt);
        collector.push(Instruction::mul());
        ctxt.drop_anonymous_variable().unwrap();
    }
}

impl Lowerable for If {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
        self.condition().lower(collector, ctxt);

        let consequent_start = ctxt.new_anonymous_label();
        let alt_start = ctxt.new_anonymous_label();
        let consequent_end = ctxt.new_anonymous_label();

        let cond = Instruction::cond_jmp(consequent_start, alt_start, consequent_start);
        let goto_end = Instruction::goto(consequent_end);

        collector.push(cond);
        ctxt.drop_anonymous_variable().unwrap();

        ctxt.set_label_position(consequent_start, collector.len() as u32)
            .unwrap();

        let branches_subcontext = ctxt.new_subcontext();

        self.consequent().lower(collector, ctxt);

        collector.push(goto_end);

        ctxt.drop_subcontext(branches_subcontext);

        ctxt.set_label_position(alt_start, collector.len() as u32)
            .unwrap();

        self.alternative().lower(collector, ctxt);

        ctxt.drop_subcontext(branches_subcontext);
        ctxt.add_anonymous_variable();

        ctxt.set_label_position(consequent_end, collector.len() as u32)
            .unwrap();
    }
}

impl Lowerable for Bindings {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
        let subcontext_id = ctxt.new_subcontext();
        self.defines().iter().for_each(|b| b.lower(collector, ctxt));

        self.ending_expression().lower(collector, ctxt);

        let len = self.defines().len() as u16;

        collector.push(Instruction::pop_copy(len));
        collector.push(Instruction::pop(len - 1));

        ctxt.drop_subcontext(subcontext_id);
        ctxt.add_anonymous_variable();
    }
}

impl Lowerable for Binding {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
        self.value().lower(collector, ctxt);
        ctxt.name_last_anonymous(self.name().to_owned()).unwrap();
    }
}

impl Lowerable for Ident {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
        // TODO: error when the name is not defined
        let stack_offset = ctxt.resolve_variable(self.name()).unwrap();

        // TODO: convert stack index to u16
        collector.push(Instruction::push_copy(stack_offset as u16));

        ctxt.add_anonymous_variable();
    }
}

#[cfg(test)]
fn lower_expr(expr: &impl Lowerable) -> (Vec<Instruction>, Context) {
    let mut collector = Vec::new();
    let mut ctxt = Context::new();

    expr.lower(&mut collector, &mut ctxt);

    (collector, ctxt)
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

        assert_eq!(ctxt.variables_stack_len(), 1);
        assert!(ctxt.variables_stack_top().unwrap().is_empty());
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

        assert_eq!(ctxt.variables_stack_len(), 1);
        assert!(ctxt.variables_stack_top().unwrap().is_empty());
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

        assert_eq!(ctxt.variables_stack_len(), 1);
        assert!(ctxt.variables_stack_top().unwrap().is_empty());
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

        assert_eq!(ctxt.resolve(0).unwrap(), 2);
        assert_eq!(ctxt.resolve(1).unwrap(), 4);
        assert_eq!(ctxt.resolve(2).unwrap(), 5);
    }

    #[test]
    fn stack_effects() {
        let (_, ctxt) = lower_expr(&simple_if());

        assert_eq!(ctxt.variables_stack_len(), 1);
        assert!(ctxt.variables_stack_top().unwrap().is_empty());
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

        assert_eq!(ctxt.variables_stack_len(), 1);
        assert!(ctxt.variables_stack_top().unwrap().is_empty());
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

        assert_eq!(ctxt.variables_stack_len(), 1);
        assert_eq!(ctxt.variables_stack_top().unwrap(), "foo");
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
        ctxt.add_variable("foo".to_owned());
        ctxt.add_variable("bar".to_owned());

        let mut instructions = Vec::new();

        simple_ident().lower(&mut instructions, &mut ctxt);

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

        assert_eq!(ctxt.variables_stack_len(), 3);
        assert!(ctxt.variables_stack_top().unwrap().is_empty());
    }
}
