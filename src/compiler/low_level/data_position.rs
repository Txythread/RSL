/// Holds all the possible locations data could be at during the runtime of the compiled program
#[derive(Clone, Debug)]
pub enum DataPosition{
    Register(/*name: */String),
    StackOffset(/*offset:*/usize),
    StackOffsetAt(/*offset location*/Box<DataPosition>),
    Heap(/*address at: */Box<DataPosition>, /*size: */usize),
}

impl DataPosition {
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

    /// The time costs of getting the value
    pub fn cost(&self) -> usize {
        match self {
            DataPosition::Register(_) => { 0 }
            DataPosition::StackOffset(_) => { 1 }
            DataPosition::StackOffsetAt(pos) => { 1 + pos.cost() }
            DataPosition::Heap(pos, _) => { 2 + pos.cost() }
        }
    }
}