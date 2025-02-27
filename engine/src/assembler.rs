use std::{collections::HashMap, ops::Div};

use pest::{
    error::ErrorVariant, iterators::{Pair, Pairs}, Parser, Span
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    parser::{self, AssemblyParser, Rule}, unwrap_or_continue, Condition, DataProcessing, DataProcessingOpcode, DataProcessingOperand, Instruction, InstructionBody, Program, Register, Shift
};

const MAX_REG_NUM: u8 = 15;

pub type Res<T> = Result<T, pest::error::Error<parser::Rule>>;

pub fn assemble(src: &str) -> Res<Program> {
    let parsed = AssemblyParser::parse(Rule::program, src)?.next().unwrap();

    let labels = get_labels(&parsed);

    let mut instructions = Vec::new();
    let mut current_addr = 0;

    for line in parsed.into_inner() {
        let line = unwrap_or_continue!(line.into_inner().next());
        match line.as_rule() {
            Rule::instruction => {
                instructions.push(assemble_instruction(line, &labels, current_addr)?);
                current_addr += 4;
            }
            Rule::label => {},
            _ => unreachable!(),
        }
    }

    Ok(Program { instructions })
}

pub fn get_lint_labels(lines: &[Res<Pairs<'_, Rule>>]) -> HashMap<String, u32> {
    let labels = lines.iter()
        .filter_map(|line| line.as_ref().ok())
        .cloned()
        .filter_map(|mut line| {
            line.next()
                .unwrap()
                .into_inner()
                .next()
                .and_then(|line| match line.as_rule() {
                    parser::Rule::label => Some(line.into_inner().next().unwrap().as_str().to_string()),
                    _ => None
                })
        });

    labels
        .map(|label| (label, 0))
        .collect()
}

pub fn parse_per_line(src: &str) -> Vec<Res<Pairs<'_, Rule>>> {
    src
        .lines()
        .map(|line| AssemblyParser::parse(Rule::lint_line, line))
        .collect::<Vec<_>>()
}

pub fn gen_source_map(lines: &[Res<Pairs<'_, Rule>>]) -> HashMap<u32, u32> {
    let mut current_addr = 0;
    let mut source_map = HashMap::new();

    for (i, line) in lines.iter().cloned().enumerate() {
        if let Ok(mut line) = line {
            if let Some(line) = line.next()
                .unwrap()
                .into_inner()
                .next() {
                    match line.as_rule() {
                        parser::Rule::instruction => {
                            source_map.insert(current_addr as u32, i as u32);
                            current_addr += 4;
                        },
                        _ => {}
                    }
                }
        }
    }

    source_map
}

pub fn lint_line(parsed: Pair<'_, Rule>, labels: &HashMap<String, u32>) -> Res<()> {
    let parsed = match parsed.into_inner().next() {
        Some(p) => p,
        None => return Ok(()),
    };

    match parsed.as_rule() {
        Rule::label => Ok(()),
        Rule::instruction => assemble_instruction(parsed, labels, 0).map(|_| ()),
        Rule::EOI => Ok(()),
        _ => unreachable!("{parsed:?}"),
    }
}

fn span_err(span: Span<'_>, msg: &str) -> pest::error::Error<parser::Rule> {
    pest::error::Error::new_from_span(
        ErrorVariant::CustomError {
            message: msg.into(),
        },
        span,
    )
}

fn get_labels(src: &Pair<'_, Rule>) -> HashMap<String, u32> {
    let mut addr = 0;
    let mut labels = HashMap::new();

    for line in src.clone().into_inner() {
        let line = unwrap_or_continue!(line.into_inner().next());
        match line.as_rule() {
            Rule::label => { labels.insert(line.into_inner().next().unwrap().as_str().to_string(), addr); },
            Rule::instruction => addr += 4,
            _ => unreachable!("{line:?}")
        }
    }

    labels
}

fn assemble_instruction(src: Pair<'_, parser::Rule>, labels: &HashMap<String, u32>, current_addr: u32) -> Res<Instruction> {
    let src_span = src.as_span();
    let mut inner = src.into_inner();
    let opcode = inner.next().ok_or(span_err(src_span, "Missing opcode"))?;
    // let opcode_span = opcode.as_span();
    
    if let Some((opcode, condition)) = parse_opcode(opcode.as_str()) {
        let body = match opcode {
            Opcode::And => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::AND),
            Opcode::Eor => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::EOR),
            Opcode::Sub => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::SUB),
            Opcode::Rsb => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::RSB),
            Opcode::Add => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::ADD),
            Opcode::Adc => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::ADC),
            Opcode::Rsc => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::RSC),
            Opcode::Tst => assemble_two_arg_dp(&mut inner, src_span, DataProcessingOpcode::TST),
            Opcode::Teq => assemble_two_arg_dp(&mut inner, src_span, DataProcessingOpcode::TEQ),
            Opcode::Cmp => assemble_two_arg_dp(&mut inner, src_span, DataProcessingOpcode::CMP),
            Opcode::Cmn => assemble_two_arg_dp(&mut inner, src_span, DataProcessingOpcode::CMN),
            Opcode::Or => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::ORR),
            Opcode::Mov => assemble_two_arg_dp_dest(&mut inner, src_span, DataProcessingOpcode::MOV),
            Opcode::Bic => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::BIC),
            Opcode::Mvn => assemble_two_arg_dp(&mut inner, src_span, DataProcessingOpcode::MVN),
            Opcode::B => assemble_branch(&mut inner, src_span, false, labels, current_addr),
            Opcode::Bl => assemble_branch(&mut inner, src_span, true, labels, current_addr),
        }?;
        Ok(Instruction { condition, body })
    } else {
        Err(span_err(src_span, "Invalid opcode"))
    }

}

#[derive(EnumIter, Debug)]
enum Opcode {
    And,
    Eor,
    Sub,
    Rsb,
    Add,
    Adc,
    Rsc,
    Tst,
    Teq,
    Cmp,
    Cmn,
    Or,
    Mov,
    Bic,
    Mvn,
    B,
    Bl
}

impl Opcode {
    pub fn as_str(&self) -> &'static [&'static str] {
        match self {
            Opcode::And => &["and"],
            Opcode::Eor => &["eor", "xor"],
            Opcode::Sub => &["sub"],
            Opcode::Rsb => &["rsb"],
            Opcode::Add => &["add"],
            Opcode::Adc => &["adc"],
            Opcode::Rsc => &["rsc"],
            Opcode::Tst => &["tst"],
            Opcode::Teq => &["teq"],
            Opcode::Cmp => &["cmp"],
            Opcode::Cmn => &["cmn"],
            Opcode::Or =>  &["or", "orr"],
            Opcode::Mov => &["mov"],
            Opcode::Bic => &["bic"],
            Opcode::Mvn => &["mvn"],
            Opcode::B =>   &["b"],
            Opcode::Bl =>  &["bl"],
        }
    }
}

fn parse_opcode(src: &str) -> Option<(Opcode, Condition)> {
    let src = src.to_ascii_lowercase();
    for opcode in Opcode::iter() {
        for op_str in opcode.as_str() {
            if !src.starts_with(op_str) {
                continue;
            }

            let remaining = &src[op_str.len()..];

            if remaining.is_empty() {
                return Some((opcode, Condition::AL))
            }

            for condition in Condition::iter() {
                if remaining == condition.as_str() {
                    return Some((opcode, condition));
                }
            }
        }
    }

    None
}

impl Condition {
    fn as_str(&self) -> &'static str {
        match self {
            Condition::EQ => "eq",
            Condition::NE => "ne",
            Condition::CS => "cs",
            Condition::CC => "cc",
            Condition::MI => "mi",
            Condition::PL => "pl",
            Condition::VS => "vs",
            Condition::VC => "vc",
            Condition::HI => "hi",
            Condition::LS => "ls",
            Condition::GE => "ge",
            Condition::LT => "lt",
            Condition::GT => "gt",
            Condition::LE => "le",
            Condition::AL => "al",
        }
    }
}

fn assemble_branch(pairs: &mut Pairs<'_, Rule>, span: Span<'_>, link: bool, labels: &HashMap<String, u32>, current_addr: u32) -> Res<InstructionBody> {
    let offset = pairs.next().ok_or(span_err(span, "Missing offset"))?;
    let offset = match offset.as_rule() {
        Rule::literal => parse_literal(offset)?,
        Rule::text => labels.get(offset.as_str())
            .ok_or(span_err(span, "Unknown label"))?
            .div(4)
            .wrapping_sub(current_addr / 4 + 1),
        _ => return Err(span_err(span, "Invalid offset"))
    };

    Ok(InstructionBody::Branch(crate::Branch { link, offset }))
}

fn assemble_two_arg_dp_dest(pairs: &mut Pairs<'_, Rule>, span: Span<'_>, opcode: DataProcessingOpcode) -> Res<InstructionBody> {
    let dest_reg = pairs.next().ok_or(span_err(span, "Missing destination"))?;
    let src = pairs.next().ok_or(span_err(span, "Missing source"))?;

    if pairs.next().is_some() {
        return Err(span_err(span, "Expected end of instruction"))
    }

    let dest = parse_reg(dest_reg)?;
    let operand = parse_dp_operand(src)?;

    Ok(InstructionBody::DataProcessing(DataProcessing {
            dest,
            opcode,
            operand,
            set_condition_codes: false,
            register: Register(0) // TODO: Expand instruction to use this as extra immediate space for MOV
    }))
}

fn assemble_two_arg_dp(pairs: &mut Pairs<'_, Rule>, span: Span<'_>, opcode: DataProcessingOpcode) -> Res<InstructionBody> {
    let reg1 = pairs.next().ok_or(span_err(span, "Missing register operand"))?;
    let src = pairs.next().ok_or(span_err(span, "Missing source"))?;

    if pairs.next().is_some() {
        return Err(span_err(span, "Expected end of instruction"))
    }

    let reg1 = parse_reg(reg1)?;
    let operand = parse_dp_operand(src)?;

    Ok(InstructionBody::DataProcessing(DataProcessing {
            dest: Register(0),
            opcode,
            operand,
            set_condition_codes: false,
            register: reg1
    }))
}

fn assemble_three_arg_dp(pairs: &mut Pairs<'_, Rule>, span: Span<'_>, opcode: DataProcessingOpcode) -> Res<InstructionBody> {
    let dest_reg = pairs.next().ok_or(span_err(span, "Missing destination"))?;
    let lhs = pairs.next().ok_or(span_err(span, "Missing lhs"))?;
    let rhs = pairs.next().ok_or(span_err(span, "Missing rhs"))?;

    if pairs.next().is_some() {
        return Err(span_err(span, "Expected end of instruction"))
    }

    let dest = parse_reg(dest_reg)?;
    let lhs = parse_reg(lhs)?;
    let operand = parse_dp_operand(rhs)?;

    Ok(InstructionBody::DataProcessing(DataProcessing {
            dest,
            opcode,
            operand,
            set_condition_codes: false,
            register: lhs
    }))
}

fn parse_dp_operand(src: Pair<'_, Rule>) -> Res<DataProcessingOperand> {
    match src.as_rule() {
        Rule::literal => Ok(DataProcessingOperand::Immediate {
            rotate: 0,
            value: parse_literal(src)? as u8,
        }),
        Rule::register => Ok(DataProcessingOperand::Register {
            shift: Shift {
                amount: crate::ShiftAmount::Immediate(0),
                ty: crate::ShiftType::LogicalLeft
            },
            register: parse_reg(src)?,
        }),
        _ => Err(span_err(src.as_span(), "Invalid source"))?,
    }
}

fn parse_reg(reg: Pair<'_, Rule>) -> Res<Register> {
    let span = reg.as_span();

    let index_pair = reg
        .into_inner()
        .next()
        .ok_or(span_err(span, "Invalid register index"))?;

    let index = match index_pair.as_rule() {
        Rule::decimal => index_pair.as_str().parse::<u8>().or(Err(span_err(
            index_pair.as_span(),
            "Malformed register index",
        )))?,
        Rule::program_counter => 15,
        Rule::link_register => 14,
        Rule::stack_pointer => 13,
        _ => Err(span_err(span, "Invalid register"))?,
    };

    if index > MAX_REG_NUM {
        Err(span_err(
            index_pair.as_span(),
            &format!("Register index cannot be greater than {MAX_REG_NUM}"),
        ))?;
    }

    Ok(Register(index))
}

fn parse_literal(literal: Pair<'_, Rule>) -> Res<u32> {
    let span = literal.as_span();
    let mut inner = literal.into_inner();

    let next = inner.next().expect("Invalid literal");

    let (contents, negative) = match next.as_rule() {
        Rule::negation => (inner.next().expect("Missing literal body"), true),
        _ => (next, false)
    };

    let value = match contents.as_rule() {
        Rule::decimal_literal => contents
            .into_inner()
            .next()
            .expect("Invalid literal")
            .as_str()
            .parse()
            .or(Err(span_err(span, "Invalid decimal literal"))),
        Rule::hex_literal => u32::from_str_radix(
            contents
                .into_inner()
                .next()
                .expect("Invalid literal")
                .as_str(),
            16,
        )
        .or(Err(span_err(span, "Invalid hex literal"))),
        _ => unreachable!(),
    }?;

    if negative {
        Ok(!value + 1)
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        simple_logger::init().unwrap();
        // assemble("label1:\n\tmov R1, #12").unwrap()
    }
}
