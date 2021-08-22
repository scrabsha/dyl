use crate::{
    ast::{
        Addition, Binding, Bindings, Bool, ExprKind, Ident, If, Integer, Multiplication,
        Subtraction,
    },
    context::{CompilerPassError, TypingContext},
    ty::Ty,
};

pub(crate) fn check_ast(
    ast: &ExprKind,
    mut ctxt: TypingContext,
) -> Result<TypingContext, CompilerPassError> {
    let input_check_rslt = ast.check_inputs(&mut ctxt);
    let output_ty = ast
        .get_output(&mut ctxt)
        .and_then(|ty| ty.eq(&Ty::Int).then(|| ()).ok_or(()));

    let pass_rslt = input_check_rslt.and(output_ty);

    ctxt.wrap_result(pass_rslt).map(|(ctxt, ())| ctxt)
}

trait Typed {
    fn check_inputs(&self, ctxt: &mut TypingContext) -> Result<(), ()>;

    fn get_output(&self, ctxt: &mut TypingContext) -> Result<Ty, ()>;
}

impl Typed for ExprKind {
    fn check_inputs(&self, ctxt: &mut TypingContext) -> Result<(), ()> {
        match self {
            ExprKind::Addition(addition) => addition.check_inputs(ctxt),
            ExprKind::Integer(integer) => integer.check_inputs(ctxt),
            ExprKind::Bindings(bindings) => bindings.check_inputs(ctxt),
            ExprKind::Ident(ident) => ident.check_inputs(ctxt),
            ExprKind::Multiplication(multiplication) => multiplication.check_inputs(ctxt),
            ExprKind::Subtraction(subtraction) => subtraction.check_inputs(ctxt),
            ExprKind::If(if_) => if_.check_inputs(ctxt),
            ExprKind::Bool(bool_) => bool_.check_inputs(ctxt),
        }
    }

    fn get_output(&self, ctxt: &mut TypingContext) -> Result<Ty, ()> {
        match self {
            ExprKind::Addition(addition) => addition.get_output(ctxt),
            ExprKind::Integer(integer) => integer.get_output(ctxt),
            ExprKind::Bindings(bindings) => bindings.get_output(ctxt),
            ExprKind::Ident(ident) => ident.get_output(ctxt),
            ExprKind::Multiplication(multiplication) => multiplication.get_output(ctxt),
            ExprKind::Subtraction(subtraction) => subtraction.get_output(ctxt),
            ExprKind::If(if_) => if_.get_output(ctxt),
            ExprKind::Bool(bool_) => bool_.get_output(ctxt),
        }
    }
}

impl Typed for Addition {
    fn check_inputs(&self, ctxt: &mut TypingContext) -> Result<(), ()> {
        let operands_are_valid = self
            .left()
            .check_inputs(ctxt)
            .and(self.right().check_inputs(ctxt));

        let left_is_int = self.left().get_output(ctxt).and_then(|ty| ty.expect_int());
        let right_is_int = self.right().get_output(ctxt).and_then(|ty| ty.expect_int());

        operands_are_valid.and(left_is_int).and(right_is_int)
    }

    fn get_output(&self, _ctxt: &mut TypingContext) -> Result<Ty, ()> {
        Ok(Ty::Int)
    }
}

impl Typed for Integer {
    fn check_inputs(&self, _ctxt: &mut TypingContext) -> Result<(), ()> {
        Ok(())
    }

    fn get_output(&self, _ctxt: &mut TypingContext) -> Result<Ty, ()> {
        Ok(Ty::Int)
    }
}

impl Typed for Bindings {
    fn check_inputs(&self, ctxt: &mut TypingContext) -> Result<(), ()> {
        let subctxt = ctxt.new_subcontext();

        let mut bindings_are_valid = Ok(());

        self.defines().iter().for_each(|binding| {
            bindings_are_valid = bindings_are_valid.and(binding.check_inputs(ctxt));

            // Next bindings and final expression may use this binding. Let's
            // add it to the context.
            let binding_ty = binding.value().get_output(ctxt).unwrap_or(Ty::Err);
            ctxt.add_binding(binding.name().to_owned(), binding_ty);
        });

        let final_is_valid = self.ending_expression().check_inputs(ctxt);
        ctxt.drop_subcontext(subctxt);

        bindings_are_valid.and(final_is_valid)
    }

    fn get_output(&self, ctxt: &mut TypingContext) -> Result<Ty, ()> {
        let subctxt = ctxt.new_subcontext();

        self.defines().iter().for_each(|binding| {
            // Next bindings and final expression may use this binding. Let's
            // add it to the context.
            let binding_ty = binding.value().get_output(ctxt).unwrap_or(Ty::Err);
            ctxt.add_binding(binding.name().to_owned(), binding_ty);
        });

        let expr_ty = self.ending_expression().get_output(ctxt);
        ctxt.drop_subcontext(subctxt);

        expr_ty
    }
}

impl Typed for Binding {
    fn check_inputs(&self, ctxt: &mut TypingContext) -> Result<(), ()> {
        self.value().check_inputs(ctxt)
    }

    fn get_output(&self, ctxt: &mut TypingContext) -> Result<Ty, ()> {
        self.value().get_output(ctxt)
    }
}

impl Typed for Ident {
    fn check_inputs(&self, ctxt: &mut TypingContext) -> Result<(), ()> {
        ctxt.resolve_binding(self.name()).map(drop).ok_or(())
    }

    fn get_output(&self, ctxt: &mut TypingContext) -> Result<Ty, ()> {
        ctxt.resolve_binding(self.name()).cloned().ok_or(())
    }
}

impl Typed for Multiplication {
    fn check_inputs(&self, ctxt: &mut TypingContext) -> Result<(), ()> {
        let operands_are_valid = self
            .left()
            .check_inputs(ctxt)
            .and(self.right().check_inputs(ctxt));

        let left_is_int = self.left().get_output(ctxt).and_then(|ty| ty.expect_int());
        let right_is_int = self.right().get_output(ctxt).and_then(|ty| ty.expect_int());

        operands_are_valid.and(left_is_int).and(right_is_int)
    }

    fn get_output(&self, _ctxt: &mut TypingContext) -> Result<Ty, ()> {
        Ok(Ty::Int)
    }
}

impl Typed for Subtraction {
    fn check_inputs(&self, ctxt: &mut TypingContext) -> Result<(), ()> {
        let operands_are_valid = self
            .left()
            .check_inputs(ctxt)
            .and(self.right().check_inputs(ctxt));

        let left_is_int = self.left().get_output(ctxt).and_then(|ty| ty.expect_int());
        let right_is_int = self.right().get_output(ctxt).and_then(|ty| ty.expect_int());

        operands_are_valid.and(left_is_int).and(right_is_int)
    }

    fn get_output(&self, _ctxt: &mut TypingContext) -> Result<Ty, ()> {
        Ok(Ty::Int)
    }
}

impl Typed for If {
    fn check_inputs(&self, ctxt: &mut TypingContext) -> Result<(), ()> {
        let children_check = self
            .condition()
            .check_inputs(ctxt)
            .and(self.consequent().check_inputs(ctxt))
            .and(self.alternative().check_inputs(ctxt));

        let consequent_ty = self.consequent().get_output(ctxt).unwrap_or(Ty::Err);
        let alternative_ty = self.alternative().get_output(ctxt).unwrap_or(Ty::Err);

        let branches_unify = consequent_ty.unify_with(alternative_ty).map(drop);

        let condition_is_bool = self
            .condition()
            .get_output(ctxt)
            .and_then(|ty| ty.expect_bool());

        children_check.and(branches_unify).and(condition_is_bool)
    }

    fn get_output(&self, ctxt: &mut TypingContext) -> Result<Ty, ()> {
        let consequent_ty = self.consequent().get_output(ctxt).unwrap_or(Ty::Err);
        let alternative_ty = self.alternative().get_output(ctxt).unwrap_or(Ty::Err);

        consequent_ty.unify_with(alternative_ty)
    }
}

impl Typed for Bool {
    fn check_inputs(&self, _ctxt: &mut TypingContext) -> Result<(), ()> {
        Ok(())
    }

    fn get_output(&self, _ctxt: &mut TypingContext) -> Result<Ty, ()> {
        Ok(Ty::Bool)
    }
}

#[cfg(test)]
mod addition {
    use super::*;

    fn sample_addition() -> ExprKind {
        ExprKind::addition(ExprKind::integer(41), ExprKind::integer(1))
    }

    fn bool_on_left() -> ExprKind {
        ExprKind::addition(ExprKind::bool_(true), ExprKind::integer(42))
    }

    #[test]
    fn returns_integer() {
        let mut ctxt = TypingContext::new();
        let expr = sample_addition();
        let output = expr.get_output(&mut ctxt);

        assert_eq!(output, Ok(Ty::Int));
    }

    #[test]
    fn expects_integers_ok() {
        let mut ctxt = TypingContext::new();
        let expr = sample_addition();

        assert!(expr.check_inputs(&mut ctxt).is_ok());
    }

    #[test]
    fn expects_integers_err() {
        let mut ctxt = TypingContext::new();

        assert!(bool_on_left().check_inputs(&mut ctxt).is_err());
    }

    #[test]
    fn double_addition_with_labels() {
        let expr = ExprKind::addition(
            ExprKind::addition(
                ExprKind::ident("a".to_owned()),
                ExprKind::ident("b".to_owned()),
            ),
            ExprKind::integer(1),
        );

        let mut ctxt = TypingContext::new();
        ctxt.add_binding("a".to_owned(), Ty::Int);
        ctxt.add_binding("b".to_owned(), Ty::Int);

        assert!(expr.check_inputs(&mut ctxt).is_ok());
        assert_eq!(expr.get_output(&mut ctxt), Ok(Ty::Int));
    }
}

#[cfg(test)]
mod integer {
    use super::*;

    fn sample_integer() -> ExprKind {
        ExprKind::integer(42)
    }

    #[test]
    fn inputs_always_successful() {
        let mut ctxt = TypingContext::new();
        assert!(sample_integer().check_inputs(&mut ctxt).is_ok());
    }

    #[test]
    fn output_always_integer() {
        let mut ctxt = TypingContext::new();
        assert_eq!(sample_integer().get_output(&mut ctxt), Ok(Ty::Int));
    }
}

#[cfg(test)]
mod bindings {
    use crate::ast::Binding;

    use super::*;

    fn sample_bindings() -> ExprKind {
        ExprKind::Bindings(Bindings::from_vec(
            vec![
                Binding::new("a".to_owned(), ExprKind::integer(40)),
                Binding::new("b".to_owned(), ExprKind::integer(1)),
            ],
            ExprKind::addition(
                ExprKind::addition(
                    ExprKind::ident("a".to_owned()),
                    ExprKind::ident("b".to_owned()),
                ),
                ExprKind::integer(1),
            ),
        ))
    }

    fn bindings_with_type_errors() -> ExprKind {
        ExprKind::single_binding(
            "foo".to_owned(),
            ExprKind::addition(ExprKind::bool_(true), ExprKind::integer(42)),
            ExprKind::ident("foo".to_owned()),
        )
    }

    fn bindings_with_unknown_ident() -> ExprKind {
        ExprKind::Bindings(Bindings::from_vec(
            vec![Binding::new("foo".to_owned(), ExprKind::integer(42))],
            ExprKind::ident("bar".to_owned()),
        ))
    }

    #[test]
    fn check_input_ok() {
        let mut ctxt = TypingContext::new();

        assert!(sample_bindings().check_inputs(&mut ctxt).is_ok());
    }

    #[test]
    fn check_inputs_type_error() {
        let mut ctxt = TypingContext::new();

        assert!(bindings_with_type_errors().check_inputs(&mut ctxt).is_err());
    }

    #[test]
    fn get_output_unknown_ident() {
        let mut ctxt = TypingContext::new();

        assert!(bindings_with_unknown_ident().get_output(&mut ctxt).is_err());
    }
}

#[cfg(test)]
mod ident {
    use super::*;

    fn sample_ident() -> ExprKind {
        ExprKind::ident("foo".to_owned())
    }

    #[test]
    fn check_input_success() {
        let mut ctxt = TypingContext::new();
        ctxt.add_binding("foo".to_owned(), Ty::Int);

        assert!(sample_ident().check_inputs(&mut ctxt).is_ok());
    }

    #[test]
    fn check_input_err_not_defined() {
        let mut ctxt = TypingContext::new();
        ctxt.add_binding("bar".to_owned(), Ty::Bool);

        assert!(sample_ident().check_inputs(&mut ctxt).is_err());
    }

    #[test]
    fn get_output_success() {
        let mut ctxt = TypingContext::new();
        ctxt.add_binding("foo".to_owned(), Ty::Bool);

        assert_eq!(sample_ident().get_output(&mut ctxt), Ok(Ty::Bool));
    }

    #[test]
    fn get_output_not_found() {
        let mut ctxt = TypingContext::new();
        assert!(sample_ident().get_output(&mut ctxt).is_err())
    }
}

#[cfg(test)]
mod multiplication {
    use super::*;

    fn sample_multiplication() -> ExprKind {
        ExprKind::multiplication(ExprKind::integer(41), ExprKind::integer(1))
    }

    #[test]
    fn returns_integer() {
        let mut ctxt = TypingContext::new();
        let expr = sample_multiplication();
        let output = expr.get_output(&mut ctxt);

        assert_eq!(output, Ok(Ty::Int));
    }

    #[test]
    fn expects_integers_ok() {
        let mut ctxt = TypingContext::new();
        let expr = sample_multiplication();

        assert!(expr.check_inputs(&mut ctxt).is_ok());
    }

    #[test]
    fn expects_integers_err() {
        let mut ctxt = TypingContext::new();
        let expr = ExprKind::multiplication(ExprKind::bool_(true), ExprKind::integer(42));

        assert!(expr.check_inputs(&mut ctxt).is_err());
    }
}

#[cfg(test)]
mod subtraction {
    use super::*;

    fn sample_subtraction() -> ExprKind {
        ExprKind::subtraction(ExprKind::integer(41), ExprKind::integer(1))
    }

    #[test]
    fn returns_integer() {
        let mut ctxt = TypingContext::new();
        let expr = sample_subtraction();
        let output = expr.get_output(&mut ctxt);

        assert_eq!(output, Ok(Ty::Int));
    }

    #[test]
    fn expects_integers_ok() {
        let mut ctxt = TypingContext::new();
        let expr = sample_subtraction();

        assert!(expr.check_inputs(&mut ctxt).is_ok());
    }

    #[test]
    fn expects_integers_err() {
        let mut ctxt = TypingContext::new();
        let expr = ExprKind::subtraction(ExprKind::bool_(true), ExprKind::integer(42));

        assert!(expr.check_inputs(&mut ctxt).is_err());
    }
}

#[cfg(test)]
mod if_ {
    use super::*;

    fn sample_if() -> ExprKind {
        ExprKind::if_(
            ExprKind::bool_(true),
            ExprKind::integer(101),
            ExprKind::integer(42),
        )
    }

    fn if_different_consequent_and_alternative() -> ExprKind {
        ExprKind::if_(
            ExprKind::bool_(false),
            ExprKind::integer(101),
            ExprKind::bool_(false),
        )
    }

    #[test]
    fn check_inputs_working() {
        let mut ctxt = TypingContext::new();

        assert!(sample_if().check_inputs(&mut ctxt).is_ok());
    }

    #[test]
    fn get_output_ok() {
        let mut ctxt = TypingContext::new();

        assert_eq!(sample_if().get_output(&mut ctxt), Ok(Ty::Int));
    }

    #[test]
    fn get_output_consequent_and_alternative_different() {
        let mut ctxt = TypingContext::new();

        assert!(if_different_consequent_and_alternative()
            .get_output(&mut ctxt)
            .is_err());
    }
}

#[cfg(test)]
mod bool_ {
    use super::*;

    fn sample_bool() -> ExprKind {
        ExprKind::bool_(true)
    }

    #[test]
    fn input_always_checks() {
        let mut ctxt = TypingContext::new();

        assert!(sample_bool().check_inputs(&mut ctxt).is_ok());
    }

    #[test]
    fn always_outputs_bool() {
        let mut ctxt = TypingContext::new();

        assert_eq!(sample_bool().get_output(&mut ctxt), Ok(Ty::Bool))
    }
}
