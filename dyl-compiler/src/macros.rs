use std::ops::{Add, Mul};

use crate::ast;

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
            [ $( $parsed:tt )* ($name, $( $value )* ) ]
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
        Expr::Block {
            bindings: vec![$(
                (stringify!($key), parse_expr! { $( $value )* }),
            )*],
            ending: parse_expr! { $( $tt )* },
        }
    };
}

macro_rules! parse_block {
    ( $( $tt:tt )* ) => {
        parse_block_inner! { [ $( $tt )* ] [] }
    };
}

macro_rules! parse_expr_inner {
    ( [] [ $( $parsed:tt )* ] ) => {
        $( $parsed )*
    };

    (
        [ $id:ident $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* Box::new(Expr::Ident(stringify!($id)))]
        }
    };

    (
        [ $lit:literal $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* Expr::from($lit) ]
        }
    };

    (
        [ { $( $block_content:tt )* } $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* parse_block! { $( $block_content )* } ]
        }
    };
}

macro_rules! parse_expr {
    ( $( $tt:tt )* ) => {
        parse_expr_inner! { [ $( $tt )* ] [] }
    };
}

macro_rules! parse_program {
    ($(
        fn $name:ident() { $( $body:tt )* }
    )*) => {
        Program([
            $(
                Function(
                    stringify!($name),
                    parse_block! { $( $body )* },
                ),
            )*
        ])
    };
}

struct Program<const N: usize>([Function; N]);

impl<const N: usize> From<[Function; N]> for Program<N> {
    fn from(functions: [Function; N]) -> Program<N> {
        Program(functions)
    }
}

impl<const N: usize> From<Program<N>> for ast::Program {
    fn from(program: Program<N>) -> ast::Program {
        let Program(functions) = program;
        let functions = functions.into_iter().map(ast::Function::from).collect();

        ast::Program::new(functions)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Function(&'static str, Expr);

impl From<Function> for ast::Function {
    fn from(function: Function) -> ast::Function {
        let Function(name, body) = function;

        let name = name.to_string();
        let body = ast::ExprKind::from(body);

        ast::Function::new(name, body)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Expr {
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

impl Expr {
    fn if_(cond: Expr, cons: Expr, alt: Expr) -> Expr {
        Expr::If {
            cond: Box::new(cond),
            cons: Box::new(cons),
            alt: Box::new(alt),
        }
    }

    fn ident(id: &'static str) -> Expr {
        Expr::Ident(id)
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

// https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=a628410cdf6e596262c565ff59561031

#[cfg(test)]
mod parse_inline {
    use super::*;

    #[test]
    fn parse_ident() {
        let left: ast::ExprKind = (*parse_expr! { foo }).into();
        let right = ast::ExprKind::ident("foo".to_string());

        assert_eq!(left, right);
    }

    #[test]
    fn parse_function() {
        let left: ast::Program = parse_program! {
            fn main() { foo }
        }
        .into();

        let right = ast::Program::new(vec![ast::Function::new(
            "main".into(),
            ast::ExprKind::ident("foo".to_string()),
        )]);

        assert_eq!(left, right);
    }

    #[test]
    fn multiple_function() {
        let left: ast::Program = parse_program! {
            fn a() { foo }
            fn b() { bar }
        }
        .into();

        let right = ast::Program::new(vec![
            ast::Function::new("a".to_owned(), ast::ExprKind::ident("foo".to_string())),
            ast::Function::new("b".to_owned(), ast::ExprKind::ident("bar".to_string())),
        ]);

        assert_eq!(left, right);
    }

    #[test]
    fn blocks_in_exprs() {
        let left: ast::ExprKind = parse_expr! {
            {
                let a = 42;
                a
            }
        }
        .into();

        let right = ast::ExprKind::bindings(
            vec![ast::Binding::new(
                "a".to_string(),
                ast::ExprKind::integer(42),
            )],
            ast::ExprKind::ident("a".to_string()),
        );

        assert_eq!(left, right);
    }
}
