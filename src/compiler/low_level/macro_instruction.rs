use crate::compiler::low_level::variable::Variable;

pub enum MacroInstruction {
    DeallocateBits(usize),
    DeallocateBitsByVariable(String, String),

    DeclareVariable(usize, Variable),                      // Declare a variable exists and potentially reserve space on the stack/heap
    DestroyVariable(usize, Variable),
}