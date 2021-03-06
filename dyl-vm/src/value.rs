use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use anyhow::{bail, Result};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    Integer(i32),
    InstructionPointer(u32),
}

impl Value {
    pub(crate) fn try_into_integer(self) -> Result<i32> {
        match self {
            Value::Integer(val) => Ok(val),
            anything => bail!(ValueConversionError {
                expected_type: Type::Integer,
                found_value: anything,
            }),
        }
    }

    pub(crate) fn try_into_instruction_pointer(self) -> Result<u32> {
        match self {
            Value::InstructionPointer(ip) => Ok(ip),
            anything => bail!(ValueConversionError {
                expected_type: Type::InstructionPointer,
                found_value: anything,
            }),
        }
    }

    fn type_(&self) -> Type {
        match self {
            Value::Integer(_) => Type::Integer,
            Value::InstructionPointer(_) => Type::InstructionPointer,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::InstructionPointer(ip) => write!(f, "*{}*", ip),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Type {
    Integer,
    InstructionPointer,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Type::Integer => write!(f, "integer"),
            Type::InstructionPointer => write!(f, "instruction pointer"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ValueConversionError {
    expected_type: Type,
    found_value: Value,
}

impl Display for ValueConversionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Expected value of type `{}`, found value `{}` of type `{}`",
            self.expected_type,
            self.found_value,
            self.found_value.type_(),
        )
    }
}

impl Error for ValueConversionError {}
