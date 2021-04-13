use nom::*;

use nom::character::complete::{digit1, multispace0};

use crate::ast::{Addition, ExprKind, Integer};

named!(integer<&str, ExprKind>, map!(
    delimited!(multispace0, digit1, multispace0),
    |i| ExprKind::integer(i.parse().unwrap())
));

named!(addition<&str, ExprKind>, do_parse!(
    first: integer >>
    rest: fold_many1!(
        complete!(tuple!(tag!("+"), integer)),
        first,
        |left, (_, right)| {
            ExprKind::Addition(Addition::new(left, right))
        }
    ) >> (rest)
));

#[cfg(test)]
mod integer {
    use super::*;

    #[test]
    fn integer_simple() {
        let left = integer("42");
        let right = Ok(("", ExprKind::integer(42)));

        assert_eq!(left, right);
    }

    #[test]
    fn integer_with_tail() {
        let left = integer("101 !");
        let right = Ok(("!", ExprKind::integer(101)));

        assert_eq!(left, right);
    }

    #[test]
    fn integer_failing_when_not_digit() {
        assert!(integer("abc").is_err());
        assert!(integer("").is_err());
    }

    #[test]
    fn integer_eats_whitespaces_before_and_after() {
        let left = integer(" 42 ");
        let right = Ok(("", ExprKind::integer(42)));

        assert_eq!(left, right);
    }
}

#[cfg(test)]
mod addition {
    use super::*;

    #[test]
    fn addition_single_factor_fails() {
        assert!(addition("42").is_err());
    }

    #[test]
    fn addition_simple() {
        let left = addition("1+1");
        let right = Ok((
            "",
            ExprKind::addition(ExprKind::integer(1), ExprKind::integer(1)),
        ));

        assert_eq!(left, right);
    }

    #[test]
    fn addition_right_associative() {
        let left = addition("1+1+1");
        let right = Ok((
            "",
            ExprKind::addition(
                ExprKind::addition(ExprKind::integer(1), ExprKind::integer(1)),
                ExprKind::integer(1),
            ),
        ));

        assert_eq!(left, right);
    }
}
