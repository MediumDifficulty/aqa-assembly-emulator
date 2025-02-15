use assembler::assemble;
use serde::Serialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

mod assembler;
pub mod macros;
pub mod parser;

#[wasm_bindgen]
pub fn test() -> i32 {
    console_log::init_with_level(log::Level::Debug).unwrap();
    set_panic_hook();
    assemble("begin:\n\tmov R2, r1, [R2 + r3]").unwrap();
    12
}

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

#[derive(Serialize)]
struct Lint {
    err: String,
    from: u32,
    to: u32
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// https://developer.arm.com/documentation/ddi0597/2024-12?lang=en
// https://iitd-plos.github.io/col718/ref/arm-instructionset.pdf
// https://peterhigginson.co.uk/ARMlite/doc.php
#[derive(Debug)]
struct Instruction {
    condition: Condition,
    instruction_body: InstructionBody,
}

#[derive(Debug)]
enum InstructionBody {
    DataProcessing(DataProcessing),
}

#[derive(Debug)]
struct DataProcessing {
    opcode: DataProcessingOpcode,
    set_condition_codes: bool,
    register: Register,
    dest: Register,
    operand: DataProcessingOperand,
}

#[derive(Debug)]
enum DataProcessingOperand {
    Immediate { rotate: u8, value: u8 },
    Register { shift: u8, register: Register },
}

#[derive(Debug)]
struct Register(u8);

#[derive(Debug)]
enum DataProcessingOpcode {
    AND,
    EOR,
    SUB,
    RSB,
    ADD,
    ADC,
    SBC,
    TST,
    TEQ,
    CMP,
    CMN,
    ORR,
    MOV,
    BIC,
    MVN,
}

#[derive(Default, Debug)]
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
