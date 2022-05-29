pub(crate) mod expression;
pub(crate) mod function;
pub(crate) mod nodes;
pub(crate) mod program;

#[macro_export]
macro_rules! node {
    ($node_name:ident( $( $args:tt )* )) => {
        $crate::macros::nodes::$node_name( $( $args )* )
    };
}

#[cfg(test)]
mod parse_inline {
    use crate::{parse_block, parse_expr, parse_program};

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
}
