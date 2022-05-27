use std::ops::{Add, Mul, Neg, Sub};

use crate::ast;

use super::nodes;

#[macro_export]
macro_rules! inline_expr {
    ( $( $tt:tt )* ) => {
        Into::into($crate::parse_expr! { $( $tt )* })
    };
}

#[macro_export]
macro_rules! parse_expr {
    ( $( $tt:tt )* ) => {
        $crate::parse_expr_inner! { [ $( $tt )* ] [] }
    };
}

#[macro_export]
macro_rules! parse_expr_inner {
    ( [] [ $( $parsed:tt )* ] ) => {
        $( $parsed )*
    };

    (
        [ if $( $tail:tt )* ]
        $parsed:tt
    ) => {
        $crate::parse_if! { [ $( $tail )* ] $parsed }
    };

    (
        [ - $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        $crate::parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* -]
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
        [ $id:ident $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        $crate::parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* $crate::node!(ident(stringify!($id)))]
        }
    };

    (
        [ { $( $block_content:tt )* } $( $tail:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        $crate::parse_expr_inner! {
            [ $( $tail )* ]
            [ $( $parsed )* $crate::parse_block! { $( $block_content )* } ]
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
macro_rules! parse_block {
    ( $( $tt:tt )* ) => {
        $crate::parse_block_inner! { [ $( $tt )* ] [] }
    };
}

#[macro_export]
macro_rules! parse_block_inner {
    (
        [ let $name:ident = $( $tt:tt )* ]
        $parsed:tt
    ) => {
        crate::parse_block_inner! {
            @munching_expr [ $( $tt )* ]  [ $name ]
            $parsed
        }
    };

    (
        @munching_expr [ ; $( $tt:tt )* ] [ $name:ident $( $value:tt )* ]
        [ $( $parsed:tt )* ]
    ) => {
        crate::parse_block_inner! {
            [ $( $tt )* ]
            [ $( $parsed )* ($name, $( $value )* ) ]
        }
    };

    (
        @munching_expr [ $head:tt $( $tail:tt )* ] [ $( $current:tt )* ]
        $parsed:tt
    ) => {
        crate::parse_block_inner! {
            @munching_expr [ $( $tail )* ] [ $( $current )* $head ]
            $parsed
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
            $crate::parse_expr! { $( $tt )* },
        ))
    };
}

#[macro_export]
macro_rules! parse_if {
    ( [ $( $tt:tt )* ] $parsed:tt ) => {
        $crate::parse_if_inner! { [ $( $tt )* ] [] $parsed }
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
        $parsed:tt
    ) => {
        $crate::parse_if_inner! {
            [ $( $tail )* ]
            [ $( $cond )* $tok ]
            $parsed
        }
    };
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expr {
    Addition {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Block {
        bindings: Vec<(&'static str, Expr)>,
        ending: Box<Expr>,
    },

    Bool(bool),

    Ident(&'static str),

    If {
        cond: Box<Expr>,
        cons: Box<Expr>,
        alt: Box<Expr>,
    },

    Integer(i32),

    Multiplication {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Subtraction {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

impl From<i32> for Expr {
    fn from(i: i32) -> Expr {
        Expr::Integer(i)
    }
}

impl From<bool> for Expr {
    fn from(b: bool) -> Expr {
        Expr::Bool(b)
    }
}

impl Add for Expr {
    type Output = Expr;

    fn add(self, rhs: Expr) -> Expr {
        nodes::addition(self, rhs)
    }
}

impl Mul for Expr {
    type Output = Expr;

    fn mul(self, other: Expr) -> Expr {
        nodes::multiplication(self, other)
    }
}

impl Neg for Expr {
    type Output = Expr;

    fn neg(self) -> Expr {
        match self {
            Expr::Integer(int) => nodes::integer(-int),

            _ => panic!("`dyl` does not support negations for now"),
        }
    }
}

impl Sub for Expr {
    type Output = Expr;

    fn sub(self, rhs: Self) -> Expr {
        nodes::subtraction(self, rhs)
    }
}

impl From<Expr> for ast::ExprKind {
    fn from(expr: Expr) -> ast::ExprKind {
        match expr {
            Expr::Addition { lhs, rhs } => ast::ExprKind::addition((*lhs).into(), (*rhs).into()),

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

            Expr::Bool(b) => ast::ExprKind::bool_(b),

            Expr::Ident(name) => ast::ExprKind::ident(name.to_string()),

            Expr::If { cond, cons, alt } => {
                ast::ExprKind::if_((*cond).into(), (*cons).into(), (*alt).into())
            }

            Expr::Integer(value) => ast::ExprKind::integer(value),

            Expr::Multiplication { lhs, rhs } => {
                ast::ExprKind::multiplication((*lhs).into(), (*rhs).into())
            }

            Expr::Subtraction { lhs, rhs } => {
                ast::ExprKind::subtraction((*lhs).into(), (*rhs).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::nodes::*;

    #[test]
    fn parse_ident() {
        let left = parse_expr! { foo };
        let right = ident("foo");

        assert_eq!(left, right);
    }

    #[test]
    fn parse_block() {
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

    #[test]
    fn integer_negation() {
        let left = parse_expr! {
            -42
        };
        let right = integer(-42);

        assert_eq!(left, right);
    }

    #[test]
    fn subtraction_simple() {
        let left = parse_expr! {
            101 - 17 - 42
        };

        let right = subtraction(subtraction(integer(101), integer(17)), integer(42));

        assert_eq!(left, right);
    }

    #[test]
    fn bindings_indirection_yeeting() {
        let left: ast::ExprKind = parse_expr! {
            {
                101 + 1
            }
        }
        .into();

        let right = ast::ExprKind::addition(ast::ExprKind::integer(101), ast::ExprKind::integer(1));

        assert_eq!(left, right);
    }
}
