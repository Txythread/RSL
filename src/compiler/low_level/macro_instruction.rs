use crate::compiler::low_level::variable::Variable;

#[derive(Clone)]
pub enum MacroInstruction {
    DeclareVariable(Variable),                      // Declare a variable exists and potentially reserve space on the stack/heap
    DestroyVariable(Variable),

    UseVariableAsArgument(Variable, usize),
    CallFunction(/*name: */String, /*argument_count: */usize),
    GetArgument(/*n-th argument n=*/usize)
}