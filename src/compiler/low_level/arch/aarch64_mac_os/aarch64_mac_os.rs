use std::string::ToString;
use crate::compiler::low_level::arch::arch::Arch;
use crate::compiler::low_level::macro_instruction::MacroInstruction;
use crate::compiler::low_level::arch::register::*;
pub struct AArch64MacOs {
    pub registers: Vec<Register>
}

impl AArch64MacOs {
    pub const NAME: &'static str = "aarch64-mac-os";
    pub const BITS_COUNT: u8 = 64;

    pub fn new() -> Self {
        AArch64MacOs {
            registers: vec![
                Register::new("x0".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::Argument(0), RegisterTag::GeneralPurpose]),
                Register::new("x1".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::Argument(1), RegisterTag::GeneralPurpose]),
                Register::new("x2".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::Argument(2), RegisterTag::GeneralPurpose]),
                Register::new("x3".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::Argument(3), RegisterTag::GeneralPurpose]),
                Register::new("x4".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::Argument(4), RegisterTag::GeneralPurpose]),
                Register::new("x5".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::Argument(5), RegisterTag::GeneralPurpose]),
                Register::new("x6".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::Argument(6), RegisterTag::GeneralPurpose]),
                Register::new("x7".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::Argument(7), RegisterTag::GeneralPurpose]),
                Register::new("x8".to_string(), 64, RegisterSaver::None, vec![RegisterTag::Scratch]),
                Register::new("x9".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::GeneralPurpose]),
                Register::new("x10".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::GeneralPurpose]),
                Register::new("x11".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::GeneralPurpose]),
                Register::new("x12".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::GeneralPurpose]),
                Register::new("x13".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::GeneralPurpose]),
                Register::new("x14".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::GeneralPurpose]),
                Register::new("x15".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::GeneralPurpose]),
                Register::new("x16".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::GeneralPurpose]),
                Register::new("x17".to_string(), 64, RegisterSaver::Caller, vec![RegisterTag::GeneralPurpose]),
                Register::new("x18".to_string(), 64, RegisterSaver::OS, vec![RegisterTag::NoModify]),
                Register::new("x19".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose]),
                Register::new("x20".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose]),
                Register::new("x21".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose]),
                Register::new("x22".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose]),
                Register::new("x23".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose]),
                Register::new("x24".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose]),
                Register::new("x25".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose]),
                Register::new("x26".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose]),
                Register::new("x27".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose]),
                Register::new("x28".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose]),
                Register::new("x29".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose, RegisterTag::FramePointer]),
                Register::new("x30".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::GeneralPurpose]),
            Register::new("sp".to_string(), 64, RegisterSaver::Callee, vec![RegisterTag::StackPointer])
        ] }
    }
}

impl Arch for AArch64MacOs {
    fn name(&self) -> String {
        Self::NAME.to_string()
    }

    fn architecture_bits(&self) -> u8 {
        Self::BITS_COUNT
    }

    fn generate_assembly(&self, macro_instructions: Vec<MacroInstruction>) -> String {
        todo!()
    }
}