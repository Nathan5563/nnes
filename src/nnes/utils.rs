use crate::nnes::{Register, NNES};

impl NNES {
    pub fn reset_nnes(&mut self) {
        self.set_program_counter(0);
        self.set_stack_pointer(0);
        self.reset_registers();
        self.reset_flags();
        self.reset_memory();
    }

    pub fn print_nnes(&self) {
        println!("Program Counter: {:04X}", self.get_program_counter());
        println!("Stack Pointer: {:02X}", self.get_stack_pointer());
        println!(
            "Accumulator: {:02X}",
            self.get_register(Register::Accumulator)
        );
        println!("X Index: {:02X}", self.get_register(Register::XIndex));
        println!("Y Index: {:02X}", self.get_register(Register::YIndex));
        println!("Flags: {:08b}", self.get_flags());
    }
}
