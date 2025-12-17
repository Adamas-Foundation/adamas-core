// Adamas Virtual Machine (Skeleton)
// Will execute smart contracts in Phase 3

pub struct VirtualMachine {
    pub memory: Vec<u8>,
}

pub enum OpCode {
    PUSH,
    ADD,
    STORE,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine { memory: Vec::new() }
    }

    // Nota: Ho messo un "_" davanti a program per evitare il warning
    pub fn execute(&mut self, _program: Vec<OpCode>) {
        println!("Executing smart contract...");
    }
}