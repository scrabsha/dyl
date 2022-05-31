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
        fn $name:ident() $body:tt
    )*) => {
        $crate::node!(program([
            $(
                $crate::inline_fn! { fn $name() $body },
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

#[cfg(test)]
mod tests {
    use super::super::nodes::*;

    #[test]
    fn single_function() {
        let left = parse_program! {
            fn main() { foo }
        };

        let right = program([function("main", block([], ident("foo")))]);

        assert_eq!(left, right);
    }

    #[test]
    fn multiple_functions() {
        let left = parse_program! {
            fn a() { foo }
            fn b() { bar }
        };

        let right = program([
            function("a", block([], ident("foo"))),
            function("b", block([], ident("bar"))),
        ]);

        assert_eq!(left, right);
    }

    #[test]
    fn integer_as_body() {
        let left = parse_program! {
            fn a() {
                42
            }
        };

        let right = program([function("a", block([], integer(42)))]);

        assert_eq!(left, right);
    }
}
