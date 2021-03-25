pub mod decode;
pub mod display;

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    /// Pushes a constant integer on the stack
    ///
    /// ```none
    /// push(v)
    /// ```
    PushI(i32),

    /// Pops two integers from the stack, add them toghether, pushes the result
    ///
    /// ```none
    /// a = s.pop()
    /// b = s.pop()
    /// push(a + b)
    /// ```
    AddI,

    /// Stops the program, with s[0] as return value.
    FullStop,

    /// Pushes a constant character on the stack
    ///
    /// ```none
    /// push(c)
    /// ```

    PushC(char),

    /// Copies a value at a given index, pushes it on top of the stack.
    ///
    /// ```none
    /// a = get(idx)
    /// push(a)
    /// ```
    CopyV(u32),

    /// Pushes the current instruction pointer on the stack, sets the
    /// instruction pointer to the specified address.
    ///
    /// ```none
    /// push(ip)
    /// ip = ptr
    /// ```
    Call(u32),

    /// Copies the current instruction pointer at a specific stack offset,
    /// replaces it with a value on the stack, and jumps to the said
    /// instruction pointer.
    ///
    /// ```none
    /// ip = get(pointer)
    /// tmp = get(value)
    /// set(pointer, value)
    /// ```
    RetW {
        pointer_offset: u32,
        value_offset: u32,
    },

    /// Sets the instruction pointer to a value in the stack, shrinks the
    /// stack by a specific amount.
    ///
    /// ```none
    /// ip = get(pointer)
    /// shrink(len(stack) - return)
    /// ```
    Ret {
        return_offset: u32,
        pointer_offset: u32,
    },

    /// Pushes a constant amount of zeros in the stack.
    ///
    /// ```none
    /// for _ in 0..n {
    ///     push(0)
    /// }
    /// ```
    ResV(u32),

    /// Pops a value from the stack and copies it at a given stack index.
    ///
    /// ```none
    /// tmp = peek()
    /// set(index, tmp)
    /// pop()
    /// ```
    CopyVS(u32),
}
