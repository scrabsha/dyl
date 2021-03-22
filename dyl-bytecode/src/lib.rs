pub mod decode;
pub mod display;

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    PushI(i32),
    AddI,
    FullStop,
    PushC(char),
    CopyV(u32),
}
