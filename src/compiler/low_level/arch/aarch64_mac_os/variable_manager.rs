use crate::compiler::low_level::arch::register::{Register, RegisterTag};
use crate::compiler::low_level::data_position::DataPosition;
use crate::compiler::low_level::macro_instruction::MacroInstruction;
use crate::compiler::low_level::variable::Variable;

pub fn find_place_for_variable(variable: &mut Variable, size: usize, other_variables: Vec<Variable>, instructions: Vec<MacroInstruction>){

}

pub fn order_variable_locations(variables: &mut Vec<Variable>, registers: Vec<Register>, instructions: Vec<MacroInstruction>, stack_offset: &mut usize) -> String{
    // Variables, where they should be, where they are and the inverse of the relevance they get to their target position (basically a bit like nice on unix-like systems)
    let mut variable_info: Vec<(Variable, DataPosition, usize)> = Vec::new();

    for variable in variables.clone(){
        let variable = variable.clone();

        println!("Creating info for {}", variable.full_name);

        let mut target_position: Option<DataPosition> = None;
        let mut target_distance: Option<usize> = None;

        for x in instructions.clone().iter().enumerate(){
            let instruction = x.1.clone();
            let distance = x.0;
            // Check if the variable is used there
            match instruction {
                MacroInstruction::CallFunction(_, arg_count) => {
                    // Check if this is in argument AND in the right position already
                    for i in 0..arg_count - 1{
                        // Get the name of the register that holds the i-th argument
                        let argument_register = registers.iter().find(|&register| register.is_argument(i as u8));

                        if argument_register.is_none() { continue; }

                        let argument_register_name = argument_register.unwrap().name.clone();

                        // Whether the variable is used in this exact argument & in its position already
                        let variable_is_argument = variable.positions.iter().find(|&pos| pos.clone().is_register(argument_register_name.clone())).is_some();

                        if variable_is_argument {
                            // Set the distance to zero, as changing it would be pure insanity now, I think
                            target_distance = Some(0);
                            target_position = Some(DataPosition::Register(argument_register_name));
                            break;
                        }
                    }
                }

                MacroInstruction::UseVariableAsArgument(searched_variable, arg_pos) => {
                    if searched_variable.full_name != variable.full_name { continue; }

                    target_distance = Some(distance);

                    let argument_register = registers.iter().find(|&register| register.is_argument(arg_pos as u8));

                    if let Some(argument_register) = argument_register {
                        // The argument does fit within the registers reserved for arguments
                        target_position = Some(DataPosition::Register(argument_register.name.clone()));
                        break;
                    }

                    // The data should be moved to the stack only when it's needed immediately.
                    if distance == 0{
                        // Set its target position to the stack
                        // Pass 0 as the offset (as the offset will be handled later
                        target_position = Some(DataPosition::StackOffset(0));
                        break;
                    }
                }

                MacroInstruction::DestroyVariable(searched_variable) => {
                    if distance != 0 { continue; }
                    if searched_variable.full_name != variable.full_name { continue; }

                    println!("Destroying variable {}", variable.full_name);
                    // Remove the variable from the list of variables, as it's useless now
                    // Just to be sure: all variables with that name
                    *variables = variables.iter().filter(|&var| var.full_name != searched_variable.full_name).map(|var| var.clone()).collect();

                    // TODO: Handle removing data from heap if necessary

                    break;
                }


                _ => continue
            }
        }

        if target_position.is_none() || target_distance.is_none() {
            // Still add it somewhere since the data should still be preserved.
            if let Some(cheapest_position) = variable.get_cheapest_position(){
                variable_info.push((variable.clone(), cheapest_position, usize::MAX));
                continue;
            }
            // If the last "if condition" did not return true, which is the case if the code here is getting executed,
            // the variable doesn't have an associated position, so it's a "stray".
            // Behaviour might be unexpected in other areas
        }
        variable_info.push((variable.clone(), target_position.clone().unwrap(), target_distance.unwrap()));
    }

    // Sort by distance (lowest first)
    variable_info.sort_by(|a, b| a.clone().2.cmp(&b.clone().2));

    for var_info in variable_info{
        println!("Name: {} \tCurrent Position: {:?} \tTarget Position: {:?} \tDistance: {}", var_info.0.full_name, var_info.0.positions.iter().nth(0), var_info.1, var_info.2)
    }

    "".to_string()
}

#[cfg(test)]
mod tests{
    use crate::compiler::low_level::arch::aarch64_mac_os::aarch64_mac_os::AArch64MacOs;
    use crate::compiler::low_level::arch::aarch64_mac_os::variable_manager::order_variable_locations;
    use crate::compiler::low_level::arch::register::RegisterSaver;
    use crate::compiler::low_level::data_position::DataPosition::Register;
    use crate::compiler::low_level::macro_instruction::MacroInstruction;
    use crate::compiler::low_level::variable::Variable;

    #[test]
    fn test_order_variable_locations(){
        println!();

        let mut aarch64 = AArch64MacOs::new();
        let aarch64_regs = aarch64.registers;

        let mut variables: Vec<Variable> = Vec::new();

        for register in aarch64_regs.iter().filter(|&x| matches!(x.clone().saver, RegisterSaver::Callee)){
            let register = register.clone();

            variables.push(Variable::new(format!("saved-register-{}", register.name), vec![Register(register.name.clone())]));
        }

        let mut stack_offset: usize = 0;

        let mut var1 = Variable::new("var-1".to_string(), vec![]);
        let mut var2 = Variable::new("var-2".to_string(), vec![]);

        let instructions: Vec<MacroInstruction> = vec![
            MacroInstruction::UseVariableAsArgument(var1.clone(), 0),
            MacroInstruction::UseVariableAsArgument(var2.clone(), 1),
            MacroInstruction::CallFunction("_malloc".to_string(), 2),
            MacroInstruction::DestroyVariable(var1.clone()),
            MacroInstruction::DestroyVariable(var2.clone()),
        ];

        variables.push(var1.clone());
        variables.push(var2.clone());

        _ = order_variable_locations(&mut variables, aarch64_regs, instructions, &mut stack_offset);
    }
}