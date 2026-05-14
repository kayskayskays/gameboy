use crate::registers::Register8;
use std::cmp::PartialEq;
use Instruction::*;
use crate::instructions::BitwiseInstruction::{Rotate, SetBit, SetZero, Shift, Swap};

pub(super) enum Instruction {
    Load(Operand, Operand),
    Halt,

    Arithmetic(Operand, ArithmeticOperationType, Carry),
    Logical(Operand, LogicalInstructionType),
    Compare(Operand),
    Bitwise(BitwiseInstruction)
}

pub(super) enum BitwiseInstruction {
    Rotate(Operand, BitwiseDirection, RotationType),
    Shift(Operand, BitwiseDirection, ShiftType),
    Swap(Operand),
    SetZero(Operand, u8),
    SetBit(Operand, u8, SetType),
}

pub(super) enum LogicalInstructionType { AND, XOR, OR }

pub(super) enum ArithmeticOperationType { ADD, SUB }

pub(super) enum Carry { TRUE, FALSE }
pub(super) enum ShiftType { ARITHMETIC, LOGICAL }
pub(super) enum RotationType { CIRCULAR, CARRY }
pub(super) enum BitwiseDirection { LEFT, RIGHT }
pub(super) enum SetType { SET, UNSET }

#[derive(Eq, PartialEq)]
pub(super) enum Operand {
    Immediate8(u8),
    Register(Register8),
    HL,
}

const HL_ID: u8 = 0b110;

const ARITHMETIC_INSTRUCTION_CONSTRUCTORS: [fn(Operand) -> Instruction; 8] = [
    |op| Arithmetic(op, ArithmeticOperationType::ADD, Carry::FALSE), // ADD
    |op| Arithmetic(op, ArithmeticOperationType::ADD, Carry::TRUE),  // ADDC
    |op| Arithmetic(op, ArithmeticOperationType::SUB, Carry::FALSE), // SUB
    |op| Arithmetic(op, ArithmeticOperationType::SUB, Carry::TRUE),  // SUBC
    |op| Logical(op, LogicalInstructionType::AND), // AND
    |op| Logical(op, LogicalInstructionType::XOR), // XOR
    |op| Logical(op, LogicalInstructionType::OR),  // OR
    Compare
];

const BITWISE_INSTRUCTION_CONSTRUCTORS: [fn(Operand) -> BitwiseInstruction; 8] = [
    |op| Rotate(op, BitwiseDirection::LEFT, RotationType::CIRCULAR),   // RLC
    |op| Rotate(op, BitwiseDirection::RIGHT, RotationType::CIRCULAR),  // RRC
    |op| Rotate(op, BitwiseDirection::LEFT, RotationType::CARRY),      // RL
    |op| Rotate(op, BitwiseDirection::RIGHT, RotationType::CARRY),     // RR

    |op| Shift(op, BitwiseDirection::LEFT, ShiftType::ARITHMETIC),     // SLA
    |op| Shift(op, BitwiseDirection::RIGHT, ShiftType::ARITHMETIC),    // SRA
    Swap,                                                                       // SWAP
    |op| Shift(op, BitwiseDirection::RIGHT, ShiftType::LOGICAL),       // SRL
];

const BITWISE_SET_CONSTRUCTORS: [fn(Operand, u8) -> BitwiseInstruction; 3] = [
    |op, idx| SetZero(op, idx),                       // BIT
    |op, idx| SetBit(op, idx, SetType::UNSET),     // RES
    |op, idx| SetBit(op, idx, SetType::SET),       // SET
];

impl Instruction {
    fn decode_operand(operand_id: u8) -> Operand {
        let operand_id = operand_id & 0b111;

        Register8::from_code(operand_id)
            .map_or_else(
                || {
                    if operand_id != HL_ID {
                        panic!("Could not resolve operand with id: {:#06x}", operand_id);
                    }

                    // The idea is: either, we were able to resolve the operand,
                    // or we get back a `HL`. The `HL` register is generally
                    // used for address lookups, so we keep it separate from the
                    // standard `Register` operands.
                    Operand::HL
                },
                Operand::Register
            )
    }
    pub(super) fn decode_load(opcode: u8) -> Option<Instruction> {
        if !(0x40..=0x7F).contains(&opcode) { return None }

        let dst = Instruction::decode_operand(opcode >> 3);
        let src = Instruction::decode_operand(opcode);

        let instruction = match (dst, src) {
            (Operand::HL, Operand::HL) => Halt,
            (dst, src) => Load(dst, src),
        };
        
        Some(instruction)
    }

    pub(super) fn decode_arithmetic(opcode: u8) -> Option<Instruction> {
        if !(0x80..=0xBF).contains(&opcode) { return None }
        
        let idx = ((opcode >> 3) & 0b111) as usize;
        let operand = Instruction::decode_operand(opcode);
        Some(ARITHMETIC_INSTRUCTION_CONSTRUCTORS[idx](operand))
    }

    pub(super) fn decode_bitwise<T>(opcode: u8, next_opcode_supplier: T) -> Option<Instruction>
    where
        T: FnOnce() -> u8,
    {
        if opcode != 0xCB { return None }

        let opcode = next_opcode_supplier();
        let operand = Instruction::decode_operand(opcode);

        let instruction = match opcode {
            0x00..=0x3F => {
                let idx = ((opcode >> 3) & 0b111) as usize;
                BITWISE_INSTRUCTION_CONSTRUCTORS[idx](operand)
            },
            _ => {
                let idx = (opcode >> 6) as usize;
                let bit_idx = (opcode >> 3) & 0b111;
                BITWISE_SET_CONSTRUCTORS[idx](operand, bit_idx)
            }
        };

        Some(Bitwise(instruction))
    }
}