use crate::ast;

use super::expression::Expr;

#[macro_export]
macro_rules! inline_fn {
    ( $( $tt:tt )* ) => {
        Into::into($crate::parse_fn! { $( $tt )* })
    };
}

#[macro_export]
macro_rules! parse_fn {
    ( fn $name:ident() { $( $body:tt )* } ) => {
        $crate::node!(function(stringify!($name), $crate::parse_block! { $( $body )* },))
    };
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Function(pub(crate) &'static str, pub(crate) Expr);

impl From<Function> for ast::Function {
    fn from(function: Function) -> ast::Function {
        let Function(name, body) = function;

        let name = name.to_string();
        let body = ast::ExprKind::from(body);

        ast::Function::new(name, body)
    }
}
