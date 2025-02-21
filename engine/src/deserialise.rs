use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bitvec::prelude::*;
use num_traits::FromPrimitive;

use crate::Condition;
use crate::DataProcessing;
use crate::DataProcessingOpcode;
use crate::DataProcessingOperand;
use crate::Instruction;
use crate::Register;
use crate::Shift;
use crate::ShiftType;

impl Instruction {
    pub fn deserialise(src: &[u8; 4]) -> Result<Self> {
        let bits = src.view_bits::<Msb0>();

        let condition = Condition::from_u8(bits[0..4].load_be::<u8>())
            .context("Invalid opcode")?;

        let mut reader = InstructionReader::new(&bits[6..]);

        match bits[4..=5].load_be::<u8>() {
            0b00 => deserialise_data_processing(&mut reader).map(|body| Instruction {
                condition,
                instruction_body: crate::InstructionBody::DataProcessing(body)
            }),
            _ => Err(anyhow!("Invalid Opcode"))
        }
    }
}

struct InstructionReader<'a> {
    pos: usize,
    pub slice: &'a BitSlice<u8, Msb0>
}

impl<'a> InstructionReader<'a> {
    pub fn new(slice: &'a BitSlice<u8, Msb0>) -> Self {
        Self {
            pos: 0,
            slice
        }
    }

    pub fn read(&mut self, bits: usize) -> &BitSlice<u8, Msb0> {
        let end = self.pos + bits;
        let r = &self.slice[self.pos..end];
        self.pos = end;
        r
    }

    pub fn read_bool(&mut self) -> bool {
        let r = self.slice.get(self.pos).as_deref() == Some(&true);
        self.pos += 1;
        r
    }

    pub fn read_register(&mut self) -> Register {
        Register(self.read(4).load_be::<u8>())
    }

    pub fn read_shift(&mut self) -> Shift {
        let register = self.slice.get(self.pos + 7).as_deref() == Some(&true);
        if register {
            let register = self.read_register();
            self.pos += 1;
            let ty = ShiftType::from_u8(self.read(2).load_be::<u8>())
                .expect("Unable to map to shift");

            Shift {
                amount: crate::ShiftAmount::Register(register),
                ty
            }
        } else {
            let amount = self.read(5).load_be::<u8>();
            let ty = ShiftType::from_u8(self.read(2).load_be::<u8>())
                .expect("Unable to map to shift");

            Shift {
                amount: crate::ShiftAmount::Immediate(amount),
                ty
            }
        }
    }
}

fn deserialise_data_processing(reader: &mut InstructionReader) -> Result<DataProcessing> {
    let immediate = reader.read_bool();
    let opcode = DataProcessingOpcode::from_u8(reader.read(4).load_be::<u8>())
        .context("Invalid DP Opcode")?;

    let set_condition_codes = reader.read_bool();
    let register = reader.read_register();
    let dest = reader.read_register();

    let operand = if immediate {
        let rotate = reader.read(4).load_be::<u8>();
        let value = reader.read(8).load_be::<u8>();
        DataProcessingOperand::Immediate { rotate, value }
    } else {
        let shift = reader.read_shift();
        let source = reader.read_register();
        DataProcessingOperand::Register { shift, register: source }
    };

    Ok(DataProcessing {
        dest,
        opcode,
        operand,
        register,
        set_condition_codes
    })
}
