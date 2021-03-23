pub mod decode;
pub mod display;

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    PushI(i32),
    AddI,
    FullStop,
    PushC(char),
    CopyV(u32),
    Call(u32),
    Return {
        pointer_offset: u32,
        value_offset: u32,
    },
}
