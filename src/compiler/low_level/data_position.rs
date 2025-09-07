use crate::compiler::low_level::data_position::DataPosition::Register;

/// Holds all the possible locations data could be at during the runtime of the compiled program
#[derive(Clone, Debug)]
pub enum DataPosition{
    Register(/*name: */String),
    StackOffset(/*offset:*/usize),
    StackOffsetAt(/*offset location*/Box<DataPosition>),
    Heap(/*address at: */Box<DataPosition>, /*size: */usize),
}

impl DataPosition {
    const GENERAL_PURPOSE_REGISTER_NAME: &'static str = "generalPurposeRegister";

    pub fn general_purpose_register() -> Self{
        Register(String::from(Self::GENERAL_PURPOSE_REGISTER_NAME))
    }

    pub fn is_general_purpose_register(&self) -> bool {
        match self {
            Register(name) => {
                *name == Self::GENERAL_PURPOSE_REGISTER_NAME
            },
            _ => false,
        }
    }

    pub fn is_register(&self, register_name: String) -> bool {
        match self {
            DataPosition::Register(own_name) => {
                if *own_name == register_name {
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    /// Gets the name of the register if it refers to one,
    /// returns None otherwise
    pub fn register_name(&self) -> Option<String> {
        match self{
            Register(name) => Some(name.clone()),
            _ => None,
        }
    }

    /// The time costs of getting the value
    pub fn cost(&self) -> usize {
        match self {
            DataPosition::Register(_) => { 0 }
            DataPosition::StackOffset(_) => { 1 }
            DataPosition::StackOffsetAt(pos) => { 1 + pos.cost() }
            DataPosition::Heap(pos, _) => { 2 + pos.cost() }
        }
    }

    pub fn immediate_stack_offset(&self) -> Option<usize>{
        match self {
            DataPosition::StackOffset(offset) => { Some(*offset) }
            _ => None,
        }
    }
}