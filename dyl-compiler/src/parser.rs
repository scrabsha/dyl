use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::map,
    error::Error as NomError,
    multi::fold_many1,
    sequence::{delimited, tuple},
    Err, IResult,
};

use anyhow::{ensure, Error, Result as AnyResult};

use crate::ast::ExprKind;

pub(crate) fn parse_input(program: &str) -> AnyResult<ExprKind> {
    addition(program)
        .map_err(|e| own_nom_err(e))
        .map_err(Error::new)
        .and_then(|(tail, expr)| {
            ensure!(tail.is_empty(), "Parser did not consume the whole program");
            Ok(expr)
        })
}

fn own_nom_err(err: Err<nom::error::Error<&str>>) -> Err<nom::error::Error<String>> {
    match err {
        Err::Error(e) => Err::Error(own_nom_error(e)),
        Err::Failure(f) => Err::Failure(own_nom_error(f)),
        Err::Incomplete(needed) => Err::Incomplete(needed),
    }
}

fn own_nom_error(err: NomError<&str>) -> NomError<String> {
    let NomError { input, code } = err;
    let input: String = input.to_owned();
    NomError { input, code }
}

fn integer(input: &str) -> IResult<&str, ExprKind> {
    map(delimited(multispace0, digit1, multispace0), |i: &str| {
        ExprKind::integer(i.parse().unwrap())
    })(input)
}

fn addition(input: &str) -> IResult<&str, ExprKind> {
    let (tail, first) = integer(input)?;

    fold_many1(tuple((tag("+"), integer)), first, |left, (_, right)| {
        ExprKind::addition(left, right)
    })(tail)
}

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
