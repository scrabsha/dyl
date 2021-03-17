pub mod decode;

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    PushI(i32),
    AddI,
}
