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