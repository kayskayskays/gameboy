use std::ops::{Index, IndexMut};
use crate::registers::flags::Flags;

pub(super) mod flags;

#[derive(Eq, PartialEq)]
pub(crate) enum Register8 {
    A, B, C, D, E, H, L, F
}

impl Register8 {
    pub(crate) fn from_code(code: u8) -> Option<Register8> {
        match code & 0b111 {
            0b111 => Some(Register8::A),
            0b000 => Some(Register8::B),
            0b001 => Some(Register8::C),
            0b010 => Some(Register8::D),
            0b011 => Some(Register8::E),
            0b100 => Some(Register8::H),
            0b101 => Some(Register8::L),
            _ => None,
        }
    }
}

impl Register8 {
    fn index(&self) -> u8 {
        match self {
            Register8::A => 0,
            Register8::B => 1,
            Register8::C => 2,
            Register8::D => 3,
            Register8::E => 4,
            Register8::H => 5,
            Register8::L => 6,
            Register8::F => 7,
        }
    }
}

pub(super) enum RegisterPair {
    BC,
    DE,
    HL,
    AF,
}

pub(super) enum Register16 {
    Pair(RegisterPair),
    StackPointer,
}

impl From<RegisterPair> for Register16 {
    fn from(pair: RegisterPair) -> Self {
        Register16::Pair(pair)
    }
}

impl RegisterPair {
    fn lo_and_hi_registers(&self) -> (Register8, Register8) {
        match self {
            RegisterPair::BC => (
                Register8::B,
                Register8::C
            ),
            RegisterPair::DE => (
                Register8::D,
                Register8::E
            ),
            RegisterPair::HL => (
                Register8::H,
                Register8::L
            ),
            RegisterPair::AF => (
                Register8::A,
                Register8::F
            )
        }
    }

    fn from_dd_code(code: u8) -> Option<Register16> {
        match code & 0b11 {
            0b00 => Some(RegisterPair::BC.into()),
            0b01 => Some(RegisterPair::DE.into()),
            0b10 => Some(RegisterPair::HL.into()),
            0b11 => Some(Register16::StackPointer),
            _ => None
        }
    }

    fn from_qq_code(code: u8) -> Option<Register16> {
        match code & 0b11 {
            0b00 => Some(RegisterPair::BC.into()),
            0b01 => Some(RegisterPair::DE.into()),
            0b10 => Some(RegisterPair::HL.into()),
            0b11 => Some(RegisterPair::AF.into()),
            _ => None
        }
    }
}

pub(super) struct Registers {
    data: [u8; 8],
    stack_pointer: u16,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            data: [0; 8],
            stack_pointer: 0,
        }
    }

    pub fn flags(&self) -> Flags {
        Flags::from(self[Register8::F])
    }

    pub(super) fn read8(&self, register: Register8) -> u8 {
        self[register]
    }

    pub(super) fn write8(&mut self, register: Register8, value: u8) {
        self[register] = value;
    }

    pub(super) fn read16(&self, register: Register16) -> u16 {
        match register {
            Register16::Pair(register_pair) => {
                let (lo, hi) = register_pair.lo_and_hi_registers();
                (self.read8(hi) as u16) << 8 | (self.read8(lo) as u16)
            },
            Register16::StackPointer => self.stack_pointer,
        }
    }

    pub(super) fn write16(&mut self, register: Register16, value: u16) {
        match register {
            Register16::Pair(register_pair) => {
                let (lo, hi) = register_pair.lo_and_hi_registers();
                self.write8(lo, (value & 0xFF) as u8);
                self.write8(hi, (value >> 8) as u8);
            },
            Register16::StackPointer => self.stack_pointer = value,
        }
    }
}

impl Index<Register8> for Registers {
    type Output = u8;

    fn index(&self, index: Register8) -> &Self::Output {
        &self.data[index.index() as usize]
    }
}

impl IndexMut<Register8> for Registers {
    fn index_mut(&mut self, index: Register8) -> &mut Self::Output {
        &mut self.data[index.index() as usize]
    }
}