#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Ty {
    Bool,
    Int,

    Err,
}

impl Ty {
    pub(crate) fn unify_with(self, other: Ty) -> Result<Ty, ()> {
        match (self, other) {
            (Ty::Err, Ty::Err) => Ok(Ty::Err),
            (Ty::Err, other) => Ok(other),
            (this, Ty::Err) => Ok(this),
            (lhs, rhs) if lhs == rhs => Ok(lhs),

            _ => Err(()),
        }
    }

    pub(crate) fn expect_bool(&self) -> Result<(), ()> {
        self.expect(&Ty::Bool)
    }

    pub(crate) fn expect_int(&self) -> Result<(), ()> {
        self.expect(&Ty::Int)
    }

    #[inline]
    fn expect(&self, expected: &Ty) -> Result<(), ()> {
        match (self, expected) {
            (lhs, rhs) if lhs == rhs => Ok(()),
            (Ty::Err, _) => Ok(()),

            _ => Err(()),
        }
    }
}
