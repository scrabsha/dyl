use nom::{
    branch::alt,
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
    level_0_expression(program)
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

fn level_0_expression(input: &str) -> IResult<&str, ExprKind> {
    let (tail, first) = integer(input)?;

    fold_many1(
        tuple((level_0_operator, integer)),
        first,
        |left, (operator, right)| operator.make_expr(left, right),
    )(tail)
}

fn level_0_operator(input: &str) -> IResult<&str, Level0Operator> {
    map(alt((tag("+"), tag("-"))), |operator| match operator {
        "+" => Level0Operator::Plus,
        "-" => Level0Operator::Minus,
        _ => unreachable!(),
    })(input)
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Level0Operator {
    Plus,
    Minus,
}

impl Level0Operator {
    fn make_expr(self, lhs: ExprKind, rhs: ExprKind) -> ExprKind {
        let expression_maker = match self {
            Level0Operator::Plus => ExprKind::addition,
            Level0Operator::Minus => ExprKind::subtraction,
        };

        expression_maker(lhs, rhs)
    }
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
mod add_and_sub {
    use super::*;

    #[test]
    fn single_factor_fails() {
        assert!(level_0_expression("42").is_err());
    }

    #[test]
    fn addition_simple() {
        let left = level_0_expression("1+1");
        let right = Ok((
            "",
            ExprKind::addition(ExprKind::integer(1), ExprKind::integer(1)),
        ));

        assert_eq!(left, right);
    }

    #[test]
    fn addition_right_associative() {
        let left = level_0_expression("1+1+1");
        let right = Ok((
            "",
            ExprKind::addition(
                ExprKind::addition(ExprKind::integer(1), ExprKind::integer(1)),
                ExprKind::integer(1),
            ),
        ));

        assert_eq!(left, right);
    }
    #[test]
    fn subtraction_simple() {
        let left = level_0_expression("43-1");
        let right = Ok((
            "",
            ExprKind::subtraction(ExprKind::integer(43), ExprKind::integer(1)),
        ));

        assert_eq!(left, right);
    }

    #[test]
    fn subtraction_right_associative() {
        let left = level_0_expression("44-1-1");
        let right = Ok((
            "",
            ExprKind::subtraction(
                ExprKind::subtraction(ExprKind::integer(44), ExprKind::integer(1)),
                ExprKind::integer(1),
            ),
        ));

        assert_eq!(left, right);
    }

    #[test]
    fn addition_subtraction_mixed() {
        let left = level_0_expression("42-1+1");
        let right = Ok((
            "",
            ExprKind::addition(
                ExprKind::subtraction(ExprKind::integer(42), ExprKind::integer(1)),
                ExprKind::integer(1),
            ),
        ));

        assert_eq!(left, right);
    }
}
