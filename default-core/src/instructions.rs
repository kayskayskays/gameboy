use crate::instructions::BitwiseInstruction::{Rotate, ModifyBit, TestBit, Shift, Swap};
use crate::registers::{Register16, Register8};
use std::cmp::PartialEq;
use Instruction::*;

pub(super) enum Instruction {
    Load(Operand8, Operand8),
    Halt,

    Arithmetic(Operand8, ArithmeticOperationType, CarryMode),
    Logical(Operand8, LogicalInstructionType),
    Compare(Operand8),
    Bitwise(BitwiseInstruction)
}

pub(super) enum BitwiseInstruction {
    Rotate(Operand8, BitwiseDirection, RotationType),
    Shift(Operand8, BitwiseDirection, ShiftType),
    Swap(Operand8),
    TestBit(Operand8, u8),
    ModifyBit(Operand8, u8, SetMode),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(super) enum Operand8 {
    Immediate8(u8),
    Address(u16),
    Register(Register8),
    AddressHl,
}

#[derive(Eq, PartialEq)]
pub(super) enum Operand16 {
    Immediate16(u16),
    Register(Register16),
}

pub(super) enum LogicalInstructionType { And, Xor, Or }
pub(super) enum ArithmeticOperationType { Add, Sub }
pub(super) enum CarryMode { With, Without }
pub(super) enum ShiftType { Arithmetic, Logical }
pub(super) enum RotationType { Circular, Carry }
pub(super) enum BitwiseDirection { Left, Right }
pub(super) enum SetMode { Set, Unset }

const HL_ID: u8 = 0b110;

const ARITHMETIC_INSTRUCTION_CONSTRUCTORS: [fn(Operand8) -> Instruction; 8] = [
    |op| Arithmetic(op, ArithmeticOperationType::Add, CarryMode::Without), // ADD
    |op| Arithmetic(op, ArithmeticOperationType::Add, CarryMode::With),  // ADDC
    |op| Arithmetic(op, ArithmeticOperationType::Sub, CarryMode::Without), // SUB
    |op| Arithmetic(op, ArithmeticOperationType::Sub, CarryMode::With),  // SUBC
    |op| Logical(op, LogicalInstructionType::And), // AND
    |op| Logical(op, LogicalInstructionType::Xor), // XOR
    |op| Logical(op, LogicalInstructionType::Or),  // OR
    Compare
];

const BITWISE_INSTRUCTION_CONSTRUCTORS: [fn(Operand8) -> BitwiseInstruction; 8] = [
    |op| Rotate(op, BitwiseDirection::Left, RotationType::Circular),   // RLC
    |op| Rotate(op, BitwiseDirection::Right, RotationType::Circular),  // RRC
    |op| Rotate(op, BitwiseDirection::Left, RotationType::Carry),      // RL
    |op| Rotate(op, BitwiseDirection::Right, RotationType::Carry),     // RR

    |op| Shift(op, BitwiseDirection::Left, ShiftType::Arithmetic),     // SLA
    |op| Shift(op, BitwiseDirection::Right, ShiftType::Arithmetic),    // SRA
    Swap,                                                                       // SWAP
    |op| Shift(op, BitwiseDirection::Right, ShiftType::Logical),       // SRL
];

const BITWISE_SET_CONSTRUCTORS: [fn(Operand8, u8) -> BitwiseInstruction; 3] = [
    |op, idx| TestBit(op, idx),                       // BIT
    |op, idx| ModifyBit(op, idx, SetMode::Unset),     // RES
    |op, idx| ModifyBit(op, idx, SetMode::Set),       // SET
];

impl Instruction {
    fn decode_operand(operand_id: u8) -> Operand8 {
        let operand_id = operand_id & 0b111;

        Register8::from_code(operand_id)
            .map_or_else(
                || {
                    if operand_id != HL_ID {
                        panic!("Could not resolve operand with id: {:#06x}", operand_id);
                    }

                    // The idea is: either, we were able to resolve the operand,
                    // or we get back an `AddressHL`. The `HL` register is
                    // generally used for address lookups, so we keep it
                    // separate from the other register operands.
                    Operand8::AddressHl
                },
                Operand8::Register
            )
    }
    pub(super) fn decode_load(opcode: u8) -> Option<Instruction> {
        if !(0x40..=0x7F).contains(&opcode) { return None }

        let dst = Instruction::decode_operand(opcode >> 3);
        let src = Instruction::decode_operand(opcode);

        let instruction = match (dst, src) {
            (Operand8::AddressHl, Operand8::AddressHl) => Halt,
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