#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ExprKind {
    Addition(Addition),
    Subtraction(Subtraction),
    Integer(Integer),
    If(If),
}

impl ExprKind {
    pub(crate) fn addition(lhs: ExprKind, rhs: ExprKind) -> ExprKind {
        ExprKind::Addition(Addition::new(lhs, rhs))
    }

    pub(crate) fn subtraction(lhs: ExprKind, rhs: ExprKind) -> ExprKind {
        ExprKind::Subtraction(Subtraction::new(lhs, rhs))
    }

    pub(crate) fn integer(value: i32) -> ExprKind {
        ExprKind::Integer(Integer::new(value))
    }

    pub(crate) fn if_(
        condition: ExprKind,
        consequent: ExprKind,
        alternative: ExprKind,
    ) -> ExprKind {
        ExprKind::If(If::new(condition, consequent, alternative))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Addition(Box<(ExprKind, ExprKind)>);

impl Addition {
    pub(crate) fn new(lhs: ExprKind, rhs: ExprKind) -> Addition {
        Addition(Box::new((lhs, rhs)))
    }

    pub(crate) fn left(&self) -> &ExprKind {
        &self.inner().0
    }

    pub(crate) fn right(&self) -> &ExprKind {
        &self.inner().1
    }

    fn inner(&self) -> &(ExprKind, ExprKind) {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Subtraction(Box<(ExprKind, ExprKind)>);

impl Subtraction {
    pub(crate) fn new(lhs: ExprKind, rhs: ExprKind) -> Subtraction {
        Subtraction(Box::new((lhs, rhs)))
    }

    pub(crate) fn left(&self) -> &ExprKind {
        &self.inner().0
    }

    pub(crate) fn right(&self) -> &ExprKind {
        &self.inner().1
    }

    fn inner(&self) -> &(ExprKind, ExprKind) {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Integer(i32);

impl Integer {
    pub(crate) fn new(value: i32) -> Integer {
        Integer(value)
    }

    pub(crate) fn value(&self) -> i32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct If(Box<(ExprKind, ExprKind, ExprKind)>);

impl If {
    pub(crate) fn new(condition: ExprKind, consequent: ExprKind, alternative: ExprKind) -> If {
        If(Box::new((condition, consequent, alternative)))
    }

    pub(crate) fn condition(&self) -> &ExprKind {
        &self.inner().0
    }

    pub(crate) fn consequent(&self) -> &ExprKind {
        &self.inner().1
    }

    pub(crate) fn alternative(&self) -> &ExprKind {
        &self.inner().2
    }

    fn inner(&self) -> &(ExprKind, ExprKind, ExprKind) {
        &self.0
    }
}
