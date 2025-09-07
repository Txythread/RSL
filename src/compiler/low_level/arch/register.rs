/*
This is a helper struct to find the correct register usage later.
Individual architectures might implement register/stack/heap selection handling themselves.
 */

#[derive(Clone)]
pub struct Register{
    pub name: String,
    pub size_bits: u8,
    pub saver: RegisterSaver,
    pub tags: Vec<RegisterTag>,
}

impl Register{
    pub fn new(name: String, size_bits: u8, saver: RegisterSaver, tags: Vec<RegisterTag>) -> Register{
        Register { name, size_bits, saver, tags }
    }

    pub fn is_argument(&self, number: u8) -> bool {
        let mut is_nth_argument = false;
        self.tags.iter().for_each(| x |
            {
                match x.clone(){
                    RegisterTag::Argument(n) => {
                        if n == number{
                            is_nth_argument = true;
                        }
                    }
                    _ => {}
                }
            }
        );

        is_nth_argument
    }
}

#[derive(Clone)]
pub enum RegisterSaver{
    Caller,     // caller-saved register
    Callee,     // callee-saved register
    OS,         // prob. don't modify
    None,        // Scratch register
}

#[derive(Clone)]
pub enum RegisterTag {
    Argument(/*(n-th argument) n=*/u8),
    GeneralPurpose,
    Scratch,
    StackPointer,
    FramePointer,
    NoModify
}