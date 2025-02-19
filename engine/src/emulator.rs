use anyhow::Result;

use crate::{Condition, DataProcessing, Flags, Instruction, ProcessorState};

impl<'a> ProcessorState<'a> {
    pub fn step(&mut self) -> Result<()> {
        let start_instruction = self.get_pc() as usize;
        let end_instruction = start_instruction + 4;
        let instruction = &self.ram[start_instruction..end_instruction].try_into()?;
        let instruction = Instruction::deserialise(instruction)?;

        if !instruction.condition.matches(self.flags) {
            return Ok(());
        }

        match instruction.instruction_body {
            crate::InstructionBody::DataProcessing(data_processing) => self.execute_data_processing(data_processing),
        }
        
        todo!()
    }

    fn execute_data_processing(&mut self, instruction: DataProcessing) {
        // let src = match instruction.
    }

    #[inline]
    fn get_pc(&self) -> u32 {
        self.registers[15]
    }
}


impl Condition {
    fn matches(&self, flags: Flags) -> bool {
        match self {
            Condition::EQ => flags.z,
            Condition::NE => !flags.z,
            Condition::CS => flags.c,
            Condition::CC => !flags.c,
            Condition::MI => flags.n,
            Condition::PL => !flags.n,
            Condition::VS => flags.v,
            Condition::VC => !flags.v,
            Condition::HI => flags.c && !flags.z,
            Condition::LS => !flags.c && flags.z,
            Condition::GE => flags.n == flags.v,
            Condition::LT => flags.n != flags.v,
            Condition::GT => !flags.z && (flags.n == flags.v),
            Condition::LE => flags.z || (flags.n != flags.v),
            Condition::AL => true,
        }
    }
}