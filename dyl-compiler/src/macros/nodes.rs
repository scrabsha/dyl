use super::{expression::Expr, function::Function, program::Program};

pub(crate) fn addition(lhs: Expr, rhs: Expr) -> Expr {
    let lhs = Box::new(lhs);
    let rhs = Box::new(rhs);

    Expr::Addition { lhs, rhs }
}

pub(crate) fn block<const N: usize>(bs: [(&'static str, Expr); N], ending: Expr) -> Expr {
    let bindings = bs.to_vec();
    let ending = Box::new(ending);

    Expr::Block { bindings, ending }
}

pub(crate) fn function(name: &'static str, body: Expr) -> Function {
    Function(name, body)
}

pub(crate) fn ident(name: &'static str) -> Expr {
    Expr::Ident(name)
}

pub(crate) fn if_(cond: Expr, cons: Expr, alt: Expr) -> Expr {
    let cond = Box::new(cond);
    let cons = Box::new(cons);
    let alt = Box::new(alt);

    Expr::If { cond, cons, alt }
}

pub(crate) fn integer(value: i32) -> Expr {
    Expr::Integer(value)
}

pub(crate) fn literal<T>(lit: T) -> Expr
where
    T: Into<Expr>,
{
    lit.into()
}

pub(crate) fn multiplication(lhs: Expr, rhs: Expr) -> Expr {
    let lhs = Box::new(lhs);
    let rhs = Box::new(rhs);

    Expr::Multiplication { lhs, rhs }
}

pub(crate) fn program<const N: usize>(functions: [Function; N]) -> Program {
    let functions = functions.to_vec();

    Program(functions)
}

pub(crate) fn subtraction(lhs: Expr, rhs: Expr) -> Expr {
    let lhs = Box::new(lhs);
    let rhs = Box::new(rhs);

    Expr::Subtraction { lhs, rhs }
}
