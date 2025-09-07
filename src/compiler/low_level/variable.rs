use std::ops::Deref;
use crate::compiler::low_level::arch::arch::Arch;
use crate::compiler::low_level::data_position::DataPosition;
use crate::util::exit::{exit, ExitCode};

#[derive(Clone)]
pub struct Variable {
    pub full_name: String,              // The full name of the variable (e.g. my_app:main.rsl:Main:loop1:myVar)
    pub positions: Vec<DataPosition>    // All the positions the data position is currently stored in (might be in a register and on the stack at the same time)
}

impl Variable {
    pub fn new(full_name: String, positions: Vec<DataPosition>) -> Variable {
        Variable { full_name, positions }
    }

    pub fn get_cheapest_position(&self) -> Option<DataPosition> {
        let mut cheapest_position: Option<DataPosition> = None;

        for position in &self.positions {
            let position = position.clone();

            if cheapest_position.is_none() || cheapest_position.clone().unwrap().cost() > position.cost() {
                cheapest_position = Some(position);
            }
        }

        cheapest_position
    }

    pub fn has_stack_position(&self) -> bool {
        for position in &self.positions {
            let position = position.clone();

            match position{
                DataPosition::StackOffset(_) | DataPosition::StackOffsetAt(_) => return true,
                _ => continue
            }
        }
        false
    }

    /// Remove every location but one location on the stack.
    /// If there are multiple locations on the stack, the most direct one (without a non-immediate value referring to its location) will be chosen.
    pub fn remove_everything_except_stack_position(&mut self) {
        let mut stack_position: Option<DataPosition> = None;

        for position in &self.positions {
            let position = position.clone();

            match position {
                DataPosition::StackOffset(_) => {stack_position = Some(position);}
                DataPosition::StackOffsetAt(offset) => {
                    // Replace the currently selected stack position if the new position's cost is smaller than the currently selected stack position's or
                    // the currently selected stack position is simply non-existent.
                    if stack_position.is_none() || offset.cost() < stack_position.clone().unwrap().cost() {
                        stack_position = Some(DataPosition::StackOffsetAt(offset));
                    }
                }
                _ => continue
            }
        }

        // Finally, replace all the positions with the stack position (if it exists)
        // or nothing if it's non-existent.
        if stack_position.is_some() {
            self.positions = vec![stack_position.unwrap()];
        }else{
            self.positions = vec![];
        }
    }

    pub fn get_stack_offset(&self) -> Option<usize> {
        if !self.has_stack_position() { return None }

        let mut stack_position: Option<DataPosition> = None;

        for position in &self.positions {
            let position = position.clone();

            match position {
                DataPosition::StackOffset(_) => { stack_position = Some(position); }
                DataPosition::StackOffsetAt(offset) => {
                    // Replace the currently selected stack position if the new position's cost is smaller than the currently selected stack position's or
                    // the currently selected stack position is simply non-existent.
                    if stack_position.is_none() || offset.cost() < stack_position.clone().unwrap().cost() {
                        stack_position = Some(DataPosition::StackOffsetAt(offset));
                    }
                }
                _ => continue
            }
        }

        if stack_position.is_none() { return None }

        stack_position.unwrap().immediate_stack_offset()
    }
}


#[derive(Clone)]
pub enum BitUnit {
    Byte,
    Word,
    DoubleWord,
    QuadWord,
    ArchitectureMax, // The maximum the architecture allows (usually 32/64 b)
}

impl BitUnit {
    fn resolve(&self, arch: Box<dyn Arch>) -> BitUnit {
        if !matches!(self, BitUnit::ArchitectureMax){
            // Not architecture dependent, just return the unit itself
            return (*self).clone();
        }

        // ArchitectureMax option, just use the max of the architecture

        let arch = arch.deref();

        let arch_max_bits = arch.architecture_bits();

        match arch_max_bits {
            8 => BitUnit::Byte,
            16 => BitUnit::Word,
            32 => BitUnit::DoubleWord,
            64 => BitUnit::QuadWord,
            _ => {
                exit(format!("Architecture \"{}\" set {} as the max amount of bits a register can store which is not a valid amount (8, 16, 32, 64),", arch.name(), arch_max_bits), ExitCode::Internal);
            }
        }
    }
}