use crate::ast::{Addition, ExprKind, If, Integer, Multiplication, Subtraction};
use crate::context::Context;
use crate::instruction::Instruction;

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
        }
    }
}

impl Lowerable for Integer {
    fn lower(&self, collector: &mut Vec<Instruction>, _ctxt: &mut Context) {
        let instr = Instruction::push_i(self.value());
        collector.push(instr);
    }
}

impl Lowerable for Addition {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
        self.left().lower(collector, ctxt);
        self.right().lower(collector, ctxt);

        let instr = Instruction::add_i();
        collector.push(instr);
    }
}

impl Lowerable for Subtraction {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
        self.left().lower(collector, ctxt);
        self.right().lower(collector, ctxt);

        let instructions = [Instruction::neg(), Instruction::add_i()];

        collector.extend_from_slice(&instructions);
    }
}

impl Lowerable for Multiplication {
    fn lower(&self, collector: &mut Vec<Instruction>, ctxt: &mut Context) {
        self.left().lower(collector, ctxt);
        self.right().lower(collector, ctxt);
        collector.push(Instruction::mul());
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

        ctxt.set_label_position(consequent_start, collector.len() as u32)
            .unwrap();

        self.consequent().lower(collector, ctxt);

        collector.push(goto_end);

        ctxt.set_label_position(alt_start, collector.len() as u32)
            .unwrap();

        self.alternative().lower(collector, ctxt);

        ctxt.set_label_position(consequent_end, collector.len() as u32)
            .unwrap();
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

    #[test]
    fn lower_addition_simple() {
        let expr = Addition::new(
            ExprKind::Integer(Integer::new(40)),
            ExprKind::Integer(Integer::new(2)),
        );
        let (left, _) = lower_expr(&expr);

        assert_eq!(
            left,
            [
                Instruction::push_i(40),
                Instruction::push_i(2),
                Instruction::add_i(),
            ]
        );
    }
}

#[cfg(test)]
mod multiplication {
    use super::*;

    #[test]
    fn lower_multiplication() {
        let expr = Multiplication::new(ExprKind::integer(7), ExprKind::integer(6));
        let (left, _) = lower_expr(&expr);

        assert_eq!(
            left,
            [
                Instruction::push_i(7),
                Instruction::push_i(6),
                Instruction::mul(),
            ]
        )
    }
}

#[cfg(test)]
mod subtraction {
    use super::*;

    #[test]
    fn lower_simple() {
        let expr = Subtraction::new(ExprKind::integer(43), ExprKind::integer(1));
        let (left, _) = lower_expr(&expr);

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
}

#[cfg(test)]
mod if_ {
    use super::*;

    #[test]
    fn lower_simple() {
        let expr = If::new(
            ExprKind::integer(1),
            ExprKind::integer(42),
            ExprKind::integer(-1),
        );
        let (left, ctxt) = lower_expr(&expr);

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

        assert_eq!(ctxt.resolve(0).unwrap(), 2);
        assert_eq!(ctxt.resolve(1).unwrap(), 4);
        assert_eq!(ctxt.resolve(2).unwrap(), 5);
    }
}
