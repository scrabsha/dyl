#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    Integer(i32),
}

impl Value {
    pub(crate) fn try_into_integer(self) -> Result<i32, Value> {
        match self {
            Value::Integer(val) => Ok(val),
            anything => Err(anything),
        }
    }
}