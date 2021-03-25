use dyl_bytecode::Instruction::*;

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
        PushI(42),
        FullStop,
    } => |rslt| assert_eq!(rslt.unwrap(), Value::Integer(42)),
}

test_bytecode_execution! {
    add_i_simple :: {
        PushI(40),
        PushI(1),
        PushI(1),
        AddI,
        AddI,
        FullStop
    } => |rslt| assert_eq!(rslt.unwrap(), Value::Integer(42)),
}

test_bytecode_execution! {
    function_simple :: {
        ResV(1),   //                     |  0 |
        PushI(41), //                | 41 |  0 |
        Call(4),   //           | IP | 41 |  0 |
        FullStop,  //                     | 42 |

        CopyV(1),  //      | 41 | IP | 41 |  0 |
        PushI(1),  // |  1 | 41 | IP | 41 |  0 |
        AddI,      //      | 42 | IP | 41 |  0 |
        CopyVS(3), //           | IP | 41 | 42 |
        Ret { pointer_offset: 0, return_offset: 2 },
    } => |rslt| assert_eq!(rslt.unwrap(), Value::Integer(42)),
}
