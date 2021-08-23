use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Ty {
    Bool,
    Int,

    Err,
}

impl Ty {
    pub(crate) fn unify_with(self, other: Ty) -> Result<Ty, UnificationError> {
        match (self, other) {
            (Ty::Err, Ty::Err) => Ok(Ty::Err),
            (Ty::Err, other) => Ok(other),
            (this, Ty::Err) => Ok(this),
            (lhs, rhs) if lhs == rhs => Ok(lhs),

            (left, right) => Err(UnificationError { left, right }),
        }
    }

    pub(crate) fn expect_bool(&self) -> Result<(), UnexpectedTypeError> {
        self.expect(&Ty::Bool)
    }

    pub(crate) fn expect_int(&self) -> Result<(), UnexpectedTypeError> {
        self.expect(&Ty::Int)
    }

    #[inline]
    fn expect(&self, expected: &Ty) -> Result<(), UnexpectedTypeError> {
        match (self, expected) {
            (lhs, rhs) if lhs == rhs => Ok(()),
            (Ty::Err, _) => Ok(()),

            _ => Err(UnexpectedTypeError {
                expected: expected.clone(),
                got: self.clone(),
            }),
        }
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Ty::Bool => "bool",
            Ty::Int => "int",

            Ty::Err => "{type error}",
        }
            .fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UnificationError {
    pub(crate) left: Ty,
    pub(crate) right: Ty,
}

impl Display for UnificationError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Expression returns two different types: `{}` and `{}`",
            self.left, self.right
        )
    }
}

impl Error for UnificationError {}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UnexpectedTypeError {
    pub(crate) expected: Ty,
    pub(crate) got: Ty,
}

impl Display for UnexpectedTypeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Expected type `{}`, found type `{}`",
            self.expected, self.got
        )
    }
}

impl Error for UnexpectedTypeError {}
