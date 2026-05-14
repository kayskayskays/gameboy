use std::ops::{Index, IndexMut};

pub(super) struct AddressBus {
    rom: [u8; 0x8000],
    vram: [u8; 0x2000],
    ram: [u8; 0x2000],
    oam: [u8; 0xA0],
    hram: [u8; 0x0100],
}

impl AddressBus {
    pub(super) fn new() -> AddressBus {
        AddressBus {
            rom: [0; 0x8000],
            vram: [0; 0x2000],
            ram: [0; 0x2000],
            oam: [0; 0xA0],
            hram: [0; 0x0100],
        }
    }

    pub(super) fn read(&self, offset: u16) -> u8 {
        self[offset]
    }

    pub(super) fn write(&mut self, offset: u16, value: u8) {
        self[offset] = value;
    }

    pub(super) fn load_rom(&mut self, rom: &[u8]) {
        self.rom[..rom.len()].copy_from_slice(rom);
    }
}

impl Index<u16> for AddressBus {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        match index {
            0x0000..=0x7FFF => &self.rom[index as usize],
            0x8000..=0x9FFF => &self.vram[(index - 0x8000) as usize],
            0xC000..=0xDFFF => &self.ram[(index - 0xC000) as usize],
            0xFE00..=0xFE9F => &self.oam[(index - 0xFE00) as usize],
            0xFEA0..=0xFEFF => unreachable!(),
            0xFF80..=0xFFFE => &self.hram[(index - 0xFF80) as usize],
            _ => todo!("unmapped: {:#06x}", index),
        }
    }
}

impl IndexMut<u16> for AddressBus {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        match index {
            0x0000..=0x7FFF => &mut self.rom[index as usize],
            0x8000..=0x9FFF => &mut self.vram[(index - 0x8000) as usize],
            0xC000..=0xDFFF => &mut self.ram[(index - 0xC000) as usize],
            0xFE00..=0xFE9F => &mut self.oam[(index - 0xFE00) as usize],
            0xFEA0..=0xFEFF => unreachable!(),
            0xFF80..=0xFFFF => &mut self.hram[(index - 0xFF80) as usize],
            _ => todo!("unmapped: {:#06x}", index),
        }
    }
}