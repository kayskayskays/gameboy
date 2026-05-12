use super::instructions::Instruction::{self, *};
use super::registers::{flags::Flags, Register8, RegisterPair, Registers};
use crate::address_bus::AddressBus;
use crate::instructions::{Carry, ArithmeticOpType, Operand, LogicalOpType};
use gameboy_core_interface::GameboyCore;

struct Cpu {
    address_bus: AddressBus,
    registers: Registers,
    program_counter: u16,
    halted: bool,
}

struct CarryStatus {
    half_carry: bool,
    carry: bool,
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
            halted: false,
        }
    }

    fn step(&mut self) {
        if self.halted { return }

        let opcode = self.address_bus.read(self.program_counter);
        self.program_counter += 1;

        let instruction = Instruction::decode_load(opcode)
            .or_else(|| Instruction::decode_arithmetic(opcode))
            .or_else(|| Instruction::decode_cb(opcode, || {
                self.program_counter += 1;
                self.address_bus.read(self.program_counter)
            }));

        if let Some(instruction) = instruction {
            self.execute(instruction);
        } else {
            self.execute_raw(opcode);
        }
    }

    fn carry_status(x: u8, y: u8, carry: u8, add: bool) -> CarryStatus {
        let mut status = CarryStatus { half_carry: false, carry: false };

        if add {
            status.half_carry = (x & 0xF) + (y & 0xF) + carry > 0xF;
            status.carry = x as u16 + y as u16 + carry as u16 > 0xFF;
        } else {
            status.half_carry = (x & 0xF) < ((y & 0xF) + carry);
            status.carry = (x as u16) < (y as u16 + carry as u16)
        }

        status
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Load(a, b) => self.execute_load(a, b),
            Halt => self.halted = true,
            ArithmeticOp(op, carry, op_type) =>
                self.execute_arithmetic_op(op, carry, op_type),
            LogicalOp(op, op_type) =>
                self.execute_logical(op, op_type),
            _ => todo!()
        }
    }

    fn read_value_from_operand(&self, operand: Operand) -> u8 {
        match operand {
            Operand::Register(register) => self.registers.read8(register),
            Operand::HL => self.address_bus.read(self.address_from_hl())
        }
    }

    fn address_from_hl(&self) -> u16 {
        self.registers.read16(RegisterPair::HL.into())
    }

    fn execute_arithmetic_op(&mut self, operand: Operand, carry: Carry, op_type: ArithmeticOpType) {
        let current_value = self.registers.read8(Register8::A);
        let new_value = self.read_value_from_operand(operand);

        let carry = if let Carry::TRUE = carry { 1 } else { 0 };

        let add = matches!(op_type, ArithmeticOpType::ADD);
        let carry_status = Cpu::carry_status(current_value, new_value, carry, add);

        let result = if add {
            current_value.wrapping_add(new_value).wrapping_add(carry)
        }  else {
            current_value.wrapping_sub(new_value).wrapping_sub(carry)
        };

        let flags = Flags::new(
            result == 0,
            !add,
            carry_status.half_carry,
            carry_status.carry
        );

        self.registers.write8(Register8::F, flags.into());
        self.registers.write8(Register8::A, result)
    }

    fn execute_logical(&mut self, operand: Operand, logical_op_type: LogicalOpType) {
        let current_value = self.registers.read8(Register8::A);
        let new_value = self.read_value_from_operand(operand);

        let result =  match logical_op_type {
            LogicalOpType::AND => current_value & new_value,
            LogicalOpType::XOR => current_value ^ new_value,
            LogicalOpType::OR => current_value | new_value,
        };

        self.registers.write8(Register8::A, result)
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
}