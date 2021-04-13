#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ExprKind {
    Addition(Addition),
    Integer(Integer),
}

impl ExprKind {
    pub(crate) fn addition(lhs: ExprKind, rhs: ExprKind) -> ExprKind {
        ExprKind::Addition(Addition::new(lhs, rhs))
    }

    pub(crate) fn integer(value: i32) -> ExprKind {
        ExprKind::Integer(Integer::new(value))
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
