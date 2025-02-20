use assembler::assemble;
use num_derive::FromPrimitive;
use serde::Serialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

mod assembler;
pub mod macros;
pub mod parser;
mod serialise;
mod emulator;
mod deserialise;

#[wasm_bindgen]
pub fn test_assemble(src: &str) -> JsValue {
    match assemble(src) {
        Ok(_) => JsValue::null(),
        Err(e) => {
            let span = match e.location {
                pest::error::InputLocation::Pos(p) => (p, p),
                pest::error::InputLocation::Span(p) => p,
            };

            serde_wasm_bindgen::to_value(&Lint {
                from: span.0 as u32,
                to: span.1 as u32,
                err: e.with_path("program.as").to_string(),
            }).unwrap()
        },
    }
}

#[wasm_bindgen]
pub fn assemble_into_ram(src: &str, ram: &mut [u8]) -> JsValue {
    let assembled = assemble(src);
    
    match assembled {
        Ok(prog) => {
            ram.fill(0);
            prog.serialise(ram);
            JsValue::null()
        },
        Err(e) => {
            let span = match e.location {
                pest::error::InputLocation::Pos(p) => (p, p),
                pest::error::InputLocation::Span(p) => p,
            };

            serde_wasm_bindgen::to_value(&Lint {
                from: span.0 as u32,
                to: span.1 as u32,
                err: e.with_path("program.as").to_string(),
            }).unwrap()
        },
    }
}

#[wasm_bindgen]
pub fn step(ram: &mut [u8], registers: &mut [u32], flags: u8) -> JsValue {
    let mut state = ProcessorState {
        flags: Flags::from(flags),
        ram,
        registers: registers.try_into().unwrap()
    };

    serde_wasm_bindgen::to_value(&match state.step() {
        Ok(_) => ExecutionResult { message: "".into(), flags: state.flags.into() },
        Err(e) => ExecutionResult { message: e.to_string(), flags: state.flags.into() },
    }).unwrap()
}

#[derive(Serialize)]
struct ExecutionResult {
    message: String,
    flags: u8
}

#[derive(Serialize)]
struct Lint {
    err: String,
    from: u32,
    to: u32
}

pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

pub struct ProcessorState<'a> {
    pub ram: &'a mut [u8],
    pub registers: &'a mut [u32; 16],
    pub flags: Flags
}

#[derive(Debug, Clone, Copy)]
pub struct Flags {
    /// Negative
    n: bool,
    /// Zero
    z: bool,
    /// Carry
    c: bool,
    /// Overflow
    v: bool
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        Self {
            n: (value >> 3) & 1 == 1,
            z: (value >> 2) & 1 == 1,
            c: (value >> 1) & 1 == 1,
            v: (value >> 0) & 1 == 1,
        }
    }
}

impl Into<u8> for Flags {
    fn into(self) -> u8 {
          ((self.n as u8) << 3)
        | ((self.z as u8) << 2)
        | ((self.c as u8) << 1)
        | ((self.v as u8) << 0)
    }
}

#[derive(Debug)]
struct Program {
    instructions: Vec<Instruction>
}

impl Program {
    pub fn serialise(&self, ram: &mut [u8]) {
        for (dest, instruction) in ram.chunks_mut(4).zip(&self.instructions) {
            instruction.serialise(dest);
        }
    }
}

// https://developer.arm.com/documentation/ddi0597/2024-12?lang=en
// https://iitd-plos.github.io/col718/ref/arm-instructionset.pdf
// https://peterhigginson.co.uk/ARMlite/doc.php
#[derive(Debug, PartialEq, Eq)]
struct Instruction {
    condition: Condition,
    instruction_body: InstructionBody,
}

#[derive(Debug, PartialEq, Eq)]
enum InstructionBody {
    DataProcessing(DataProcessing),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DataProcessing {
    opcode: DataProcessingOpcode,
    set_condition_codes: bool,
    register: Register,
    dest: Register,
    operand: DataProcessingOperand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DataProcessingOperand {
    Immediate { rotate: u8, value: u8 },
    Register { shift: Shift, register: Register },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Shift {
    ty: ShiftType,
    amount: ShiftAmount
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShiftAmount {
    Immediate(u8),
    Register(Register)
}

#[derive(Debug, Clone, Copy, FromPrimitive, Default, PartialEq, Eq)]
enum ShiftType {
    #[default]
    LogicalLeft,
    LogicalRight,
    ArithmeticRight,
    RotateRight
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Register(u8);

#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq, Eq)]
enum DataProcessingOpcode {
    AND,
    EOR,
    SUB,
    RSB,
    ADD,
    ADC,
    SBC,
    RSC,
    TST,
    TEQ,
    CMP,
    CMN,
    ORR,
    MOV,
    BIC,
    MVN,
}

#[derive(Default, Debug, Clone, Copy, FromPrimitive, PartialEq, Eq)]
enum Condition {
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    #[default]
    AL,
}
