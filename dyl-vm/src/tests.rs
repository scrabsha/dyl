use dyl_bytecode::{Instruction, operations::{AddI, Call, FStop, PopCopy, PushCopy, PushI, ResV, Ret}};

use crate::interpreter::Interpreter;
use crate::value::Value;

macro_rules! run_bytecode {
    ($( $instr:expr ),* $(,)? ) => {{
        let instrs = vec![$($instr),*];

        Interpreter::from_instructions(instrs).run()
    }};
}

macro_rules! test_bytecode_execution {
    ($test_name:ident :: { $( $instr:expr ),* $(,)? } => |$out_value:ident| $assertions:expr $(,)?) => {
        #[test]
        fn $test_name() {
            let $out_value = run_bytecode! { $( $instr ),* };
            $assertions
        }
    };
}

test_bytecode_execution! {
    push_i_simple :: {
        Instruction::PushI(PushI(42)),
        Instruction::FStop(FStop),
    } => |rslt| assert_eq!(rslt.unwrap(), Value::Integer(42)),
}

test_bytecode_execution! {
    add_i_simple :: {
        Instruction::PushI(PushI(40)),
        Instruction::PushI(PushI(1)),
        Instruction::PushI(PushI(1)),
        Instruction::AddI(AddI),
        Instruction::AddI(AddI),
        Instruction::FStop(FStop),
    } => |rslt| assert_eq!(rslt.unwrap(), Value::Integer(42)),
}

test_bytecode_execution! {
    function_simple :: {
        Instruction::ResV(ResV(1)),         //                     |  0 |
        Instruction::PushI(PushI(41)),      //                | 41 |  0 |
        Instruction::Call(Call(4)),         //           | IP | 41 |  0 |
        Instruction::FStop(FStop),          //                     | 42 |

        Instruction::PushCopy(PushCopy(1)),  //      | 41 | IP | 41 |  0 |
        Instruction::PushI(PushI(1)),        // |  1 | 41 | IP | 41 |  0 |
        Instruction::AddI(AddI),             //      | 42 | IP | 41 |  0 |
        Instruction::PopCopy(PopCopy(3)),    //           | IP | 41 | 42 |
        Instruction::Ret(Ret { ip_offset: 0, shrink_offset: 2 }),
    } => |rslt| assert_eq!(rslt.unwrap(), Value::Integer(42)),
}
