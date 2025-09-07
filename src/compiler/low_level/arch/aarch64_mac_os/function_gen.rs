use std::ops::Deref;
use crate::compiler::low_level::arch::aarch64_mac_os::aarch64_mac_os::AArch64MacOs;
use crate::compiler::low_level::arch::register::RegisterSaver;
use crate::compiler::low_level::data_position::DataPosition::Register;
use crate::compiler::low_level::macro_instruction::MacroInstruction;
use crate::compiler::low_level::variable::Variable;

impl AArch64MacOs {
    fn generate_assembly(&self, macro_instructions: Vec<MacroInstruction>) -> String {
        let mut assembly = "".to_string();

        let mut uncompleted_instructions = macro_instructions.iter().clone().collect::<Vec<_>>();
        let mut alive_variables: Vec<Variable> = Vec::new();

        // Add reserved space for a register that serves as an intermediate register during some arithmetic operations
        // x8 is perfect for this as it's a scratch register
        // and its contents don't need to preserved after a function call anyway
        alive_variables.push(Variable::new("arithmetic_reserve".to_string(), vec![Register("x8".to_string())]));

        // Make sure all callee-preserved registers get stored somewhere
        for register in self.registers.iter().filter(|&x| matches!(x.clone().saver, RegisterSaver::Callee)){
            let register = register.clone();

            alive_variables.push(Variable::new(format!("saved-register-{}", register.name), vec![Register(register.name.clone())]));
        }

        // The amount of bytes allocated to the stack since the start of the current subroutine
        let mut stack_offset_since_function_start: u32 = 0;

        while uncompleted_instructions.len() > 0 {
            let current_instruction = (*uncompleted_instructions[0]).clone();

            // Drop the current instruction
            uncompleted_instructions.remove(0);

            // Generate assembly for current instruction
           /* match current_instruction {
                MacroInstruction::DeclareVariable(size, variable) => {
                    let mut variable = variable.clone();

                    let place = find_place_for_variable(&mut variable, size, alive_variables.clone());
                }
                MacroInstruction::DestroyVariable(_, _) => {}
            }*/
        }

        assembly
    }
}