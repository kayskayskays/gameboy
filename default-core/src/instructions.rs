use crate::registers::Register8;
use std::cmp::PartialEq;

pub(super) enum Instruction {
    Load(Operand, Operand),
    Halt
}

#[derive(Eq, PartialEq)]
pub(super) enum Operand {
    Register(Register8),
    HL,
}

const HL_ID: u8 = 0b110;

impl Instruction {
    fn decode_load_operand(operand_id: u8) -> Operand {
        let operand_id = operand_id & 0b111;

        Register8::from_code(operand_id)
            .map_or_else(
                || {
                    if operand_id != HL_ID {
                        panic!("Could not resolve operand with id: {:#06x}", operand_id);
                    }
                    Operand::HL
                },
                |register| Operand::Register(register)
            )
    }
    pub(super) fn decode_load(opcode: u8) -> Instruction {
        let first_operand = Instruction::decode_load_operand(opcode >> 3);
        let second_operand = Instruction::decode_load_operand(opcode);

        if first_operand == Operand::HL && second_operand == Operand::HL {
            Instruction::Halt
        } else {
            Instruction::Load(first_operand, second_operand)
        }
    }

    pub(super) fn decode_accumulator(opcode: u8) -> Instruction {
        todo!()
    }
    
    pub(super) fn decode_cb(opcode: u8) -> Instruction {
        todo!()
    }
}