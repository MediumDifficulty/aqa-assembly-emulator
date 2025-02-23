use anyhow::{anyhow, Result};
use log::info;

use crate::{Condition, DataProcessing, DataProcessingOpcode, Flags, Instruction, ProcessorState, Register, Shift};

impl<'a> ProcessorState<'a> {
    pub fn step(&mut self) -> Result<()> {
        let start_instruction = self.get_pc() as usize;
        let end_instruction = start_instruction + 4;
        let instruction = &self.ram[start_instruction..end_instruction].try_into()?;
        let instruction = Instruction::deserialise(instruction)?;

        info!("{instruction:?}");

        if !instruction.condition.matches(self.flags) {
            self.inc_pc();
            return Ok(());
        }

        match instruction.instruction_body {
            crate::InstructionBody::DataProcessing(data_processing) => self.execute_data_processing(data_processing),
        }?;

        self.inc_pc();

        Ok(())
    }

    fn inc_pc(&mut self) {
        self.registers[15] += 4;
    }

    fn execute_data_processing(&mut self, instruction: DataProcessing) -> Result<()> {
        let rhs = match instruction.operand {
            crate::DataProcessingOperand::Immediate { rotate, value } => (value as u32).rotate_left(rotate as u32),
            crate::DataProcessingOperand::Register { shift, register } => shift.eval(self.registers[register.0 as usize]),
        };

        let lhs = self.get_register(instruction.register)?;
        let mut flags = self.flags;
        let dest = self.get_register_mut(instruction.dest)?;

        // TODO: Implement overflow flag
        match instruction.opcode {
            DataProcessingOpcode::AND => *dest = lhs & rhs,
            DataProcessingOpcode::EOR => *dest = lhs ^ rhs,
            DataProcessingOpcode::SUB => {
                *dest = lhs.wrapping_sub(rhs);
                flags.c = *dest >= lhs;
            },
            DataProcessingOpcode::RSB => {
                *dest = rhs.wrapping_sub(lhs);
                flags.c = *dest > rhs
            },
            DataProcessingOpcode::ADD => {
                *dest = lhs.wrapping_add(rhs);
                flags.c = *dest > lhs;
            },
            DataProcessingOpcode::ADC => {
                *dest = lhs.wrapping_add(rhs).wrapping_add(flags.c as u32);
                flags.c = *dest > lhs;
            },
            DataProcessingOpcode::SBC => {
                *dest = lhs.wrapping_sub(rhs).wrapping_add(flags.c as u32).wrapping_sub(1);
            },
            DataProcessingOpcode::RSC => {
                *dest = rhs.wrapping_sub(lhs).wrapping_add(flags.c as u32).wrapping_sub(1);
            },
            DataProcessingOpcode::TST => {
                let dummy = lhs & rhs;
                self.flags.z = dummy == 0;
                self.flags.n = (dummy >> 31) & 1 == 1; 
        
                return Ok(());
            },
            DataProcessingOpcode::TEQ => {
                let dummy = lhs ^ rhs;
                self.flags.z = dummy == 0;
                self.flags.n = (dummy >> 31) & 1 == 1; 
        
                return Ok(());
            },
            DataProcessingOpcode::CMP => {
                let dummy = lhs.wrapping_sub(rhs);
                self.flags.c = dummy >= lhs;
                self.flags.z = dummy == 0;
                self.flags.n = (dummy >> 31) & 1 == 1; 
        
                return Ok(());
            },
            DataProcessingOpcode::CMN => {
                let dummy = lhs.wrapping_add(rhs);
                self.flags.c = dummy > lhs;
                self.flags.z = dummy == 0;
                self.flags.n = (dummy >> 31) & 1 == 1; 

                return Ok(());
            },
            DataProcessingOpcode::ORR => *dest = lhs | rhs,
            DataProcessingOpcode::MOV => *dest = rhs,
            DataProcessingOpcode::BIC => *dest = lhs & !rhs,
            DataProcessingOpcode::MVN => *dest = !rhs,
        };

        flags.z = *dest == 0;
        flags.n = (*dest >> 31) & 1 == 1; 
        
        self.flags = flags;

        Ok(())
    }

    fn get_register_mut(&mut self, reg: Register) -> Result<&mut u32> {
        self.registers.get_mut(reg.0 as usize)
            .ok_or(anyhow!("Invalid Register index"))
    }

    fn get_register(&self, reg: Register) -> Result<u32> {
        self.registers.get(reg.0 as usize)
            .copied()
            .ok_or(anyhow!("Invalid Register index"))
    }

    fn get_pc(&self) -> u32 {
        self.registers[15]
    }
}

impl Shift {
    fn eval(&self, input: u32) -> u32 {
        let shift = match self.amount {
            crate::ShiftAmount::Immediate(val) => val as u32,
            crate::ShiftAmount::Register(register) => register.0 as u32,
        };

        match self.ty {
            crate::ShiftType::LogicalLeft => input >> shift,
            crate::ShiftType::LogicalRight => input << shift,
            crate::ShiftType::ArithmeticRight => ((input as i32) << shift) as u32, // TODO: Confirm if this is correct
            crate::ShiftType::RotateRight => input.rotate_right(shift),
        }
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