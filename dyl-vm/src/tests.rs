use dyl_bytecode::Instruction;

use crate::interpreter::Interpreter;
use crate::value::Value;

macro_rules! generate_bytecode {
    (@internal($acc:ident, $val:expr) {}) => {};

    (@internal($acc:ident, $val:expr) { $label:ident: $( $tail:tt)* }) => {
        const $label:u32 = $val;
        generate_bytecode! { @internal($acc, $val) { $( $tail )* } }
    };

    (@internal($acc:ident, $val:expr) { push_i $n:literal $( $tail:tt )* }) => {
        $acc.push(dyl_bytecode::Instruction::push_i($n));
        generate_bytecode! { @internal($acc, $val + 1) { $( $tail )* } }
    };

    (@internal($acc:ident, $val:expr) { add_i $( $tail:tt )* }) => {
        $acc.push(dyl_bytecode::Instruction::add_i());
        generate_bytecode! { @internal($acc, $val + 1) { $( $tail )* } }
    };

    (@internal($acc:ident, $val:expr) { f_stop $( $tail:tt )* }) => {
        $acc.push(dyl_bytecode::Instruction::f_stop());
        generate_bytecode! { @internal($acc, $val + 1) { $( $tail )* } }
    };

    (@internal($acc:ident, $val:expr) { push_cpy $idx:literal $( $tail:tt )* }) => {
        $acc.push(dyl_bytecode::Instruction::push_cpy($idx));
        generate_bytecode! { @internal($acc, $val + 1) { $( $tail )* } }
    };

    (@internal($acc:ident, $val:expr) { call $label:ident $( $tail:tt )* }) => {
        $acc.push(dyl_bytecode::Instruction::call($label));
        generate_bytecode! { @internal($acc, $val + 1) { $( $tail )* } }
    };

    (@internal($acc:ident, $val:expr) { ret $stack_top:literal $ip_offset:literal $( $tail:tt )* }) => {
        $acc.push(dyl_bytecode::Instruction::ret($ip_offset, $stack_top));
        generate_bytecode! { @internal($acc, $val + 1) { $( $tail )* } }
    };

    (@internal($acc:ident, $val:expr) { res_v $idx:literal $( $tail:tt )* }) => {
        $acc.push(dyl_bytecode::Instruction::res_v($idx));
        generate_bytecode! { @internal($acc, $val + 1) { $( $tail )* } }
    };

    (@internal($acc:ident, $val:expr) { pop_cpy $idx:literal $( $tail:tt )* }) => {
        $acc.push(dyl_bytecode::Instruction::pop_cpy($idx));
        generate_bytecode! { @internal($acc, $val + 1) { $( $tail )* } }
    };

    (@internal($acc:ident, $val:expr) { goto $label:ident $( $tail:tt )* } ) => {
        $acc.push(dyl_bytecode::Instruction::goto($label));
        generate_bytecode! { @internal($acc, $val + 1) { $( $tail )* } }
    };

    ( $( $tail:tt )* ) => {{
        let mut acc = Vec::new();
        generate_bytecode! { @internal(acc, 0) { $( $tail )* } };
        acc
    }};
}

mod generate_bytecode {
    use super::*;

    #[test]
    fn push_i() {
        assert_eq!(
            generate_bytecode! { push_i 42 push_i 101 },
            [Instruction::push_i(42), Instruction::push_i(101)],
        )
    }

    #[test]
    fn add_i() {
        assert_eq!(generate_bytecode! { add_i }, [Instruction::add_i()],)
    }

    #[test]
    fn f_stop() {
        assert_eq!(generate_bytecode! { f_stop }, [Instruction::f_stop()],)
    }

    #[test]
    fn push_c() {
        assert_eq!(
            generate_bytecode! { push_cpy 2 push_cpy 4 },
            [Instruction::push_cpy(2), Instruction::push_cpy(4)],
        )
    }

    #[test]
    fn call_earlier_defined() {
        assert_eq!(
            generate_bytecode! {
                LABEL:
                    call LABEL
            },
            [Instruction::call(0)],
        );

        assert_eq!(
            generate_bytecode! {
                    add_i
                    add_i
                LABEL:
                    call LABEL
            },
            [
                Instruction::add_i(),
                Instruction::add_i(),
                Instruction::call(2),
            ],
        );
    }

    #[test]
    fn call_later_defined() {
        assert_eq!(
            generate_bytecode! {
                    call LABEL
                LABEL:
            },
            [Instruction::call(1)],
        );

        assert_eq!(
            generate_bytecode! {
                    call LABEL
                add_i
                add_i
                    LABEL:
            },
            [
                Instruction::call(3),
                Instruction::add_i(),
                Instruction::add_i(),
            ],
        );
    }

    #[test]
    fn ret() {
        assert_eq!(
            generate_bytecode! {
                ret 4 1
            },
            [Instruction::ret(1, 4)],
        );
    }

    #[test]
    fn res_v() {
        assert_eq!(
            generate_bytecode! {
                res_v 32
            },
            [Instruction::res_v(32)],
        )
    }

    #[test]
    fn pop_cpy() {
        assert_eq!(
            generate_bytecode! {
                pop_cpy 3
            },
            [Instruction::pop_cpy(3)],
        )
    }
}

macro_rules! run_bytecode {
    ( $( $input:tt)* ) => {{
        let instrs = generate_bytecode! { $( $input )* };

        Interpreter::from_instructions(instrs).run()
    }};
}

macro_rules! test_bytecode_execution {
    ($test_name:ident :: { $( $instr:tt )* } = Ok($val:expr) $(,)?) => {
        #[test]
        fn $test_name() {
            let rslt = run_bytecode! { $( $instr )* };
            assert_eq!(rslt.unwrap(), $val);
        }
    };

    ($test_name:ident :: { $( $instr:tt )* } = Err($val:expr) $(,)?) => {
        #[test]
        fn $test_name() {
            let rslt = run_bytecode! { $( $instr )* };
            assert_eq!(rslt.unwrap_err(), $val);
        }
    };
}

test_bytecode_execution! {
    push_i_simple :: {
        push_i 42
        f_stop
    } = Ok(Value::Integer(42)),
}

test_bytecode_execution! {
    add_i_simple :: {
        push_i 40
        push_i 1
        push_i 1
        add_i
        add_i
        f_stop
    } = Ok(Value::Integer(42)),
}

test_bytecode_execution! {
    function_simple :: {
            res_v 1
            push_i 41
            call ADD_1
            f_stop

        ADD_1:
            push_cpy 1
            push_i 1
            add_i
            pop_cpy 3
            ret 2 0
    } = Ok(Value::Integer(42)),
}

test_bytecode_execution! {
    goto_simple :: {
            goto NEXT
        PREV:
            push_i 42
            f_stop
        NEXT:
            goto PREV
    } = Ok(Value::Integer(42)),
}
