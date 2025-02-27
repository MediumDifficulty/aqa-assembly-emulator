use std::collections::HashMap;

use assembler::assemble;
use log::info;
use num_derive::FromPrimitive;
use serde::Serialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[cfg(test)] use proptest_derive::Arbitrary;
#[cfg(test)] use proptest::prelude::{any, Strategy, BoxedStrategy};

mod assembler;
pub mod macros;
pub mod parser;
mod serialise;
mod emulator;
mod deserialise;

#[wasm_bindgen]
pub fn lint(src: &str) -> JsValue {
    let parsed = assembler::parse_per_line(src);
    let labels = assembler::get_lint_labels(&parsed);
    let source_map = assembler::gen_source_map(&parsed);

    let lints = parsed
        .into_iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let line = line.and_then(|mut line| assembler::lint_line(line.next().unwrap(), &labels));

            match line {
                Ok(_) => None,
                Err(e) => {
                    let span = match e.location {
                        pest::error::InputLocation::Pos(p) => (p, p),
                        pest::error::InputLocation::Span(p) => p,
                    };

                    Some(Lint {
                        err: e.clone().with_path("program.as").to_string(),
                        from: span.0 as u32,
                        to: span.1 as u32,
                        line: i as u32
                    })
                },
            }
        })
        // .map(|lint| serde_wasm_bindgen::to_value(&lint).unwrap())
        .collect::<Vec<_>>();

    serde_wasm_bindgen::to_value(&Lints {
        lints,
        source_map
    }).unwrap()
}

#[wasm_bindgen]
pub fn assemble_into_ram(src: &str, ram: &mut [u8]) {
    setup_logging();
    if let Ok(prog) = assemble(src) {
        ram.fill(0);
        prog.serialise(ram);
    }
}

#[wasm_bindgen]
pub fn step(ram: &mut [u8], registers: &mut [u32], flags: u8) -> JsValue {
    setup_logging();
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
struct Lints {
    lints: Vec<Lint>,
    source_map: HashMap<u32, u32>
}

#[derive(Serialize)]
struct Lint {
    pub err: String,
    pub line: u32,
    pub from: u32,
    pub to: u32
}

pub fn setup_logging() {
    console_error_panic_hook::set_once();
    static mut INITIALISED: bool = false;
    unsafe {
        if !INITIALISED {
            console_log::init_with_level(log::Level::Trace).unwrap();
            info!("Initialised logging");
            INITIALISED = true
        }
    }
}

pub struct ProcessorState<'a> {
    pub ram: &'a mut [u8],
    pub registers: &'a mut [u32; 16],
    pub flags: Flags
}

#[derive(Debug, Clone, Copy, Default)]
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
#[cfg_attr(test, derive(Arbitrary))]
pub struct Instruction {
    condition: Condition,
    body: InstructionBody,
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum InstructionBody {
    DataProcessing(DataProcessing),
    Branch(Branch)
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Branch {
    link: bool,
    #[cfg_attr(test, proptest(strategy = "any::<u32>().prop_map(|x| x & ((1 << 24) - 1))"))]
    offset: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct DataProcessing {
    opcode: DataProcessingOpcode,
    set_condition_codes: bool,
    register: Register,
    dest: Register,
    operand: DataProcessingOperand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum DataProcessingOperand {
    #[cfg_attr(test, proptest(strategy = "any::<(u8, u8)>().prop_map(|(a, b)| Self::Immediate{ rotate: a % 16, value: b })"))]
    Immediate { rotate: u8, value: u8 },
    Register { shift: Shift, register: Register },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Shift {
    ty: ShiftType,
    amount: ShiftAmount
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum ShiftAmount {
    #[cfg_attr(test, proptest(strategy = "any::<u8>().prop_map(|x| Self::Immediate(x % 16))"))]
    Immediate(u8),
    Register(Register)
}

impl Default for ShiftAmount {
    fn default() -> Self {
        Self::Immediate(0)
    }
}

#[derive(Debug, Clone, Copy, FromPrimitive, Default, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum ShiftType {
    #[default]
    LogicalLeft,
    LogicalRight,
    ArithmeticRight,
    RotateRight
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Register(u8);

#[cfg(test)]
impl proptest::prelude::Arbitrary for Register {
    type Parameters = ();

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        proptest::prelude::any::<u8>()
            .prop_map(|x| Register(x % 16))
            .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum DataProcessingOpcode {
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
#[cfg_attr(test, derive(Arbitrary))]
pub enum Condition {
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

#[cfg(test)]
mod test {
    use proptest::proptest;

    use crate::Instruction;

    proptest! {
        #[test]
        fn serde_all(instruction: Instruction) {
            let mut dest = [0u8; 4];
            instruction.serialise(&mut dest);
            assert_eq!(Instruction::deserialise(&dest).unwrap(), instruction);
        }
    }
}
