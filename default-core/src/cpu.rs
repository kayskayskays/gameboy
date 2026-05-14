use crate::address_bus::AddressBus;
use crate::instructions::{ArithmeticOperationType, BitwiseDirection, BitwiseInstruction, CarryMode, Instruction::{self, *}, LogicalInstructionType, Operand8, RotationType, SetMode, ShiftType};
use crate::registers::{Register16, Register8, RegisterPair, Registers};
use gameboy_core_interface::GameboyCore;
use BitwiseInstruction::{ModifyBit, Rotate, Shift, Swap, TestBit};

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
    fn compute(x: u8, y: u8, carry: u8, op_type: ArithmeticOperationType) -> Self {
        let mut status = CarryStatus { half_carry: false, carry: false };

        if matches!(op_type, ArithmeticOperationType::Add) {
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

        let opcode = self.next_program_byte();

        let instruction = Instruction::decode_load(opcode)
            .or_else(|| Instruction::decode_arithmetic(opcode))
            .or_else(|| Instruction::decode_bitwise(opcode, || self.next_program_byte()));

        if let Some(instruction) = instruction {
            self.execute(instruction);
        } else {
            self.execute_raw(opcode);
        }
    }

    fn hl_pointer(&self) -> u16 {
        self.registers.read16(RegisterPair::HL.into())
    }

    fn stack_pointer(&self) -> u16 {
        self.registers.read16(Register16::StackPointer)
    }

    fn set_stack_pointer(&mut self, value: u16) {
        self.registers.write16(Register16::StackPointer, value);
    }

    fn stack_pop(&mut self) -> u8 {
        let sp = self.stack_pointer();
        let value = self.address_bus.read(sp);
        self.set_stack_pointer(sp.wrapping_add(1));
        value
    }

    fn stack_push(&mut self, value: u8) {
        let sp = self.stack_pointer().wrapping_sub(1);
        self.set_stack_pointer(sp);
        self.address_bus.write(sp, value);
    }

    fn read_value_from_operand(&self, operand: &Operand8) -> u8 {
        match operand {
            Operand8::Immediate8(immediate ) => *immediate,
            Operand8::Address(address) => self.address_bus.read(*address),
            Operand8::Register(register) => self.registers.read8(*register),
            Operand8::AddressHL => self.address_bus.read(self.hl_pointer())
        }
    }

    fn write_value_to_operand(&mut self, operand: &Operand8, value: u8) {
        match operand {
            Operand8::Immediate8(_) => unreachable!(),
            Operand8::Address(address) => self.address_bus.write(*address, value),
            Operand8::Register(register) => self.registers.write8(*register, value),
            Operand8::AddressHL => self.address_bus.write(self.hl_pointer(), value),
        }
    }

    fn accumulator(&self) -> u8 {
        self.registers.read8(Register8::A)
    }

    fn next_program_byte(&mut self) -> u8 {
        let next = self.address_bus.read(self.program_counter);
        self.program_counter += 1;
        next
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

    fn execute_load(&mut self, dst: Operand8, src: Operand8) {
        let value = self.read_value_from_operand(&src);
        self.write_value_to_operand(&dst, value);
    }

    fn execute_arithmetic(&mut self, operand: Operand8, op_type: ArithmeticOperationType, carry: CarryMode) {
        let accumulator = self.accumulator();
        let operand_value = self.read_value_from_operand(&operand);

        let carry = if let CarryMode::With = carry { 1 } else { 0 };

        let add = matches!(op_type, ArithmeticOperationType::Add);
        let carry_status = CarryStatus::compute(accumulator, operand_value, carry, op_type);

        let result = if add {
            accumulator.wrapping_add(operand_value).wrapping_add(carry)
        }  else {
            accumulator.wrapping_sub(operand_value).wrapping_sub(carry)
        };

        self.registers.update_flags(|flags| {
            flags.zero = result == 0;
            flags.subtract = !add;
            flags.half_carry = carry_status.half_carry;
            flags.carry = carry_status.carry;
        });

        self.registers.write8(Register8::A, result)
    }

    fn execute_logical(&mut self, operand: Operand8, logical_op_type: LogicalInstructionType) {
        let accumulator = self.accumulator();
        let operand_value = self.read_value_from_operand(&operand);

        let result =  match logical_op_type {
            LogicalInstructionType::And => accumulator & operand_value,
            LogicalInstructionType::Xor => accumulator ^ operand_value,
            LogicalInstructionType::Or => accumulator | operand_value,
        };

        self.registers.update_flags(|flags| {
            flags.zero = result == 0;
            // AND always sets the half-carry bit.
            flags.half_carry |= matches!(logical_op_type, LogicalInstructionType::And);
        });

        self.registers.write8(Register8::A, result)
    }

    fn execute_compare(&mut self, operand: Operand8) {
        let accumulator = self.accumulator();
        let operand_value = self.read_value_from_operand(&operand);

        let carry_status = CarryStatus::compute(accumulator, operand_value, 0, ArithmeticOperationType::Sub);
        self.registers.update_flags(|flags| {
            flags.zero = accumulator == operand_value;
            flags.subtract = true;
            flags.half_carry = carry_status.half_carry;
            flags.carry = carry_status.carry;
        });
    }

    fn execute_bitwise(&mut self, bitwise_instruction: BitwiseInstruction) {
        match bitwise_instruction {
            Rotate(op, direction, rotation_type) => self.execute_bitwise_rotate(op, direction, rotation_type),
            Shift(op, direction, shift_type) => self.execute_bitwise_shift(op, direction, shift_type),
            Swap(op) => self.execute_bitwise_swap(op),
            TestBit(op, bit_idx) => self.execute_test_bit(op, bit_idx),
            ModifyBit(op, bit_idx, set_type) => self.execute_set_bit(op, bit_idx, set_type),
        }
    }

    fn execute_bitwise_rotate(&mut self, operand: Operand8, direction: BitwiseDirection, rotation_type: RotationType) {
        let operand_value = self.read_value_from_operand(&operand);

        let current_carry = if self.registers.flags().carry { 1 } else { 0 };
        let mut new_carry = current_carry;

        let circular_rotation = matches!(rotation_type, RotationType::Circular);

        let result = match direction {
            BitwiseDirection::Left => {
                let shifted_operand_value = operand_value << 1;

                shifted_operand_value |
                    if circular_rotation {
                        let hi_bit = (operand_value & 0b1000_0000) >> 7;
                        new_carry = hi_bit;
                        hi_bit
                    } else {
                        current_carry
                    }
            }
            BitwiseDirection::Right => {
                let shifted_operand_value = operand_value >> 1;

                shifted_operand_value |
                    if circular_rotation {
                        let lo_bit = operand_value & 0b1;
                        new_carry = lo_bit;
                        lo_bit << 7
                    } else {
                        current_carry << 7
                    }
            }
        };

        self.registers.update_flags(|flags| {
            flags.zero = result == 0;
            flags.carry = new_carry != 0;
        });

        self.write_value_to_operand(&operand, result)
    }

    fn execute_bitwise_shift(&mut self, operand: Operand8, direction: BitwiseDirection, shift_type: ShiftType) {
        let operand_value = self.read_value_from_operand(&operand);

        let left = matches!(direction, BitwiseDirection::Left);
        let logical_shift = matches!(shift_type, ShiftType::Logical);

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

        self.registers.update_flags(|flags| {
            flags.carry = carry != 0;
            flags.zero = result == 0;
        });

        self.write_value_to_operand(&operand, result);
    }

    fn execute_bitwise_swap(&mut self, operand: Operand8) {
        let operand_value = self.read_value_from_operand(&operand);

        let hi = operand_value & 0xF0;
        let lo = operand_value & 0xF;

        let result = (lo << 4) | (hi >> 4);
        self.write_value_to_operand(&operand, result);
        self.registers.update_flags(|flags| flags.zero = result == 0);
    }

    fn execute_test_bit(&mut self, operand: Operand8, bit_idx: u8) {
        let operand_value = self.read_value_from_operand(&operand);
        let bit_value = (operand_value >> bit_idx) & 0b1;
        self.registers.update_flags(|flags| flags.zero = bit_value != 0);
    }

    fn execute_set_bit(&mut self, operand: Operand8, bit_idx: u8, set_type: SetMode) {
        let operand_value = self.read_value_from_operand(&operand);

        let result = if matches!(set_type, SetMode::Set) {
            operand_value | (1 << bit_idx)
        } else {
            operand_value & !(1 << bit_idx)
        };

        self.write_value_to_operand(&operand, result);
    }

    fn execute_raw(&mut self, opcode: u8) {
        match opcode {
            opcode @ (0xC1 | 0xD1 | 0xE1 | 0xF1 | 0xC5 | 0xD5 | 0xE5 | 0xF5) => {
                // POP / PUSH
                let register_pair = match opcode & 0xF {
                    0xC => RegisterPair::BC,
                    0xD => RegisterPair::DE,
                    0xE => RegisterPair::HL,
                    0xF => RegisterPair::AF,
                    _ => unreachable!(),
                };

                if opcode & 0xF == 1 {
                    let lo = self.stack_pop();
                    let hi = self.stack_pop();
                    self.registers.write16(register_pair.into(), (hi as u16) << 8 | (lo as u16))
                } else {
                    let value = self.registers.read16(register_pair.into());
                    self.stack_push((value >> 8) as u8);
                    self.stack_push(value as u8);
                }
            }
            opcode @ (0xC7 | 0xD7 | 0xE7 | 0xF7 | 0xCF | 0xDF | 0xEF | 0xFF) => {
                // RST
                self.stack_push((self.program_counter >> 8) as u8);
                self.stack_push(self.program_counter as u8);
                self.program_counter = (opcode - 0xC7) as u16;
            }
            opcode @ (0xC0 | 0xD0 | 0xC8 | 0xD8) => {
                let flags = self.registers.flags();

                let flag_set = if opcode & 0xF == 0xC {
                    flags.zero
                } else {
                    flags.carry
                };

                let require_unset_flag = opcode & 0xF == 0x8;

                if require_unset_flag ^ flag_set {
                    let lo = self.stack_pop();
                    let hi = self.stack_pop();
                    self.set_stack_pointer((hi as u16) << 8 | (lo as u16));
                }
            }
            _ => {
                let immediate = self.next_program_byte();
                self.execute_raw_with_immediate8(opcode, immediate);
            }
        }
    }

    fn execute_raw_with_immediate8(&mut self, opcode: u8, immediate: u8) {
        let immediate_operand = Operand8::Immediate8(immediate);

        match opcode {
            0xE8 => {
                let sp = self.stack_pointer();
                let carry_status = CarryStatus::compute(sp as u8, immediate, 0, ArithmeticOperationType::Add);

                let sp = self.stack_pointer().wrapping_add(immediate as i8 as u16);
                self.set_stack_pointer(sp);

                self.registers.update_flags(|flags| {
                    flags.zero = false;
                    flags.subtract = false;
                    flags.half_carry = carry_status.half_carry;
                    flags.carry = carry_status.carry;
                })
            },
            0xC6 => self.execute_arithmetic(immediate_operand, ArithmeticOperationType::Add, CarryMode::Without),
            0xD6 => self.execute_arithmetic(immediate_operand, ArithmeticOperationType::Sub, CarryMode::Without),
            0xCE => self.execute_arithmetic(immediate_operand, ArithmeticOperationType::Add, CarryMode::With),
            0xDE => self.execute_arithmetic(immediate_operand, ArithmeticOperationType::Sub, CarryMode::With),
            0xE6 => self.execute_logical(immediate_operand, LogicalInstructionType::And),
            0xF6 => self.execute_logical(immediate_operand, LogicalInstructionType::Or),
            0xEE => self.execute_logical(immediate_operand, LogicalInstructionType::Xor),
            0xFE => self.execute_compare(immediate_operand),
            opcode @ (0xE0 | 0xF0 | 0xE2 | 0xF2) => {
                let address = if opcode & 0xF == 0x2 {
                    self.read_value_from_operand(&Operand8::Register(Register8::C))
                } else {
                    immediate
                } as u16 + 0xFF00;

                let address_operand = Operand8::Address(address);
                let register_operand = Operand8::Register(Register8::A);

                if opcode == 0xE0 {
                    self.execute_load(address_operand, register_operand);
                } else {
                    self.execute_load(register_operand, address_operand);
                }
            }
            _ => {
                let next_immediate = self.next_program_byte();
                self.execute_raw_with_immediate16(opcode, (immediate as u16) << 8 | (next_immediate as u16));
            },
        }
    }

    fn execute_raw_with_immediate16(&mut self, opcode: u8, immediate: u16) {
        match opcode {
            opcode @ (0xEA | 0xFA) => {
                let address_operand = Operand8::Address(immediate);
                let register_operand = Operand8::Register(Register8::A);

                if opcode == 0xEA {
                    self.execute_load(address_operand, register_operand);
                } else {
                    self.execute_load(register_operand, address_operand);
                }
            }
            _ => todo!()
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
