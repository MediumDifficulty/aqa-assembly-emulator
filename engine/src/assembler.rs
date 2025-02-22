use std::collections::HashMap;

use pest::{
    error::ErrorVariant,
    iterators::{Pair, Pairs},
    Parser, Span,
};

use crate::{
    parser::{self, AssemblyParser, Rule}, unwrap_or_continue, unwrap_or_return, Condition, DataProcessing, DataProcessingOpcode, DataProcessingOperand, Instruction, InstructionBody, Program, Register, Shift
};

const MAX_REG_NUM: u8 = 15;

type Res<T> = Result<T, pest::error::Error<parser::Rule>>;

pub fn assemble(src: &str) -> Res<Program> {
    let parsed = AssemblyParser::parse(Rule::program, src)?.next().unwrap();

    let mut labels = HashMap::new();
    let mut instructions = Vec::new();

    for (i, line) in parsed.into_inner().enumerate() {
        let line = unwrap_or_continue!(line.into_inner().next());
        match line.as_rule() {
            Rule::label => {
                let label = line.into_inner().next().expect("Invalid label").as_str();
                labels.insert(label.to_string(), i);
            }
            Rule::instruction => {
                instructions.push(assemble_instruction(line)?);
            }
            _ => unreachable!(),
        }
    }

    Ok(Program { instructions })
}

pub fn lint_line(line: &str) -> Res<()> {
    let parsed = 
        unwrap_or_return!(
            unwrap_or_return!(
                AssemblyParser::parse(Rule::lint_line, line)?
                    .next(),
                Ok(())
            )
            .into_inner()
            .next(),
            Ok(())
        );

    match parsed.as_rule() {
        Rule::label => Ok(()),
        Rule::instruction => assemble_instruction(parsed).map(|_| ()),
        Rule::EOI => Ok(()),
        _ => unreachable!(),
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

fn assemble_instruction(src: Pair<'_, parser::Rule>) -> Res<Instruction> {
    let src_span = src.as_span();
    let mut inner = src.into_inner();
    let opcode_pair = inner.next().ok_or(span_err(src_span, "Missing opcode"))?;
    let opcode = opcode_pair.as_str();
    match opcode {
        "mov" => assemble_mov(&mut inner, src_span),
        _ => Err(span_err(opcode_pair.as_span(), "Invalid opcode")),
    }
}

fn assemble_mov(pairs: &mut Pairs<'_, Rule>, span: Span<'_>) -> Res<Instruction> {
    let dest_reg = pairs.next().ok_or(span_err(span, "Missing destination"))?;
    let src = pairs.next().ok_or(span_err(span, "Missing source"))?;

    let dest = parse_reg(dest_reg)?;

    let operand = match src.as_rule() {
        Rule::literal => DataProcessingOperand::Immediate {
            rotate: 0,
            value: parse_literal(src)? as u8,
        },
        Rule::register => DataProcessingOperand::Register {
            shift: Shift {
                amount: crate::ShiftAmount::Immediate(0),
                ty: crate::ShiftType::LogicalLeft
            },
            register: parse_reg(src)?,
        },
        _ => Err(span_err(src.as_span(), "Invalid source"))?,
    };

    Ok(Instruction {
        condition: Condition::default(),
        instruction_body: InstructionBody::DataProcessing(DataProcessing {
            dest,
            opcode: DataProcessingOpcode::MOV,
            operand,
            set_condition_codes: false,
            register: Register(0) // TODO: Expand instruction to use this as extra immediate space
        })
    })
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
    let literal = literal.into_inner()
        .next().expect("Invalid literal");

    match literal.as_rule() {
        Rule::decimal_literal => literal
            .into_inner()
            .next()
            .expect("Invalid literal")
            .as_str()
            .parse()
            .or(Err(span_err(span, "Invalid decimal literal"))),
        Rule::hex_literal => u32::from_str_radix(
            literal
                .into_inner()
                .next()
                .expect("Invalid literal")
                .as_str(),
            16,
        )
        .or(Err(span_err(span, "Invalid hex literal"))),
        _ => unreachable!(),
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
