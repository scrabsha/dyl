use std::ops::{Add, Mul};

pub(crate) use crate::ast;

#[macro_export]
macro_rules! node {
    ($node_name:ident( $( $args:tt )* )) => {
        $crate::macros::nodes::$node_name( $( $args )* )
    };
}

#[macro_export]
macro_rules! parse_block_inner {
    (
        [ let $name:ident = $( $tt:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        parse_block_inner! {
            @munching_expr [ $( $tt )* ]  [ $name ]
            [ $( $parsed )* ]
        }
    };

    (
        @munching_expr [ ; $( $tt:tt )* ] [ $name:ident $( $value:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        parse_block_inner! {
            [ $( $tt )* ]
            [ $( $parsed )* ($name, $( $value )* ) ]
        }
    };

    (
        @munching_expr [ $head:tt $( $tail:tt )* ] [ $( $current:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        parse_block_inner! {
            @munching_expr [ $( $tail )* ] [ $( $current )* $head ]
            [ $( $parsed )* ]
        }
    };

    (
        [ $( $tt:tt )* ]
        [
            $(
                ( $key:ident, $( $value:tt )* )
            )*
        ]
    ) => {
        $crate::node!(block(
            [ $( (stringify!($key), $crate::parse_expr! { $( $value )* }) ),* ],
            parse_expr! { $( $tt )* },
        ))
    };
}

#[macro_export]
macro_rules! parse_block {
    ( $( $tt:tt )* ) => {
        $crate::parse_block_inner! { [ $( $tt )* ] [] }
    };
}

#[macro_export]
macro_rules! parse_if_inner {
    (
        [
            { $( $cons:tt )* }
            else
            { $( $alt:tt )* }
            $( $tail:tt )*
        ]
        [ $( $cond:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        $crate::parse_expr_inner! {
            [ $( $tail )* ]
            [
                $( $parsed )*
                $crate::node!(if_(
                    $crate::parse_expr! { $( $cond )* },
                    $crate::parse_block! { $( $cons )* },
                    $crate::parse_block! { $( $alt )* },
                ))
            ]
        }
    };

    (
        [
            $tok:tt $( $tail:tt )*
        ]
        [ $( $cond:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        $crate::parse_if_inner! {
            [ $( $tail )* ]
            [ $( $cond )* $tok ]
            [ $( $parsed )* ]
        }
    };
}

#[macro_export]
macro_rules! parse_if {
    ( [ $( $tt:tt )* ] [ $( $parsed:tt )* ] ) => {
        $crate::parse_if_inner! { [ $( $tt )* ] [] [ $( $parsed )* ] }
    };
}

#[macro_export]
macro_rules! parse_expr_inner {
    ( [] [ $( $parsed:tt )* ] ) => {
        $( $parsed )*
    };

    (
        [ if $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        $crate::parse_if! { [ $( $tail )* ] [ $( $parsed )* ] }
    };

    (
        [ $id:ident $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        $crate::parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* $crate::node!(ident(stringify!($id)))]
        }
    };

    (
        [ $lit:literal $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        $crate::parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* $crate::node!(literal($lit)) ]
        }
    };

    (
        [ { $( $block_content:tt )* } $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        $crate::parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* parse_block! { $( $block_content )* } ]
        }
    };

    (
        [ $tok:tt $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        $crate::parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* $tok ]
        }
    }
}

#[macro_export]
macro_rules! parse_expr {
    ( $( $tt:tt )* ) => {
        Into::into($crate::parse_expr_inner! { [ $( $tt )* ] [] })
    };
}

macro_rules! parse_expr {
    ( $( $tt:tt )* ) => {
        parse_expr_inner! { [ $( $tt )* ] [] }
    };
}

#[macro_export]
macro_rules! parse_program {
    ($(
        fn $name:ident() { $( $body:tt )* }
    )*) => {
        node!(program([
            $(
                node!(function(
                    stringify!($name),
                    parse_block! { $( $body )* },
                )),
            )*
        ]))
    };
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Program(Vec<Function>);

impl From<Program> for ast::Program {
    fn from(program: Program) -> ast::Program {
        let Program(functions) = program;
        let functions = functions.into_iter().map(ast::Function::from).collect();

        ast::Program::new(functions)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Function(&'static str, Expr);

impl From<Function> for ast::Function {
    fn from(function: Function) -> ast::Function {
        let Function(name, body) = function;

        let name = name.to_string();
        let body = ast::ExprKind::from(body);

        ast::Function::new(name, body)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expr {
    Ident(&'static str),
    Integer(i32),
    Addition {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Multiplication {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        cons: Box<Expr>,
        alt: Box<Expr>,
    },
    Block {
        bindings: Vec<(&'static str, Expr)>,
        ending: Box<Expr>,
    },
}

pub(crate) mod nodes {
    use super::*;

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

    pub(crate) fn program<const N: usize>(functions: [Function; N]) -> Program {
        Program(functions.to_vec())
    }
}

impl From<i32> for Expr {
    fn from(i: i32) -> Expr {
        Expr::Integer(i)
    }
}

impl Add for Expr {
    type Output = Expr;

    fn add(self, other: Expr) -> Expr {
        Expr::Addition {
            lhs: Box::new(self),
            rhs: Box::new(other),
        }
    }
}

impl Mul for Expr {
    type Output = Expr;

    fn mul(self, other: Expr) -> Expr {
        Expr::Multiplication {
            lhs: Box::new(self),
            rhs: Box::new(other),
        }
    }
}

impl From<Expr> for ast::ExprKind {
    fn from(expr: Expr) -> ast::ExprKind {
        match expr {
            Expr::Ident(name) => ast::ExprKind::ident(name.to_string()),
            Expr::Integer(value) => ast::ExprKind::integer(value),
            Expr::Addition { lhs, rhs } => {
                ast::ExprKind::addition(ast::ExprKind::from(*lhs), ast::ExprKind::from(*rhs))
            }
            Expr::Multiplication { lhs, rhs } => {
                ast::ExprKind::multiplication(ast::ExprKind::from(*lhs), ast::ExprKind::from(*rhs))
            }
            Expr::If { cond, cons, alt } => ast::ExprKind::if_(
                ast::ExprKind::from(*cond),
                ast::ExprKind::from(*cons),
                ast::ExprKind::from(*alt),
            ),
            Expr::Block { bindings, ending } => {
                if bindings.is_empty() {
                    (*ending).into()
                } else {
                    ast::ExprKind::bindings(
                        bindings
                            .into_iter()
                            .map(|(name, value)| ast::Binding::new(name.to_string(), value.into()))
                            .collect(),
                        (*ending).into(),
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod parse_inline {
    use super::*;

    use nodes::*;

    #[test]
    fn parse_ident() {
        let left = parse_expr! { foo };
        let right = ident("foo");

        assert_eq!(left, right);
    }

    #[test]
    fn parse_function() {
        let left = parse_program! {
            fn main() { foo }
        };

        let right = program([function("main", block([], ident("foo")))]);

        assert_eq!(left, right);
    }

    #[test]
    fn multiple_function() {
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
    fn blocks_in_exprs() {
        let left = parse_expr! {
            {
                let a = 42;
                a
            }
        };

        let right = block([("a", integer(42))], ident("a"));

        assert_eq!(left, right);
    }

    #[test]
    fn body_with_integer() {
        let left = parse_program! {
            fn a() {
                42
            }
        };

        let right = program([function("a", block([], integer(42)))]);

        assert_eq!(left, right);
    }

    #[test]
    fn double_bindings() {
        let left = parse_block! {
            let a = 1;
            let b = 2;
            a
        };

        let right = block([("a", integer(1)), ("b", integer(2))], ident("a"));

        assert_eq!(left, right);
    }

    #[test]
    fn if_else() {
        let left = parse_expr! {
            if 1 {
                let a = 42;
                a
            } else {
                101
            }
        };

        let right = if_(
            integer(1),
            block([("a", integer(42))], ident("a")),
            block([], integer(101)),
        );

        assert_eq!(left, right);
    }

    #[test]
    fn addition_() {
        let left = parse_expr! { a + b };

        let right = addition(ident("a"), ident("b"));

        assert_eq!(left, right);
    }

    #[test]
    fn cute_if_else() {
        let left = parse_expr! {
            if 1 {
                let a = 42;
                let b = 101;
                a + b
            } else {
                101
            }
        };

        use nodes::*;

        let right = if_(
            integer(1),
            block(
                [("a", integer(42)), ("b", integer(101))],
                addition(ident("a"), ident("b")),
            ),
            block([], integer(101)),
        );

        assert_eq!(left, right);
    }

    #[test]
    fn addition_as_part_of_if() {
        let left = parse_expr! {
            if 1 { 1 } else { 1 } + 1
        };

        let right = addition(
            if_(integer(1), block([], integer(1)), block([], integer(1))),
            integer(1),
        );

        assert_eq!(left, right);
    }
}
