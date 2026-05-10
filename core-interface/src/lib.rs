
pub trait GameboyCore {
    fn load_rom(&mut self, rom: &[u8]);
    fn step(&mut self);
}