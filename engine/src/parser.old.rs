use nom::{branch::alt, bytes::complete::{tag, take_until}, character::complete::{alphanumeric1, digit1}, combinator::{into, map, map_res}, number::complete::u8, sequence::{delimited, preceded, terminated}, IResult, Parser};

type Label = u32;

#[derive(Debug)]
enum Instruction {
    // AQA base
    LDR(Register, IndirectAddr),
    STR(Register, IndirectAddr),
    ADD(Register, Register, DataSource),
    SUB(Register, Register, DataSource),
    MOV(Register, DataSource),
    CMP(Register, DataSource),
    B(Label),
    Branch(BranchType, Label),
    AND(Register, Register, DataSource),
    ORR(Register, Register, DataSource),
    EOR(Register, Register, DataSource),
    MVN(Register, DataSource),
    LSL(Register, Register, DataSource),
    LSR(Register, Register, DataSource),
    HALT,
}

#[derive(Debug)]
enum BranchType {
    EQ,
    NE,
    GT,
    LT
}

#[derive(Debug, PartialEq, Eq)]
pub struct Register(u8);

#[derive(Debug)]
enum DataSource {
    Literal(u32),
    Register(Register)
}

#[derive(Debug)]
struct IndirectAddr {
    base: DataSource,
    offset: AddrOffset,
    negative: bool,
}

#[derive(Debug)]
enum AddrOffset {
    Immediate(u32),
    Register(u32, Register)
}

fn register(input: &str) -> IResult<&str, Register> {
    preceded(tag("R"), 
        map(decimal, |out: u32| Register(out as u8))
    ).parse(input)
}

fn literal(input: &str) -> IResult<&str, u32> {
    preceded(tag("#"), 
        decimal
    ).parse(input)
}

fn decimal(input: &str) -> IResult<&str, u32> {
    map_res(
        digit1,
        |out: &str| u32::from_str_radix(out, 10)
    ).parse(input)
}

fn label(input: &str) -> IResult<&str, String> {
    into(
        terminated(
            alphanumeric1,
            tag(":"))
        )
        .parse(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    preceded(
        alt((
            tag(";"),
            tag("//")
        )), take_until("\n")).parse(input)
}

fn indirect_addr(input: &str) -> IResult<&str, IndirectAddr> {
    alt((
        map(literal, |lit| IndirectAddr {
            base: DataSource::Literal(lit),
            negative: false,
            offset: AddrOffset::Immediate(0)
        }),
        delimited(
            tag("["),
            alt((
                map(register, |reg| IndirectAddr {
                    base: DataSource::Register(reg),
                    negative: false,
                    offset: AddrOffset::Immediate(0)
                }),
                map(register, |reg| IndirectAddr {
                    base: DataSource::Register(reg),
                    negative: false,
                    offset: AddrOffset::Immediate(0)
                })
            )),
            tag("]")
        )
    )).parse(input)
}

fn data_source(input: &str) -> IResult<&str, DataSource> {
    alt((
        map(literal, DataSource::Literal),
        map(register, DataSource::Register)
    )).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register() {
        let res = register("R5").unwrap();

        assert_eq!(res.1, Register(5))
    }
}