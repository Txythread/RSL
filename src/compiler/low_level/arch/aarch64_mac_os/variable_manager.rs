use std::cmp::PartialEq;
use std::collections::HashMap;
use std::env::var;
use std::fmt::format;
use crate::compiler::low_level::arch::register::{Register, RegisterTag};
use crate::compiler::low_level::data_position::DataPosition;
use crate::compiler::low_level::macro_instruction::MacroInstruction;
use crate::compiler::low_level::variable::Variable;

pub fn find_place_for_variable(variable: &mut Variable, size: usize, other_variables: Vec<Variable>, instructions: Vec<MacroInstruction>){

}

pub fn order_variable_locations(variables: &mut Vec<Variable>, registers: Vec<Register>, instructions: Vec<MacroInstruction>, stack_offset: &mut usize) -> String{
    // Variables, where they should be, where they are and the inverse of the relevance they get to their target position (basically a bit like nice on unix-like systems)
    let mut variables_info: Vec<(Variable, DataPosition, usize)> = Vec::new();

    // The code that will be used to move the variables around
    let mut code = String::new();

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
                variables_info.push((variable.clone(), cheapest_position, usize::MAX));
                continue;
            }
            // If the last "if condition" did not return true, which is the case if the code here is getting executed,
            // the variable doesn't have an associated position, so it's a "stray".
            // Behaviour might be unexpected in other areas
            continue;
        }
        variables_info.push((variable.clone(), target_position.clone().unwrap(), target_distance.unwrap()));
    }

    // Sort by distance (lowest first)
    variables_info.sort_by(|a, b| a.clone().2.cmp(&b.clone().2));

    #[cfg(test)]
    for var_info in variables_info.clone(){
        println!("Name: {} \tCurrent Position: {:?} \tTarget Position: {:?} \tDistance: {}", var_info.0.full_name, var_info.0.positions.iter().nth(0), var_info.1, var_info.2)
    }

    // The newly generated mapping from registers to variables.
    // Basically variables_info but realistic (no registers being used multiple times)
    let mut register_to_variable_map: HashMap<Register, Variable> = HashMap::new();

    // All the variables that were in registers previously but don't fit anymore
    // and are to be stored in the stack now.
    let mut new_stack_items: Vec<Variable> = Vec::new();

    // The registers that are not in register_to_variable_map yet,
    // which means they can still be used to store variables to them.
    let mut available_registers: Vec<Register> = registers.clone();

    // Now, find the ideal realistic storage position for all variables
    // This means trying to match the recommendation variable_info
    // as best as possible (especially for all vars with a short distance)
    // while not storing multiple variables on the same register.
    for x in variables_info.clone().iter().enumerate(){
        let mut var_info = x.1.clone();
        let i = x.0;
        // If the variable should be stored in a register, look if there's place for it left somewhere
        // If not, let it live on the stack (or the heap for that matter) for now.
        // The only way the variable info says it should be stored on the stack is if it has
        // been stored on the stack previously, so there's no reason to even mark it for later.

        if !matches!(var_info.1, DataPosition::Register(_)){
            // This is not a register position,
            // continue (reasoning above).
            continue;
        }

        // Look if a specific register is requested or
        // just some general-purpose register
        if !var_info.1.is_general_purpose_register(){
            // The requested register from the available_registers list if it's still available, None otherwise.
            let register = available_registers.iter().find(|&x| x.name == var_info.1.register_name().unwrap());

            if let Some(register) = register {
                let register = register.clone();

                // The register is still available.
                // Remove it from the list and add it to register_to_var_map.

                let position_in_available_registers = available_registers.iter().position(|x| x.clone() == register).unwrap();
                available_registers.remove(position_in_available_registers);

                register_to_variable_map.insert(register, var_info.0.clone());
                continue;
            }
            // The requested register is not available, so look for a general purpose register instead.
            // This is done by just continuing to the next step, which would handel normal general-purpose register requests.
        }

        // Either there was nothing but a general purpose register needed,
        // or the real favourite failed (was overwritten by another reg)
        // Find the least-used general purpose register and go with that
        // or the stack (if there are no regs left).

        // The remaining registers and how much damage using them would cause
        // (0 being the most).
        let mut register_cost_map: Vec<(Register, usize)> = Vec::new();

        // Loop through all available general purpose registers.
        for available_register in available_registers.clone().iter().filter(|&x| x.tags.iter().map(|x|x.clone()).collect::<Vec<RegisterTag>>().contains(&RegisterTag::GeneralPurpose)){
            let available_register = available_register.clone();

            // Now calculate the cost of using this register by going through all variable infos and looking if it's used somewhere.
            // Usage found -> use distance as cost factor, if not use usize::MAX
            let mut costs = usize::MAX;

            for var_info in variables_info.clone().iter(){
                let var_target = var_info.clone().1;

                if let Some(target_name) = var_target.register_name(){
                    if target_name != available_register.name { continue; }

                    // Register ('available_register') is going to be used by this variable (or at least the variable wants to use this reg).
                    let distance = var_info.clone().2;
                    costs = distance;
                }
            }

            register_cost_map.push((available_register.clone(), costs));
        }

        // Sort so the register with the smallest distance is at the top
        register_cost_map.sort_by(|a, b| a.clone().1.cmp(&b.clone().1));

        // Get the last register (with the biggest distance) from the register_cost_map.
        // If it exists, use it, if not, there's no place left in the registers.
        // This means the stack will need to handle that.
        let most_cost_effective_reg = register_cost_map.last();

        if let Some(most_cost_effective_reg) = most_cost_effective_reg {
            // Remove the most_cost_effective_reg from the list of available registers as it's no longer going to be available
            let reg_position_in_available_registers = available_registers.iter().position(|x| x.clone() == most_cost_effective_reg.clone().0).unwrap();
            available_registers.remove(reg_position_in_available_registers);

            // Add the variable to the mapping
            _ = register_to_variable_map.insert(most_cost_effective_reg.clone().0, var_info.0.clone());

            // Continue as the rest of the code is about handling the case that there is no space left in the registers.
            continue;
        }

        // No place left in the registers, push to stack (if there is no alternative position (on the stack) already).

        // Check if there is a stack location already.
        if var_info.0.has_stack_position() {
            // Stack position already exists, so delete all locations but the stack position.
            // This is mainly to delete the register locations.
            variables_info[i].0.remove_everything_except_stack_position();

            // Continue as the rest of the code is there to push the variable to the stack as there's no other storage location and no space in the registers anymore.
            continue;
        }


        // Generate the location and update the stack offset (since function/stack frame start)
        let location = DataPosition::StackOffset(*stack_offset);
        *stack_offset += 8;

        // Update the location in the  variable
        var_info.0.positions = vec![location];

        // Mark this for actually adding the push code later.
        new_stack_items.push(var_info.clone().0);
    }

    // Push the data to the stack.

    // Data that had no previous location but is now new at the stack does not need to be allocated,
    // it just needs a reserved address, which it's already got,
    // but it still needs to have its stack position stored in the variables variable

    // Data is best stored in pairs, as this is more efficient on the aarch64 architecture.
    // Keep info about one variable to be able to push in pairs where possible
    // Store the variable (with its new location in the positions field) and the register it has been stored at previously
    let mut pair_first_part: Option<(Variable, Register)> = None;

    // Go through all the data from the stack
    for stack_variable in new_stack_items.clone(){
        // The variable as it's been in its previous state (with the old positions).
        let original_variable = variables.iter().find(|&var| var.full_name == stack_variable.full_name);

        if original_variable.is_none() { continue; }

        let original_variable = original_variable.unwrap();

        // If the original variable wasn't stored in a register, skip it (as explained above).
        let least_costy_position = original_variable.get_cheapest_position();

        if least_costy_position.is_none() { continue; }
        let least_costy_position = least_costy_position.unwrap();
        if !matches!(least_costy_position, DataPosition::Register(_)) {
            // Update the positions in variables.
            // Up to now, the "variables" variable still has the old position.
            // As those positions are now outdated, they should be overwritten.
            // This happens by first "retracing" the position in said variable, ...
            let position_in_variables = variables.iter().position(|x| x.clone().full_name == stack_variable.full_name).unwrap();
            // ... and then changing it right there
            variables[position_in_variables].positions = stack_variable.positions.clone();
            continue;
        }

        let current_register = registers.iter().find(|&x|x.name == least_costy_position.register_name().unwrap());
        if current_register.is_none() { panic!("Register {} not found though there's a value that claims to have been stored in it.", least_costy_position.register_name().unwrap())}
        let current_register = current_register.unwrap();

        // Try to make pairs as explained above
        if pair_first_part.is_none() {
            pair_first_part = Some((stack_variable.clone(), current_register.clone()));

            // Continue as the rest of the code is for actually storing the pair.
            continue;
        }

        // Store the pair if possible.
        // Storing the pair is impossible if their expected storing positions are not immediately after each other.
        // This could happen if a new value has been allocated in between.
        // If there are 8B in between the storage positions of those two variables,
        // they are after each other for sure -> store them as a pair
        let storage_position_difference = (pair_first_part.clone().unwrap().0.get_stack_offset().unwrap() as isize) - stack_variable.get_stack_offset().unwrap() as isize;

        // Also generate the register of the second part

        {
            let stack_offset1 = stack_variable.get_stack_offset().unwrap();
            let reg_name1 = current_register.clone().name;
            let stack_offset2 = pair_first_part.clone().unwrap().0.get_stack_offset().unwrap();
            let reg_name2 = pair_first_part.clone().unwrap().1.name;

            println!("Reg {} should be stored at {}. Reg {} should be stored at {}", reg_name1, stack_offset1, reg_name2, stack_offset2);
        }
        // Check difference as detailed above
        if storage_position_difference == 8 {
            // Store in order: second_pair_part, first_pair_part at the position of first_pair_part
            let stack_offset = stack_variable.get_stack_offset().unwrap();
            code += format!("stp\t{}, {}, [sp, #{}]\n", current_register.name, pair_first_part.clone().unwrap().1.name, stack_offset).as_str();
        }else if storage_position_difference == -8 {
            // Store in reverse order
            let stack_offset = pair_first_part.clone().unwrap().0.get_stack_offset().unwrap();
            code += format!("stp\t{}, {}, [sp, #{}]\n", pair_first_part.clone().unwrap().1.name, current_register.name, stack_offset).as_str();
        }else {
            // Store pair_first_part
            {
                let stack_offset = pair_first_part.clone().unwrap().0.get_stack_offset().unwrap();
                code += format!("str\t{}, [sp, #{}]", pair_first_part.clone().unwrap().1.name, stack_offset).as_str();
            }
            // Store the second pair part
            {
                let stack_offset = stack_variable.get_stack_offset().unwrap();
                code += format!("str\t{}, [sp, #{}]", current_register.name, stack_offset).as_str();
            }
        }

        // Update the locations stored in the "variables" variable.
        // Now that the data is stored in its new position, the old position can be forgotten
        // without an issue.

        // Do this for the current one first
        {
            let position_in_variables = variables.iter().position(|x| x.clone().full_name == stack_variable.full_name).unwrap();
            variables[position_in_variables].positions = stack_variable.positions.clone();
        }

        // Now the same for the first_pair_part
        {
            let position_in_variables = variables.iter().position(|x| x.clone().full_name == pair_first_part.clone().unwrap().0.full_name).unwrap();
            variables[position_in_variables].positions = pair_first_part.clone().unwrap().0.positions.clone();
        }

        // Reset the pair_first_part bc it's been handled already.
        pair_first_part = None;
    }

    // Optionally push the last variable to the stack (from the variables that need to be on the stack), if it exists
    // (which happens if the amount of variables that need to be pushed to the stack is uneven).
    if let Some(variable) = pair_first_part {
        let position_in_variables = variables.iter().position(|x| x.clone().full_name == variable.0.full_name).unwrap();
        let target_stack_position = variable.0.get_stack_offset().unwrap();
        let start_register_name = variable.1.name;
        variables[position_in_variables].positions = variables[position_in_variables].positions.clone();

        code += format!("str\t{}, [sp, #{}]\n", start_register_name, target_stack_position).as_str();

        // Just to be sure, IDK what I'll code later
        pair_first_part = None;
    }


    // Find the discrepancy between the variables stored in registers.
    // Ignore all the variables that should remain in the same position and the ones that
    // have been stored to the stack.
    // Only the original register and the target register need to be stored, everything else can be discarded
    let mut changed_variables: Vec<(Register, Register)> = Vec::new();

    // The variables that are currently on stack but need to be stored in a register.
    // This stores both stack offset (in frame) and the target register.
    let mut variables_from_stack: Vec<(usize, Register)> = Vec::new();

    for i in register_to_variable_map {
        let variable = i.1.clone();
        let current_position = variable.get_cheapest_position();
        let target_register = i.0.clone();

        let target_position = DataPosition::Register(target_register.clone().name);

        if current_position == Some(target_position) {
            // The variable is in its target position already,
            // no further action required
            continue;
        }

        // The target position is both:
        // * a register
        // * not the original position

        // Look if the original position exists, if not, there is no further action required
        // as the space is already reserved.
        if current_position.is_none() { continue; }
        let current_position = current_position.unwrap();

        // Check if the variable is currently in the stack or in a register
        if let Some(current_stack_offset) = current_position.immediate_stack_offset(){
            // It's currently on the stack but needs to be in a register.
            // Append it to the array to take care of it later.
            variables_from_stack.push((current_stack_offset, target_register.clone()));
            continue;
        }

        // It's currently in register but needs to be in another register.

        let current_register = registers.iter().find(|&x|x.name == variable.get_cheapest_position().unwrap().register_name().unwrap()).unwrap().clone();

        changed_variables.push((current_register, target_register.clone()));
    }

    /// Generate assembly for changing the position of a variable from one register to another (and do so recursively if there's still data on the "target register").
    /// When "max iterations" is reached, the data is stored in a scratch register instead.
    fn recursively_move_registers(changed_variables: &mut Vec<(Register, Register)>, all_variables: &mut Vec<Variable>, pos_in_changed_vars: usize, code: &mut String, registers: Vec<Register>) {
        let current_variable = changed_variables[pos_in_changed_vars].clone();
        let current_variable_current_pos = current_variable.0;
        let current_variable_target_pos = current_variable.1;

        println!("Finding variable from {}", current_variable_current_pos.name.clone());
        let current_variable_pos_in_changed_variables = changed_variables.iter().position(|x| x.clone().0.name == current_variable_current_pos.name).unwrap();
        let current_variable_pos_in_variables = all_variables.iter().position(|x|x.clone().get_cheapest_position().unwrap().register_name().unwrap() == current_variable_current_pos.name).unwrap();

        // Find data that is stored in the target register.
        // This would mean the other variable's current position is the same as this variable's target position.
        let annoying_variable = changed_variables.iter().find(|&x| x.clone().0.name == current_variable_target_pos.name);

        // In case the annoying variable doesn't exist,
        // the contents of the (target) register can just be overwritten ruthlessly.
        if annoying_variable.is_none() {
            // Just generate assembly for pushing
            *code += format!("mov\t{}, {}\n", current_variable_target_pos.name, current_variable_current_pos.name).as_str();

            // Remove the variable from the list of changed variables
            changed_variables.remove(current_variable_pos_in_changed_variables);
            let current_variable_pos_in_variables = all_variables.iter().position(|x|x.clone().get_cheapest_position().unwrap().register_name().unwrap() == current_variable_current_pos.name).unwrap();
            all_variables[current_variable_pos_in_variables].positions = vec![DataPosition::Register(current_variable_target_pos.name)];
            return;
        }

        let annoying_variable = annoying_variable.unwrap().clone();
        let annoying_variable_pos_in_changed_variables = changed_variables.iter().position(|x| x.clone().0.name == annoying_variable.0.name).unwrap();
        let annoying_variable_pos_in_variables = changed_variables.iter().position(|x| x.clone().0.name == annoying_variable.0.name).unwrap();

        // If the scratch register contains no (relevant) data right now,
        // overwrite it with the annoying variable.
        // Moving it to its actual target happens in later iterations.
        let scratch_register = registers.iter().find(|&x|x.clone().tags.contains(&RegisterTag::Scratch)).unwrap();
        let scratch_register_name = scratch_register.name.clone();
        let scratch_register_empty = changed_variables.iter().find(|&x| x.clone().0.name == scratch_register_name).is_none();


        if scratch_register_empty {
            // Move the other variable to the scratch register
            changed_variables[annoying_variable_pos_in_changed_variables].0 = scratch_register.clone();
            all_variables[annoying_variable_pos_in_variables].positions = vec![DataPosition::Register(scratch_register_name.clone())];
            *code += format!("mov\t{}, {}\n", scratch_register_name, annoying_variable.0.name).as_str();
            // Move the current variable to its destination
            all_variables[current_variable_pos_in_variables].positions = all_variables[current_variable_pos_in_variables].positions.clone();
            changed_variables.remove(current_variable_pos_in_changed_variables);
            *code += format!("mov\t{}, {}\n", current_variable_target_pos.name, current_variable_current_pos.name).as_str();
        }
    }


    #[cfg(test)]
    for i in 0..variables.len(){
        println!("variables[{}] = (name: {}, current-pos: {:?})", i, variables[i].full_name, variables[i].get_cheapest_position());
    }

    // TODO: The upper method must be executed in a WHILE loop until the changed variables variable reaches zero
    let mut i: usize = 0;
    while changed_variables.len() > 0 {
        let changed_variables_length = changed_variables.len();
        recursively_move_registers(&mut changed_variables, &mut *variables, i % changed_variables_length, &mut code, registers.clone());
        i += 1;
    }

    code
}

#[cfg(test)]
mod tests{
    use crate::compiler::low_level::arch::aarch64_mac_os::aarch64_mac_os::AArch64MacOs;
    use crate::compiler::low_level::arch::aarch64_mac_os::variable_manager::order_variable_locations;
    use crate::compiler::low_level::arch::register::RegisterSaver;
    use crate::compiler::low_level::data_position::DataPosition;
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
        let mut var3 = Variable::new("var-3".to_string(), vec![]);

        let mut instructions: Vec<MacroInstruction> = vec![
            MacroInstruction::UseVariableAsArgument(var1.clone(), 0),
            MacroInstruction::UseVariableAsArgument(var2.clone(), 1),
            MacroInstruction::CallFunction("_malloc".to_string(), 2),
            MacroInstruction::UseVariableAsArgument(var3.clone(), 0),
            MacroInstruction::DestroyVariable(var1.clone()),
            MacroInstruction::DestroyVariable(var2.clone()),
        ];

        variables.push(var1.clone());
        variables.push(var2.clone());
        variables.push(var3.clone());

        for i in 4..21 {
            let variable = Variable::new(format!("var-{}", i), vec![]);

            instructions.insert(4, MacroInstruction::UseVariableAsArgument(variable.clone(), 0));

            variables.push(variable.clone());
        }


        println!("{}", order_variable_locations(&mut variables, aarch64_regs, instructions, &mut stack_offset));
    }

    #[test]
    fn other_test(){
        let mut aarch64 = AArch64MacOs::new();
        let aarch64_regs = aarch64.registers;

        let var_1 = Variable::new("var-1".to_string(), vec![DataPosition::Register("x0".to_string())]);
        let var_2 = Variable::new("var-2".to_string(), vec![DataPosition::Register("x1".to_string())]);

        let mut instructions: Vec<MacroInstruction> = vec![
            MacroInstruction::UseVariableAsArgument(var_2.clone(), 0),
            MacroInstruction::UseVariableAsArgument(var_1.clone(), 1),
            MacroInstruction::CallFunction("_malloc".to_string(), 2),
        ];

        let mut variables: Vec<Variable> = vec![var_1.clone(), var_2.clone()];
        let mut stack_offset: usize = 0;

        println!("{}", order_variable_locations(&mut variables, aarch64_regs, instructions, &mut stack_offset));
    }
}