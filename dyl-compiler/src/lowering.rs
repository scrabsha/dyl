use dyl_bytecode::Instruction;

use crate::ast::{Addition, ExprKind, Integer, Subtraction};

pub(crate) fn lower_ast(ast: &ExprKind) -> Vec<Instruction> {
    let mut tmp = Vec::new();
    ast.lower(&mut tmp);

    tmp.push(Instruction::f_stop());

    tmp
}

trait Lowerable {
    fn lower(&self, collector: &mut Vec<Instruction>);
}

impl Lowerable for ExprKind {
    fn lower(&self, collector: &mut Vec<Instruction>) {
        match self {
            ExprKind::Addition(e) => e.lower(collector),
            ExprKind::Integer(e) => e.lower(collector),
            ExprKind::Subtraction(e) => e.lower(collector),
        }
    }
}

impl Lowerable for Integer {
    fn lower(&self, collector: &mut Vec<Instruction>) {
        let instr = Instruction::push_i(self.value());
        collector.push(instr);
    }
}

impl Lowerable for Addition {
    fn lower(&self, collector: &mut Vec<Instruction>) {
        self.right().lower(collector);
        self.left().lower(collector);

        let instr = Instruction::add_i();
        collector.push(instr);
    }
}

impl Lowerable for Subtraction {
    fn lower(&self, collector: &mut Vec<Instruction>) {
        self.left().lower(collector);
        self.right().lower(collector);

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
        let mut collector = Vec::new();

        expr.lower(&mut collector);

        assert_eq!(collector, [Instruction::push_i(42)]);
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
        let mut collector = Vec::new();

        expr.lower(&mut collector);

        assert_eq!(
            collector,
            [
                Instruction::push_i(2),
                Instruction::push_i(40),
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
        let mut collector = Vec::new();

        expr.lower(&mut collector);

        assert_eq!(
            collector,
            [
                Instruction::push_i(43),
                Instruction::push_i(1),
                Instruction::neg(),
                Instruction::add_i(),
            ],
        );
    }
}
