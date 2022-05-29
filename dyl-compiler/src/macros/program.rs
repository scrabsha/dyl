use crate::ast;

use super::function::Function;

#[macro_export]
macro_rules! inline_program {
    ( $( $tt:tt )* ) => {
        Into::into($crate::parse_program! { $( $tt )* })
    };
}

#[macro_export]
macro_rules! parse_program {
    ($(
        fn $name:ident() { $( $body:tt )* }
    )*) => {
        $crate::node!(program([
            $(
                $crate::inline_fn! { fn $name() { $( $body )* }},
            )*
        ]))
    };
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Program(pub(crate) Vec<Function>);

impl From<Program> for ast::Program {
    fn from(program: Program) -> ast::Program {
        let Program(functions) = program;
        let functions = functions.into_iter().map(ast::Function::from).collect();

        ast::Program::new(functions)
    }
}
