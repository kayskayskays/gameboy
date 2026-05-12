use super::instructions::Instruction::{self, *};
use super::registers::{flags::Flags, Register8, RegisterPair, Registers};
use crate::address_bus::AddressBus;
use crate::instructions::{ArithmeticOperationType, BitwiseDirection, BitwiseInstruction, Carry, LogicalInstructionType, Operand, RotationType, SetType, ShiftType};
use gameboy_core_interface::GameboyCore;
use BitwiseInstruction::{Rotate, SetBit, SetZero, Shift, Swap};

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

impl CarryStatus {
    fn compute(x: u8, y: u8, carry: u8, add: bool) -> Self {
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
            .or_else(|| {
                Instruction::decode_bitwise(opcode, || {
                    self.program_counter += 1;
                    self.address_bus.read(self.program_counter)
                })
            });

        if let Some(instruction) = instruction {
            self.execute(instruction);
        } else {
            self.execute_raw(opcode);
        }
    }

    fn read_value_from_operand(&self, operand: &Operand) -> u8 {
        match operand {
            Operand::Register(register) => self.registers.read8(*register),
            Operand::HL => self.address_bus.read(self.address_from_hl())
        }
    }

    fn write_value_to_operand(&mut self, operand: &Operand, value: u8) {
        match operand {
            Operand::Register(register) => self.registers.write8(*register, value),
            Operand::HL => self.address_bus.write(self.address_from_hl(), value)
        }
    }

    fn address_from_hl(&self) -> u16 {
        self.registers.read16(RegisterPair::HL.into())
    }

    fn accumulator(&self) -> u8 {
        self.registers.read8(Register8::A)
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Load(dst, src) => self.execute_load(dst, src),
            Halt => self.halted = true,
            Arithmetic(op, op_type, carry) => self.execute_arithmetic(op, op_type, carry),
            Logical(op, op_type) => self.execute_logical(op, op_type),
            Compare(op) => self.execute_compare(op),
            Bitwise(bitwise_instruction) => self.execute_bitwise(bitwise_instruction),
        }
    }

    fn execute_load(&mut self, dst: Operand, src: Operand) {
        let value = self.read_value_from_operand(&src);
        self.write_value_to_operand(&dst, value);
    }

    fn execute_arithmetic(&mut self, operand: Operand, op_type: ArithmeticOperationType, carry: Carry) {
        let accumulator = self.accumulator();
        let operand_value = self.read_value_from_operand(&operand);

        let carry = if let Carry::TRUE = carry { 1 } else { 0 };

        let add = matches!(op_type, ArithmeticOperationType::ADD);
        let carry_status = CarryStatus::compute(accumulator, operand_value, carry, add);

        let result = if add {
            accumulator.wrapping_add(operand_value).wrapping_add(carry)
        }  else {
            accumulator.wrapping_sub(operand_value).wrapping_sub(carry)
        };

        let flags = Flags::new(
            result == 0,
            !add,
            carry_status.half_carry,
            carry_status.carry
        );

        self.registers.set_flags(flags);
        self.registers.write8(Register8::A, result)
    }

    fn execute_logical(&mut self, operand: Operand, logical_op_type: LogicalInstructionType) {
        let accumulator = self.accumulator();
        let operand_value = self.read_value_from_operand(&operand);

        let result =  match logical_op_type {
            LogicalInstructionType::AND => accumulator & operand_value,
            LogicalInstructionType::XOR => accumulator ^ operand_value,
            LogicalInstructionType::OR => accumulator | operand_value,
        };

        self.registers.write8(Register8::A, result)
    }

    fn execute_compare(&mut self, operand: Operand) {
        if self.accumulator() == self.read_value_from_operand(&operand) {
            self.registers.update_flags(|flags| flags.zero = true)
        }
    }

    fn execute_bitwise(&mut self, bitwise_instruction: BitwiseInstruction) {
        match bitwise_instruction {
            Rotate(op, direction, rotation_type) => self.execute_bitwise_rotate(op, direction, rotation_type),
            Shift(op, direction, shift_type) => self.execute_bitwise_shift(op, direction, shift_type),
            Swap(op) => self.execute_bitwise_swap(op),
            SetZero(op, bit_idx) => self.execute_set_zero(op, bit_idx),
            SetBit(op, bit_idx, set_type) => self.execute_set_bit(op, bit_idx, set_type),
        }
    }

    fn execute_bitwise_rotate(&mut self, operand: Operand, direction: BitwiseDirection, rotation_type: RotationType) {
        let operand_value = self.read_value_from_operand(&operand);

        let current_carry = if self.registers.flags().carry { 1 } else { 0 };
        let mut new_carry = current_carry;

        let circular_rotation = matches!(rotation_type, RotationType::CIRCULAR);

        let result = match direction {
            BitwiseDirection::LEFT => {
                let shifted_operand_value = operand_value << 1;

                if circular_rotation {
                    let hi_bit = (operand_value & 0b1000_0000) >> 7;
                    new_carry = hi_bit;
                    shifted_operand_value | hi_bit
                } else {
                    shifted_operand_value | current_carry
                }
            }
            BitwiseDirection::RIGHT => {
                let shifted_operand_value = operand_value >> 1;

                if circular_rotation {
                    let lo_bit = operand_value & 01;
                    new_carry = lo_bit;
                    shifted_operand_value | (lo_bit << 7)
                } else {
                    shifted_operand_value | (current_carry << 7)
                }
            }
        };

        if current_carry != new_carry {
            self.registers.update_flags(|flags| flags.carry = new_carry != 0);
        }

        self.write_value_to_operand(&operand, result)
    }

    fn execute_bitwise_shift(&mut self, operand: Operand, direction: BitwiseDirection, shift_type: ShiftType) {
        let operand_value = self.read_value_from_operand(&operand);

        let left = matches!(direction, BitwiseDirection::LEFT);
        let logical_shift = matches!(shift_type, ShiftType::LOGICAL);

        // We don't expect to receive an instruction that directs us to both
        // shift left and to perform a logical shift.
        assert!(!left || !logical_shift);

        let unshifted_hi_bit = operand_value & 0b1000_0000;
        let (result, carry) = if left {
            (operand_value << 1, unshifted_hi_bit >> 7)
        } else {
            let carry = operand_value & 0b1;
            let shifted_operand_value = operand_value >> 1;

            let result = if logical_shift {
                shifted_operand_value
            } else {
                // If we're doing an arithmetic shift, then we need to ensure
                // the hi-bit of the operand is the same as it was prior to
                // the shift.
                shifted_operand_value | unshifted_hi_bit
            };

            (result, carry)
        };

        self.registers.update_flags(|flags| flags.carry = carry != 0);
        self.write_value_to_operand(&operand, result);
    }

    fn execute_bitwise_swap(&mut self, operand: Operand) {
        let operand_value = self.read_value_from_operand(&operand);

        let hi = operand_value & 0xF0;
        let lo = operand_value & 0xF;

        self.write_value_to_operand(&operand, (lo << 4) | (hi >> 4))
    }

    fn execute_set_zero(&mut self, operand: Operand, bit_idx: u8) {
        let operand_value = self.read_value_from_operand(&operand);
        let bit_value = (operand_value >> bit_idx) & 0b1;
        self.registers.update_flags(|flags| flags.zero = bit_value != 0);
    }

    fn execute_set_bit(&mut self, operand: Operand, bit_idx: u8, set_type: SetType) {
        let operand_value = self.read_value_from_operand(&operand);

        let result = if matches!(set_type, SetType::SET) {
            operand_value | (1 << bit_idx)
        } else {
            operand_value & !(1 << bit_idx)
        };

        self.write_value_to_operand(&operand, result);
    }

    fn execute_raw(&mut self, opcode: u8) {
        match opcode {
            _ => {}
        }
    }
}

impl GameboyCore for Cpu {
    fn load_rom(&mut self, rom: &[u8]) {
        self.address_bus.load_rom(rom)
    }

    fn step(&mut self) {
        self.step()
    }
}