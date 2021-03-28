use dyl_bytecode::Instruction;

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
        Instruction::push_i(42),
        Instruction::f_stop(),
    } => |rslt| assert_eq!(rslt.unwrap(), Value::Integer(42)),
}

test_bytecode_execution! {
    add_i_simple :: {
        Instruction::push_i(40),
        Instruction::push_i(1),
        Instruction::push_i(1),
        Instruction::add_i(),
        Instruction::add_i(),
        Instruction::f_stop(),
    } => |rslt| assert_eq!(rslt.unwrap(), Value::Integer(42)),
}

test_bytecode_execution! {
    function_simple :: {
        Instruction::res_v(1),    //                     |  0 |
        Instruction::push_i(41),  //                | 41 |  0 |
        Instruction::call(4),     //           | IP | 41 |  0 |
        Instruction::f_stop(),    //                     | 42 |

        Instruction::push_cpy(1), //      | 41 | IP | 41 |  0 |
        Instruction::push_i(1),   // |  1 | 41 | IP | 41 |  0 |
        Instruction::add_i(),     //      | 42 | IP | 41 |  0 |
        Instruction::pop_cpy(3),  //           | IP | 41 | 42 |
        Instruction::ret(0, 2),   //                     | 42 |
    } => |rslt| assert_eq!(rslt.unwrap(), Value::Integer(42)),
}
