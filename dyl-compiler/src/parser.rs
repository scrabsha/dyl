use anyhow::{ensure, Error, Result as AnyResult};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, digit1, multispace0},
    combinator::{map, opt, recognize},
    error::{Error as NomError, ErrorKind, ParseError},
    multi::{fold_many1, many0, many1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Err, IResult, Parser,
};

use crate::ast::{Binding, ExprKind};

pub(crate) fn parse_input(input_code: &str) -> AnyResult<ExprKind> {
    program(input_code)
        .map_err(own_nom_err)
        .map_err(Error::new)
        .and_then(|(tail, expr)| {
            ensure!(
                tail.is_empty(),
                "Parser did not consume the whole program: {} remains",
                tail
            );
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

fn program(input: &str) -> IResult<&str, ExprKind> {
    alt((bindings, expr))(input)
}

fn block(input: &str) -> IResult<&str, ExprKind> {
    delimited(left_curly, alt((bindings, expr)), right_curly)(input)
}

fn expr(input: &str) -> IResult<&str, ExprKind> {
    alt((level_0_expression, level_1_expression, atomic_expr))(input)
}

fn integer(input: &str) -> IResult<&str, ExprKind> {
    let maybe_minus = opt(tag("-"));

    map(
        space_insignificant(recognize(tuple((maybe_minus, digit1)))),
        |i: &str| ExprKind::integer(i.parse().unwrap()),
    )(input)
}

fn level_0_expression(input: &str) -> IResult<&str, ExprKind> {
    let (tail, first) = alt((level_1_expression, atomic_expr))(input)?;

    fold_many1(
        tuple((level_0_operator, alt((level_1_expression, atomic_expr)))),
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

fn level_1_expression(input: &str) -> IResult<&str, ExprKind> {
    let (tail, first) = atomic_expr(input)?;
    fold_many1(tuple((star, atomic_expr)), first, |lhs, (_, rhs)| {
        ExprKind::multiplication(lhs, rhs)
    })(tail)
}

fn star(input: &str) -> IResult<&str, ()> {
    map(space_insignificant(tag("*")), drop)(input)
}

fn if_else(input: &str) -> IResult<&str, ExprKind> {
    let (tail, _) = if_(input)?;
    let (tail, condition) = expr(tail)?;
    let (tail, consequent) = delimited(left_curly, expr, right_curly)(tail)?;
    let (tail, _) = else_(tail)?;
    let (tail, alternative) = delimited(left_curly, expr, right_curly)(tail)?;

    let if_ = ExprKind::if_(condition, consequent, alternative);
    Ok((tail, if_))
}

fn bindings(input: &str) -> IResult<&str, ExprKind> {
    let (tail, bs) = many1(binding)(input)?;
    let (tail, ending) = expr(tail)?;
    let bindings = ExprKind::bindings(bs, ending);

    Ok((tail, bindings))
}

fn binding(input: &str) -> IResult<&str, Binding> {
    let (tail, name) = delimited(let_, ident, equal)(input)?;
    let (tail, value) = terminated(expr, semicolon)(tail)?;
    Ok((tail, Binding::new(name, value)))
}

fn atomic_expr(input: &str) -> IResult<&str, ExprKind> {
    alt((integer, if_else, block, ident_expr))(input)
}

fn ident_expr(input: &str) -> IResult<&str, ExprKind> {
    let (tail, name) = ident(input)?;
    Ok((tail, ExprKind::ident(name)))
}

fn ident(input: &str) -> IResult<&str, String> {
    let (tail, name) = space_insignificant(recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    )))(input)?;

    Ok((tail, name.to_string()))
}

fn if_(input: &str) -> IResult<&str, ()> {
    keyword("if")(input)
}

fn else_(input: &str) -> IResult<&str, ()> {
    keyword("else")(input)
}

fn let_(input: &str) -> IResult<&str, ()> {
    keyword("let")(input)
}

fn equal(input: &str) -> IResult<&str, ()> {
    map(space_insignificant(tag("=")), drop)(input)
}

fn semicolon(input: &str) -> IResult<&str, ()> {
    map(space_insignificant(tag(";")), drop)(input)
}

fn keyword(kw: &str) -> impl Fn(&str) -> IResult<&str, ()> + '_ {
    move |input| {
        let (tail, _) = map(preceded(multispace0, tag(kw)), drop)(input)?;
        let next_is_alphabetic = tail
            .chars()
            .next()
            .map(char::is_alphabetic)
            .unwrap_or(false);

        if next_is_alphabetic {
            Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
        } else {
            let (tail, _) = multispace0(tail)?;
            Ok((tail, ()))
        }
    }
}

fn left_curly(input: &str) -> IResult<&str, ()> {
    map(space_insignificant(tag("{")), drop)(input)
}

fn right_curly(input: &str) -> IResult<&str, ()> {
    map(space_insignificant(tag("}")), drop)(input)
}

fn space_insignificant<'a, O, E>(
    parser: impl Parser<&'a str, O, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    E: ParseError<&'a str>,
{
    delimited(multispace0, parser, multispace0)
}

#[cfg(test)]
mod program {
    use super::*;

    // TODO: once we get function parsing, replace it with a set of functions

    #[test]
    fn handles_bindings() {
        let left = program("let a = 40; let b = 2; a + b");
        let right = Ok((
            "",
            ExprKind::bindings(
                vec![
                    Binding::new("a".to_owned(), ExprKind::integer(40)),
                    Binding::new("b".to_owned(), ExprKind::integer(2)),
                ],
                ExprKind::addition(
                    ExprKind::ident("a".to_owned()),
                    ExprKind::ident("b".to_owned()),
                ),
            ),
        ));

        assert_eq!(left, right);
    }

    #[test]
    fn handles_expression() {
        let left = program("1 + 2 + 2");
        let right = Ok((
            "",
            ExprKind::addition(
                ExprKind::addition(ExprKind::integer(1), ExprKind::integer(2)),
                ExprKind::integer(2),
            ),
        ));

        assert_eq!(left, right);
    }
}

#[cfg(test)]
mod block {
    use super::*;

    #[test]
    fn handles_bindings() {
        let left = block("{ let a = 42; a }");
        let right = Ok((
            "",
            ExprKind::bindings(
                vec![Binding::new("a".to_owned(), ExprKind::integer(42))],
                ExprKind::ident("a".to_owned()),
            ),
        ));

        assert_eq!(left, right);
    }

    #[test]
    fn handles_expression() {
        let left = block("{ 42 }");
        let right = Ok(("", ExprKind::integer(42)));

        assert_eq!(left, right);
    }
}

#[cfg(test)]
mod expr {
    use super::*;

    #[test]
    fn if_addition_parses() {
        let left = expr("if 1 { 1 } else { 1 } + 1");
        let right = Ok((
            "",
            ExprKind::addition(
                ExprKind::if_(
                    ExprKind::integer(1),
                    ExprKind::integer(1),
                    ExprKind::integer(1),
                ),
                ExprKind::integer(1),
            ),
        ));

        assert_eq!(left, right);
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

    #[test]
    fn negative() {
        let left = integer("-101");
        let right = Ok(("", ExprKind::integer(-101)));

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

#[cfg(test)]
mod mul {
    use super::*;

    #[test]
    fn parse_simple() {
        let left = level_1_expression("7*6");
        let right = Ok((
            "",
            ExprKind::multiplication(ExprKind::integer(7), ExprKind::integer(6)),
        ));

        assert_eq!(left, right);
    }

    #[test]
    fn when_spaced() {
        let left = level_1_expression("21 * 2");
        let right = Ok((
            "",
            ExprKind::multiplication(ExprKind::integer(21), ExprKind::integer(2)),
        ));

        assert_eq!(left, right);
    }
}

#[cfg(test)]
mod math {
    use super::*;

    #[test]
    fn priority_simple() {
        let left = level_0_expression("10 * 4 + 2");
        let right = Ok((
            "",
            ExprKind::addition(
                ExprKind::multiplication(ExprKind::integer(10), ExprKind::integer(4)),
                ExprKind::integer(2),
            ),
        ));

        assert_eq!(left, right);
    }
}

#[cfg(test)]
mod if_else {
    use super::*;

    #[test]
    fn if_else_simple() {
        let left = if_else("if0{1}else{42}");
        let right = Ok((
            "",
            ExprKind::if_(
                ExprKind::integer(0),
                ExprKind::integer(1),
                ExprKind::integer(42),
            ),
        ));

        assert_eq!(left, right);
    }

    #[test]
    fn if_else_spaced_braces() {
        let left = if_else("if 0 { 1 } else { 42 }");
        let right = Ok((
            "",
            ExprKind::if_(
                ExprKind::integer(0),
                ExprKind::integer(1),
                ExprKind::integer(42),
            ),
        ));

        assert_eq!(left, right);
    }

    #[test]
    fn addition_as_condition() {
        let left = if_else("if 1 + 1 { 1 } else { 1 }");
        let right = Ok((
            "",
            ExprKind::if_(
                ExprKind::addition(ExprKind::integer(1), ExprKind::integer(1)),
                ExprKind::integer(1),
                ExprKind::integer(1),
            ),
        ));

        assert_eq!(left, right);
    }
}

#[cfg(test)]
mod keyword {
    use super::*;

    #[test]
    fn parses() {
        let left = keyword("if")("if");
        let right = Ok(("", ()));

        assert_eq!(left, right);
    }

    #[test]
    fn fails_when_followed_by_letter() {
        assert!(keyword("if")("iff").is_err());
    }

    #[test]
    fn works_when_followed_by_non_letter() {
        let left = keyword("if")("if42");
        let right = Ok(("42", ()));

        assert_eq!(left, right);
    }

    #[test]
    fn kw_followed_by_space_and_letter() {
        assert!(keyword("let")("let a").is_ok());
    }
}

#[cfg(test)]
mod binding {
    use super::*;

    #[test]
    fn simple() {
        let left = binding("let a = 42;");
        let right = Ok(("", Binding::new("a".to_owned(), ExprKind::integer(42))));

        assert_eq!(left, right);
    }

    #[test]
    fn with_if_else() {
        let left = binding("let foo = if 5 { 42 } else { 101 };");
        let right = Ok((
            "",
            Binding::new(
                "foo".to_owned(),
                ExprKind::if_(
                    ExprKind::integer(5),
                    ExprKind::integer(42),
                    ExprKind::integer(101),
                ),
            ),
        ));

        assert_eq!(left, right);
    }
}

#[cfg(test)]
mod bindings {
    use super::*;

    #[test]
    fn bindings_simple() {
        let left = bindings("let a = 42; a");
        let right = Ok((
            "",
            ExprKind::single_binding(
                "a".to_owned(),
                ExprKind::integer(42),
                ExprKind::ident("a".to_owned()),
            ),
        ));

        assert_eq!(left, right);
    }
}
