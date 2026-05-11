use crate::registers::Register8;
use std::cmp::PartialEq;
use Instruction::{ArithmeticOp, Compare, LogicalOp, Rotate, SetBit, SetZ, Shift, Swap};

pub(super) enum Instruction {
    Load(Operand, Operand),
    Halt,

    ArithmeticOp(Operand, Carry, ArithmeticOpType),
    LogicalOp(Operand, LogicalOpType),
    Compare(Operand),

    Rotate(Operand, BitwiseDirection, RotationType),
    Shift(Operand, BitwiseDirection, ShiftType),
    Swap(Operand),
    SetZ(Operand, u8),
    SetBit(Operand, u8, SetType),
}

pub(super) enum LogicalOpType { AND, XOR, OR }

trait Execute {
    fn apply(&self, val: u8) -> u8;
}

pub(super) enum ArithmeticOpType { ADD, SUB }

pub(super) enum Carry { TRUE, FALSE }
pub(super) enum ShiftType { ARITHMETIC, LOGICAL }
pub(super) enum RotationType { CIRCULAR, CARRY }
pub(super) enum BitwiseDirection { LEFT, RIGHT }
pub(super) enum SetType { SET, UNSET }

#[derive(Eq, PartialEq)]
pub(super) enum Operand {
    Register(Register8),
    HL,
}

const HL_ID: u8 = 0b110;

const ARITHMETIC_INSTRUCTION_CONSTRUCTORS: [fn(Operand) -> Instruction; 8] = [
    |op| ArithmeticOp(op, Carry::FALSE, ArithmeticOpType::ADD), // ADD
    |op| ArithmeticOp(op, Carry::TRUE, ArithmeticOpType::ADD),  // ADDC
    |op| ArithmeticOp(op, Carry::FALSE, ArithmeticOpType::SUB), // SUB
    |op| ArithmeticOp(op, Carry::TRUE, ArithmeticOpType::SUB),  // SUBC
    |op| LogicalOp(op, LogicalOpType::AND), // AND
    |op| LogicalOp(op, LogicalOpType::XOR), // XOR
    |op| LogicalOp(op, LogicalOpType::OR),  // OR
    Compare // CP
];

const BITWISE_INSTRUCTION_CONSTRUCTORS: [fn(Operand) -> Instruction; 8] = [
    |op| Rotate(op, BitwiseDirection::LEFT, RotationType::CIRCULAR),   // RLC
    |op| Rotate(op, BitwiseDirection::RIGHT, RotationType::CIRCULAR),  // RRC
    |op| Rotate(op, BitwiseDirection::LEFT, RotationType::CARRY),      // RL
    |op| Rotate(op, BitwiseDirection::RIGHT, RotationType::CARRY),     // RR

    |op| Shift(op, BitwiseDirection::LEFT, ShiftType::ARITHMETIC),     // SLA
    |op| Shift(op, BitwiseDirection::RIGHT, ShiftType::ARITHMETIC),    // SRA
    Swap,                                                                       // SWAP
    |op| Shift(op, BitwiseDirection::RIGHT, ShiftType::LOGICAL),       // SRL
];

const BITWISE_SET_CONSTRUCTORS: [fn(Operand, u8) -> Instruction; 3] = [
    |op, idx| SetZ(op, idx),                       // BIT
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
                    // or we get back a `HL`. `HL` may have special
                    // interpretation for certain instructions, so we keep it
                    // separate from the standard `Register` operands.
                    Operand::HL
                },
                |register| Operand::Register(register)
            )
    }
    pub(super) fn decode_load(opcode: u8) -> Instruction {
        assert!((0x40..=0x7F).contains(&opcode));

        let first_operand = Instruction::decode_operand(opcode >> 3);
        let second_operand = Instruction::decode_operand(opcode);

        if first_operand == Operand::HL && second_operand == Operand::HL {
            // Seeing the HL operand twice corresponds to a HALT instruction.
            Instruction::Halt
        } else {
            Instruction::Load(first_operand, second_operand)
        }
    }

    pub(super) fn decode_arithmetic(opcode: u8) -> Instruction {
        assert!((0x80..=0xBF).contains(&opcode));

        let idx = ((opcode >> 3) & 0b111) as usize;
        let operand = Instruction::decode_operand(opcode);
        ARITHMETIC_INSTRUCTION_CONSTRUCTORS[idx](operand)
    }
    
    pub(super) fn decode_cb(opcode: u8) -> Instruction {
        match opcode {
            0x00..=0x3F => {
                let idx = ((opcode >> 3) & 0b111) as usize;
                let operand = Instruction::decode_operand(opcode);
                BITWISE_INSTRUCTION_CONSTRUCTORS[idx](operand)
            },
            _ => {
                let idx = (opcode >> 6) as usize;
                let bit_idx = (opcode >> 3) & 0b111;
                let operand = Instruction::decode_operand(opcode);
                BITWISE_SET_CONSTRUCTORS[idx](operand, bit_idx)
            }
        }
    }
}