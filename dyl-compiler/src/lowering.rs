use crate::ast::{Addition, ExprKind, Integer, Subtraction};
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
