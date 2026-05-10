use super::registers::{Register16, Register8};

pub(super) enum Instruction {}

impl Instruction {
    pub(super) fn decode_load(opcode: u8) -> Option<Instruction> {
        None
    }

    pub(super) fn decode_alu(opcode: u8) -> Option<Instruction> {
        None
    }
    
    pub(super) fn decode_cb(opcode: u8) -> Option<Instruction> {
        None
    }
}