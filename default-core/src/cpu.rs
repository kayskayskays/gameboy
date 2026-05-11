use super::instructions::Instruction::{self, *};
use super::registers::{flags::Flags, Register8, RegisterPair, Registers};
use crate::address_bus::AddressBus;
use crate::instructions::{Carry, Operand};
use gameboy_core_interface::GameboyCore;

struct Cpu {
    address_bus: AddressBus,
    registers: Registers,
    program_counter: u16,
    halted: bool,
}

struct CarryStatus(bool, bool);

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
            halted: false,
        }
    }

    fn step(&mut self) {
        if self.halted {
            return;
        }

        let opcode = self.address_bus.read(self.program_counter);
        self.program_counter += 1;

        let instruction = match opcode {
            0x40..=0x7F => Some(Instruction::decode_load(opcode)),
            0x80..=0xBF => Some(Instruction::decode_arithmetic(opcode)),
            0xCB => {
                let cb_opcode = self.address_bus.read(self.program_counter);
                self.program_counter += 1;
                Some(Instruction::decode_cb(cb_opcode))
            }
            _ => None,
        };

        if let Some(instruction) = instruction {
            self.execute(instruction);
        } else {
            self.execute_raw(opcode);
        }
    }

    fn carry_status(x: u8, y: u8, carry: u8) -> CarryStatus {
        let half_carry = (x & 0xF) + (y & 0xF) + carry > 0xF;
        let carry = x as u16 + y as u16 + carry as u16 > 0xFF;
        CarryStatus(half_carry, carry)
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Load(a, b) => self.execute_load(a, b),
            Halt => self.halted = false,
            Add(op, carry) => self.execute_add(op, carry),
            _ => todo!()
        }
    }

    fn execute_add(&mut self, operand: Operand, carry: Carry) {
        let current_value = self.registers.read8(Register8::A);

        let value_to_add = match operand {
            Operand::Register(register) => self.registers.read8(register),
            Operand::HL => self.address_bus.read(self.address_from_hl())
        };

        let carry = if let Carry::TRUE = carry { 1 } else { 0 };

        let carry_status = Cpu::carry_status(current_value, value_to_add, carry);
        let sum = current_value.wrapping_add(carry).wrapping_add(value_to_add);

        let flags = Flags::new(
            sum == 0,
            false,
            carry_status.0,
            carry_status.1
        );

        self.registers.write8(Register8::F, flags.into());
        self.registers.write8(Register8::A, sum)
    }

    fn address_from_hl(&self) -> u16 {
        self.registers.read16(RegisterPair::HL.into())
    }

    fn execute_load(&mut self, first_operand: Operand, second_operand: Operand) {
        match (first_operand, second_operand) {
            (Operand::Register(a), Operand::HL) => {
                let address = self.address_from_hl();
                let value = self.address_bus.read(address);
                self.registers.write8(a, value);
            },
            (Operand::Register(a), Operand::Register(b)) => {
                let value = self.registers.read8(b);
                self.registers.write8(a, value);
            },
            (Operand::HL, Operand::Register(b)) => {
                let address = self.address_from_hl();
                let value = self.registers.read8(b);
                self.address_bus.write(address, value);
            },
            _ => unreachable!()
        }
    }

    fn execute_raw(&mut self, opcode: u8) {
        match opcode {
            _ => {}
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let a_value = self.registers.read8(Register8::A);
        let (new_value, carry) = a_value.overflowing_add(value);

        let half_carry = (a_value & 0xF) + (value & 0xF) > 0xF;
        self.registers.write8(
            Register8::F,
            Flags::new(false, false, half_carry, carry).into()
        );

        new_value
    }
}