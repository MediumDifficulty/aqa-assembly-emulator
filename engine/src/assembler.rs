use std::{collections::HashMap, ops::Div};

use log::info;
use pest::{
    error::ErrorVariant,
    iterators::{Pair, Pairs},
    Parser, Span,
};

use crate::{
    parser::{self, AssemblyParser, Rule}, unwrap_or_continue, Condition, DataProcessing, DataProcessingOpcode, DataProcessingOperand, Instruction, InstructionBody, Program, Register, Shift
};

const MAX_REG_NUM: u8 = 15;

pub type Res<T> = Result<T, pest::error::Error<parser::Rule>>;

pub fn assemble(src: &str) -> Res<Program> {
    let parsed = AssemblyParser::parse(Rule::program, src)?.next().unwrap();

    let labels = get_labels(&parsed);

    info!("{labels:#?}");

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
    let opcode_pair = inner.next().ok_or(span_err(src_span, "Missing opcode"))?;
    let opcode_span = opcode_pair.as_span();
    let mut opcode_pair = opcode_pair.into_inner();

    let opcode_rule = opcode_pair.next().ok_or(span_err(src_span, "Missing base"))?.as_rule();
    let condition_rule = opcode_pair.next()
        .map(|cond| cond.as_rule())
        .unwrap_or(Rule::cond_al);

    let condition = Condition::from_rule(condition_rule) 
        .ok_or(span_err(src_span, "Invalid condition"))?;


    let body = match &opcode_rule {
        Rule::op_and => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::AND),
        Rule::op_eor => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::EOR),
        Rule::op_sub => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::SUB),
        Rule::op_rsb => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::RSB),
        Rule::op_add => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::ADD),
        Rule::op_adc => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::ADC),
        Rule::op_rsc => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::RSC),
        Rule::op_tst => assemble_two_arg_dp(&mut inner, src_span, DataProcessingOpcode::TST),
        Rule::op_teq => assemble_two_arg_dp(&mut inner, src_span, DataProcessingOpcode::TEQ),
        Rule::op_cmp => assemble_two_arg_dp(&mut inner, src_span, DataProcessingOpcode::CMP),
        Rule::op_cmn => assemble_two_arg_dp(&mut inner, src_span, DataProcessingOpcode::CMN),
        Rule::op_or => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::ORR),
        Rule::op_mov => assemble_two_arg_dp_dest(&mut inner, src_span, DataProcessingOpcode::MOV),
        Rule::op_bic => assemble_three_arg_dp(&mut inner, src_span, DataProcessingOpcode::BIC),
        Rule::op_mvn => assemble_two_arg_dp(&mut inner, src_span, DataProcessingOpcode::MVN),
        Rule::op_b => assemble_branch(&mut inner, src_span, false, labels, current_addr),
        Rule::op_bl => assemble_branch(&mut inner, src_span, true, labels, current_addr),
        _ => Err(span_err(opcode_span, "Invalid opcode")),
    }?;

    Ok(Instruction { condition, body })
}

impl Condition {
    fn from_rule(rule: Rule) -> Option<Self> {
        match rule {
            Rule::cond_eq => Some(Condition::EQ),
            Rule::cond_ne => Some(Condition::NE),
            Rule::cond_cs => Some(Condition::CS),
            Rule::cond_cc => Some(Condition::CC),
            Rule::cond_mi => Some(Condition::MI),
            Rule::cond_pl => Some(Condition::PL),
            Rule::cond_vs => Some(Condition::VS),
            Rule::cond_vc => Some(Condition::VC),
            Rule::cond_hi => Some(Condition::HI),
            Rule::cond_ls => Some(Condition::LS),
            Rule::cond_ge => Some(Condition::GE),
            Rule::cond_lt => Some(Condition::LT),
            Rule::cond_gt => Some(Condition::GT),
            Rule::cond_le => Some(Condition::LE),
            Rule::cond_al => Some(Condition::AL),
            _ => None
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
