/// Holds all the possible locations data could be at during the runtime of the compiled program
pub enum DataPosition{
    Register(/*name: */String),
    StackOffset(/*offset:*/usize),
    StackOffsetAt(/*offset location*/Box<DataPosition>),
    Heap(/*address at: */Box<DataPosition>),
}