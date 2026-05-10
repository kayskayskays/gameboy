
pub(super) struct AddressBus {
    rom:  [u8; 0x8000],
    vram: [u8; 0x2000],
    ram:  [u8; 0x2000],
    oam:  [u8; 0xA0],
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
        match offset {
            0x0000..=0x7FFF => self.rom[offset as usize],
            0x8000..=0x9FFF => self.vram[(offset - 0x8000) as usize],
            0xC000..=0xDFFF => self.ram[(offset - 0xC000) as usize],
            0xFE00..=0xFE9F => self.oam[(offset - 0xFE00) as usize],
            0xFEA0..=0xFEFF => unreachable!(),
            0xFF00..=0xFFFF => self.hram[(offset - 0xFF00) as usize],
            _ => todo!("unmapped: {:#06x}", offset),
        }
    }
    
    pub(super) fn load_rom(&mut self, rom: &[u8]) {
        self.rom[..rom.len()].copy_from_slice(rom);
    }
}