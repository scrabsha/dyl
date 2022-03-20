#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Program {
    functions: Vec<Function>,
}

impl Program {
    pub(crate) fn new(functions: Vec<Function>) -> Program {
        Program { functions }
    }

    pub(crate) fn functions(&self) -> &[Function] {
        self.functions.as_slice()
    }

    #[cfg(test)]
    pub(crate) fn just_main(body: ExprKind) -> Program {
        let f = Function {
            name: "main".to_string(),
            body,
        };
        let functions = vec![f];

        Program { functions }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Function {
    name: String,
    body: ExprKind,
}

impl Function {
    pub(crate) fn new(name: String, body: ExprKind) -> Function {
        Function { name, body }
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) fn body(&self) -> &ExprKind {
        &self.body
    }

    #[cfg(test)]
    pub(crate) fn main_(body: ExprKind) -> Function {
        Function {
            name: "main".to_string(),
            body,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ExprKind {
    Addition(Addition),
    Subtraction(Subtraction),
    Multiplication(Multiplication),
    Integer(Integer),
    If(If),
    Bindings(Bindings),
    Ident(Ident),
    Bool(Bool),
}

impl ExprKind {
    pub(crate) fn addition(lhs: ExprKind, rhs: ExprKind) -> ExprKind {
        ExprKind::Addition(Addition::new(lhs, rhs))
    }

    pub(crate) fn subtraction(lhs: ExprKind, rhs: ExprKind) -> ExprKind {
        ExprKind::Subtraction(Subtraction::new(lhs, rhs))
    }

    pub(crate) fn multiplication(lhs: ExprKind, rhs: ExprKind) -> ExprKind {
        ExprKind::Multiplication(Multiplication::new(lhs, rhs))
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

    pub(crate) fn bindings(bs: Vec<Binding>, next: ExprKind) -> ExprKind {
        ExprKind::Bindings(Bindings::from_vec(bs, next))
    }

    pub(crate) fn ident(name: String) -> ExprKind {
        ExprKind::Ident(Ident::new(name))
    }

    pub(crate) fn bool_(bool_: bool) -> ExprKind {
        ExprKind::Bool(Bool::new(bool_))
    }
}

#[cfg(test)]
impl ExprKind {
    pub(crate) fn single_binding(
        name: String,
        value: ExprKind,
        inner_expression: ExprKind,
    ) -> ExprKind {
        ExprKind::Bindings(Bindings::single(name, value, inner_expression))
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Multiplication(Box<(ExprKind, ExprKind)>);

impl Multiplication {
    pub(crate) fn new(lhs: ExprKind, rhs: ExprKind) -> Multiplication {
        Multiplication(Box::new((lhs, rhs)))
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Bindings(Vec<Binding>, Box<ExprKind>);

impl Bindings {
    pub(crate) fn from_vec(bs: Vec<Binding>, next: ExprKind) -> Bindings {
        Bindings(bs, Box::new(next))
    }

    pub(crate) fn defines(&self) -> &[Binding] {
        self.0.as_slice()
    }

    pub(crate) fn ending_expression(&self) -> &ExprKind {
        &self.1
    }
}

#[cfg(test)]
impl Bindings {
    pub(crate) fn single(name: String, value: ExprKind, next: ExprKind) -> Bindings {
        let binding = Binding::new(name, value);
        Bindings(vec![binding], Box::new(next))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Binding(String, ExprKind);

impl Binding {
    pub(crate) fn new(name: String, value: ExprKind) -> Binding {
        Binding(name, value)
    }

    pub(crate) fn name(&self) -> &str {
        self.0.as_str()
    }

    pub(crate) fn value(&self) -> &ExprKind {
        &self.1
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Ident(String);

impl Ident {
    pub(crate) fn new(name: String) -> Ident {
        Ident(name)
    }

    pub(crate) fn name(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Bool(bool);

impl Bool {
    pub(crate) fn new(bool_: bool) -> Bool {
        Bool(bool_)
    }

    pub(crate) fn value(&self) -> bool {
        self.0
    }
}
