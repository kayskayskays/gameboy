const ZERO_MASK: u8 = 0b1000_0000;
const SUBTRACT_MASK: u8 = 0b0100_0000;
const HALF_CARRY_MASK: u8 = 0b0010_0000;
const CARRY_MASK: u8 = 0b0001_0000;

pub(crate) struct Flags {
    pub(crate) zero: bool,
    pub(crate) subtract: bool,
    pub(crate) half_carry: bool,
    pub(crate) carry: bool,
}

impl Flags {
    pub(crate) fn empty() -> Self {
        Self {
            zero: false,
            subtract: false,
            half_carry: false,
            carry: false,
        }
    }
    
    pub(crate) fn new(zero: bool, subtract: bool, half_carry: bool, carry: bool) -> Self {
        Self { zero, subtract, half_carry, carry, }
    }
}

impl From<Flags> for u8 {
    fn from(flags: Flags) -> u8 {
        let mut result = 0;
        if flags.zero { result |= ZERO_MASK; }
        if flags.subtract { result |= SUBTRACT_MASK; }
        if flags.half_carry { result |= HALF_CARRY_MASK; }
        if flags.carry { result |= CARRY_MASK; }
        result
    }
}

impl From<u8> for Flags {
    fn from(flags: u8) -> Flags {
        Flags {
            zero: (flags & ZERO_MASK) != 0,
            subtract: (flags & SUBTRACT_MASK) != 0,
            half_carry: (flags & HALF_CARRY_MASK) != 0,
            carry: (flags & CARRY_MASK) != 0,
        }
    }
}