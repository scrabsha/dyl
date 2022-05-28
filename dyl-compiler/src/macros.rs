use std::ops::{Add, Mul};

use crate::ast;

//////
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
        Expr::Block {
            bindings: vec![$(
                (stringify!($key), parse_expr! { $( $value )* }),
            )*],
            ending: Box::new(parse_expr! { $( $tt )* }),
        }
    };
}

macro_rules! parse_block {
    ( $( $tt:tt )* ) => {
        parse_block_inner! { [ $( $tt )* ] [] }
    };
}

macro_rules! parse_if_inner {
    (
        [
            { $( $cons:tt )* }
            else
            { $( $alt:tt )* }
        ]
        [ $( $cond:tt )* ]
    ) => {
        Expr::If {
            cond: Box::new( parse_expr! { $( $cond )* } ),
            cons: Box::new( parse_block! { $( $cons )* } ),
            alt: Box::new( parse_block! { $( $alt )* } ),
        }
    };

    (
        [
            $tok:tt $( $tail:tt )*
        ]
        [ $( $cond:tt )* ]
    ) => {
        parse_if_inner! {
            [ $( $tail )* ]
            [ $( $cond )* $tok ]
        }
    };
}

macro_rules! parse_if {
    ( $( $tt:tt )* ) => {
        parse_if_inner! { [ $( $tt )* ] [] }
    };
}

macro_rules! parse_expr_inner {
    ( [] [ $( $parsed:tt )* ] ) => {
        $( $parsed )*
    };

    (
        [ if $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        parse_if! { $( $tail )* }
    };

    (
        [ $id:ident $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* Expr::Ident(stringify!($id))]
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

    (
        [ $tok:tt $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* $tok ]
        }
    }
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

mod expr {
    use super::*;

    pub(super) fn addition(lhs: Expr, rhs: Expr) -> Expr {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);

        Expr::Addition { lhs, rhs }
    }

    pub(super) fn block<const N: usize>(bs: [(&'static str, Expr); N], ending: Expr) -> Expr {
        let bindings = bs.to_vec();
        let ending = Box::new(ending);

        Expr::Block { bindings, ending }
    }

    pub(super) fn ident(name: &'static str) -> Expr {
        Expr::Ident(name)
    }

    pub(super) fn if_(cond: Expr, cons: Expr, alt: Expr) -> Expr {
        let cond = Box::new(cond);
        let cons = Box::new(cons);
        let alt = Box::new(alt);

        Expr::If { cond, cons, alt }
    }

    pub(super) fn integer(value: i32) -> Expr {
        Expr::Integer(value)
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

impl Expr {
    fn if_(cond: Expr, cons: Expr, alt: Expr) -> Expr {
        Expr::If {
            cond: Box::new(cond),
            cons: Box::new(cons),
            alt: Box::new(alt),
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

    use super::expr::*;

    #[test]
    fn parse_ident() {
        let left: ast::ExprKind = parse_expr! { foo }.into();
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

    #[test]
    fn body_with_integer() {
        let left: ast::Program = parse_program! {
            fn a() {
                42
            }
        }
        .into();

        let right = ast::Program::new(vec![ast::Function::new(
            "a".into(),
            ast::ExprKind::integer(42),
        )]);

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
        let left: ast::ExprKind = parse_expr! {
            if 1 {
                let a = 42;
                a
            } else {
                101
            }
        }
        .into();

        let right = ast::ExprKind::if_(
            ast::ExprKind::integer(1),
            ast::ExprKind::bindings(
                vec![ast::Binding::new(
                    "a".to_string(),
                    ast::ExprKind::integer(42),
                )],
                ast::ExprKind::ident("a".to_string()),
            ),
            ast::ExprKind::integer(101),
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
}
