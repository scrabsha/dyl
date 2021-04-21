use crate::operations::Operation;
use crate::Instruction;

impl Instruction {
    pub fn encode(&self, encoder: &mut Vec<u8>) {
        match self {
            Instruction::PushI(op) => op.encode(encoder),
            Instruction::AddI(op) => op.encode(encoder),
            Instruction::FStop(op) => op.encode(encoder),
            Instruction::PushCopy(op) => op.encode(encoder),
            Instruction::Call(op) => op.encode(encoder),
            Instruction::Ret(op) => op.encode(encoder),
            Instruction::ResV(op) => op.encode(encoder),
            Instruction::PopCopy(op) => op.encode(encoder),
            Instruction::Goto(op) => op.encode(encoder),
            Instruction::CondJmp(op) => op.encode(encoder),
            Instruction::Neg(op) => op.encode(encoder),
            Instruction::Mul(op) => op.encode(encoder),
        }
    }

    pub fn encode_multiple<'a, I>(instructions: I) -> Vec<u8>
    where
        I: IntoIterator<Item = &'a Instruction>,
    {
        let mut buff = Vec::new();
        instructions.into_iter().for_each(|i| i.encode(&mut buff));

        buff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_instructions_simple() {
        let instructions = [
            Instruction::push_i(42),
            Instruction::push_i(101),
            Instruction::add_i(),
        ];

        let left = Instruction::encode_multiple(&instructions);
        let right = [
            0, 0, 0, 0, 42, // push_i 42
            0, 0, 0, 0, 101, // push_i 101
            1,   // add_i
        ];
        assert_eq!(left, right);
    }
}
