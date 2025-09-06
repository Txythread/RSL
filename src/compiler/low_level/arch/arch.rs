use crate::compiler::low_level::macro_instruction::MacroInstruction;

// The general definition and layout of every architecture

pub trait Arch{
    /// Get the full name of the architecture (like "aarch64-MacOS")
    fn name(&self) -> String;

    /// The amount of bits the "largest" register.
    fn architecture_bits(&self) -> u8;


    /// Generate assembly from the macro instructions in the given instruction set.
   fn generate_assembly(&self, macro_instructions: Vec<MacroInstruction>) -> String;
}