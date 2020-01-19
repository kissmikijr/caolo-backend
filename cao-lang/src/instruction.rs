use crate::scalar::Scalar;
use crate::traits::ByteEncodeProperties;
use crate::{subprogram_description, SubProgram};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
/// Single instruction of the interpreter
pub enum Instruction {
    /// The first node executed
    Start = 0,
    /// Add two numbers
    Add = 1,
    /// Subtract two numbers
    Sub = 2,
    /// Multiply two numbers
    Mul = 5,
    /// Divide the first number by the second
    Div = 7,
    /// Call a function provided by the runtime
    /// Requires function name as a string as input
    Call = 9,
    /// Push an int onto the stack
    ScalarInt = 10,
    /// Push a float onto the stack
    ScalarFloat = 11,
    /// Push a label onto the stack
    ScalarLabel = 17,
    /// Pop the next N (positive integer) number of items from the stack and write them to memory
    /// Push the pointer to the beginning of the array onto the stack
    ScalarArray = 13,
    /// Writes the strings followed by the instruction to memory and pushes the pointer pointing to
    /// it onto the stack
    StringLiteral = 19,
    /// Empty instruction that has no effects
    Pass = 14,
    /// Clones the last element on the stack
    /// Does nothing if no elements are on the stack
    CopyLast = 15,
    /// If the value at the top of the stack is truthy jumps to the input node
    /// Else does nothing
    JumpIfTrue = 16,
    /// Quit the program
    /// Implicitly inserted by the compiler after every leaf node
    Exit = 18,
    /// Jump to the label on top of the stack
    Jump = 20,
    /// Write the top value on the stack to the given register
    WriteReg = 21,
    /// Read the value in the given register and push it on the stack
    ReadReg = 22,
    /// Compares two scalars
    Equals = 23,
    /// Compares two scalars
    NotEquals = 24,
    /// Is the first param less than the second?
    Less = 25,
    /// Is the first param less than or equal to the second?
    LessOrEq = 26,
}

impl TryFrom<u8> for Instruction {
    type Error = String;

    fn try_from(c: u8) -> Result<Instruction, Self::Error> {
        use Instruction::*;
        match c {
            0 => Ok(Start),
            1 => Ok(Add),
            2 => Ok(Sub),
            5 => Ok(Mul),
            7 => Ok(Div),
            9 => Ok(Call),
            10 => Ok(ScalarInt),
            11 => Ok(ScalarFloat),
            13 => Ok(ScalarArray),
            14 => Ok(Pass),
            15 => Ok(CopyLast),
            16 => Ok(JumpIfTrue),
            17 => Ok(ScalarLabel),
            18 => Ok(Exit),
            19 => Ok(StringLiteral),
            20 => Ok(Jump),
            21 => Ok(WriteReg),
            22 => Ok(ReadReg),
            23 => Ok(Equals),
            24 => Ok(NotEquals),
            25 => Ok(Less),
            26 => Ok(LessOrEq),
            _ => Err(format!("Unrecognized instruction [{}]", c)),
        }
    }
}

pub fn get_instruction_descriptions() -> Vec<SubProgram<'static>> {
    vec![
        subprogram_description!(
            Instruction::Add,
            "Add two scalars",
            [Scalar, Scalar],
            [Scalar]
        ),
        subprogram_description!(
            Instruction::Sub,
            "Subtract two scalars",
            [Scalar, Scalar],
            [Scalar]
        ),
        subprogram_description!(
            Instruction::Mul,
            "Multiply two scalars",
            [Scalar, Scalar],
            [Scalar]
        ),
        subprogram_description!(
            Instruction::Div,
            "Divide two scalars",
            [Scalar, Scalar],
            [Scalar]
        ),
        subprogram_description!(Instruction::Start, "Start of the program", [], []),
        subprogram_description!(Instruction::Pass, "Do nothing", [], []),
        subprogram_description!(Instruction::ScalarInt, "Make an integer", [], [Scalar]),
        subprogram_description!(Instruction::ScalarFloat, "Make a real number", [], [Scalar]),
        subprogram_description!(Instruction::StringLiteral, "Make a text", [], [Scalar]),
        subprogram_description!(
            Instruction::JumpIfTrue,
            "Jump to the input node if the last value is true else do nothing.",
            [Scalar],
            []
        ),
        subprogram_description!(
            Instruction::Equals,
            "Return 1 if the inputs are equal, 0 otherwise",
            [Scalar, Scalar],
            [Scalar]
        ),
        subprogram_description!(
            Instruction::NotEquals,
            "Return 0 if the inputs are equal, 1 otherwise",
            [Scalar, Scalar],
            [Scalar]
        ),
        subprogram_description!(
            Instruction::Less,
            "Return 1 if the first input is less than the second, 0 otherwise",
            [Scalar, Scalar],
            [Scalar]
        ),
        subprogram_description!(
            Instruction::LessOrEq,
            "Return 1 if the first input is less than or equal to the second, 0 otherwise",
            [Scalar, Scalar],
            [Scalar]
        ),
    ]
}
