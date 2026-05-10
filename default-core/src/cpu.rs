use super::instructions::Instruction::{self, *};
use super::registers::{flags::Flags, Register8::*, Registers};
use crate::address_bus::AddressBus;
use gameboy_core_interface::GameboyCore;


struct Cpu {
    address_bus: AddressBus,
    registers: Registers,
    program_counter: u16,
}

impl GameboyCore for Cpu {
    fn load_rom(&mut self, rom: &[u8]) {
        self.address_bus.load_rom(rom)
    }

    fn step(&mut self) {
        self.step()
    }
}

impl Cpu {
    fn new() -> Self {
        Cpu {
            address_bus: AddressBus::new(),
            registers: Registers::new(),
            program_counter: 0,
        }
    }

    fn step(&mut self) {
        let opcode = self.address_bus.read(self.program_counter);
        self.program_counter += 1;

        let instruction = match opcode {
            0x40..=0x7F => Instruction::decode_load(opcode),
            0x80..=0xBF => Instruction::decode_alu(opcode),
            0xCB => {
                let cb_opcode = self.address_bus.read(self.program_counter);
                self.program_counter += 1;
                Instruction::decode_cb(cb_opcode)
            }
            _ => None,
        };

        if let Some(instruction) = instruction {
            self.execute(instruction);
        } else {
            self.execute_raw(opcode);
        }
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
        }
    }

    fn execute_raw(&mut self, opcode: u8) {
        match opcode {
            _ => {}
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let a_value = self.registers.read8(A);
        let (new_value, carry) = a_value.overflowing_add(value);

        let half_carry = (a_value & 0xF) + (value & 0xF) > 0xF;
        self.registers.write8(
            F,
            Flags::new(false, false, half_carry, carry).into()
        );

        new_value
    }
}