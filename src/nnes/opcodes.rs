use crate::nnes::*;

impl NNES
{
    pub fn handle_brk(&mut self)
    {
        self.set_flag(Flag::Break, true);
    }

    pub fn handle_lda(&mut self)
    {
        let pc: u16 = self.get_program_counter();
        let param: u8 = self.memory_read(pc);
        self.set_program_counter(pc + 1);
        self.set_register(Register::ACCUMULATOR, param);
        self.update_op_flags(param);        
    }

    pub fn handle_tax(&mut self)
    {
        let reg_acc: u8 = self.get_register(Register::ACCUMULATOR);
        self.set_register(Register::XIndex, reg_acc);
        self.update_op_flags(reg_acc);
    }
    
    pub fn handle_inx(&mut self)
    {
        let reg_x: u8 = self.get_register(Register::XIndex);
        if reg_x == 0xff
        { 
            self.set_register(Register::XIndex, 0); 
            self.update_op_flags(0);
        }
        else 
        { 
            self.set_register(Register::XIndex, reg_x + 1); 
            self.update_op_flags(reg_x + 1);
        }
    }
}