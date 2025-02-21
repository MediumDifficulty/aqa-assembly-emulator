use bitvec::{field::BitField, order::Msb0, slice::BitSlice, view::AsMutBits};
use funty::Integral;
use log::info;

use crate::{DataProcessing, Instruction, Register, Shift, ShiftAmount};

impl Instruction {
    pub fn serialise(&self, mut dest: &mut [u8]) {
        let bits = dest.as_mut_bits::<Msb0>();
        let mut writer = InstructionWriter::new(bits);
        writer.write(self.condition as u8, 4);

        match &self.instruction_body {
            crate::InstructionBody::DataProcessing(data_processing) => serialise_data_processing(&mut writer, &data_processing),
        }
    }
}

#[allow(unused)]
fn print_serialised(serialised: &[u8]) {
    let mut s = String::new();
    for byte in serialised {
        s.push_str(format!("{byte:08b}").as_str());
    }
    info!("{s}");
}

struct InstructionWriter<'a> {
    pos: usize,
    pub slice: &'a mut BitSlice<u8, Msb0>
}

impl<'a> InstructionWriter<'a> {
    pub fn new(slice: &'a mut BitSlice<u8, Msb0>) -> Self {
        Self {
            pos: 0,
            slice
        }
    }

    pub fn skip(&mut self, count: usize) {
        self.pos += count;
    }

    pub fn write(&mut self, value: impl Integral, bits: usize) {
        let end = self.pos + bits;
        self.slice[self.pos..end].store_be(value);
        self.pos = end;
    }

    fn write_shift(&mut self, shift: Shift) {
        match shift.amount {
            crate::ShiftAmount::Immediate(val) => self.write(val, 5),
            crate::ShiftAmount::Register(Register(reg)) => {
                self.write(reg, 4);
                self.write(0, 1);
            },
        }

        self.write(shift.ty as u8, 2);
        self.write(matches!(shift.amount, ShiftAmount::Register(_)) as u8, 1);
    }
}

fn serialise_data_processing(writer: &mut InstructionWriter, instruction: &DataProcessing) {
    // Instruction code
    writer.write(0, 2);

    // Skip Immediate Operand
    writer.skip(1);

    // Opcode
    writer.write(instruction.opcode as u8, 4);

    writer.write(instruction.set_condition_codes as u8, 1);

    writer.write(instruction.register.0, 4);
    writer.write(instruction.dest.0, 4);

    match &instruction.operand {
        crate::DataProcessingOperand::Immediate { rotate, value } =>  {
            writer.slice.set(6, true);
            writer.write(*rotate, 4);
            writer.write(*value, 8);
        },
        crate::DataProcessingOperand::Register { shift, register } => {
            writer.slice.set(6, false);
            writer.write_shift(*shift);
            writer.write(register.0, 4);
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::Instruction;

    use super::print_serialised;

    #[test]
    fn test_fn() {
        let mut dest = [0u8; 4];
        let instruction = Instruction {
            condition: Default::default(),
            instruction_body: crate::InstructionBody::DataProcessing(crate::DataProcessing {
                opcode: crate::DataProcessingOpcode::MOV,
                set_condition_codes: false,
                register: crate::Register(0),
                dest: crate::Register(1),
                operand: crate::DataProcessingOperand::Immediate { rotate: 0, value: 12 }
            }),
        };
        instruction.serialise(&mut dest);
        print_serialised(&dest);
    }
}