use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU8, Ordering::SeqCst},
        Arc,
    },
    time::{Duration, Instant},
};

use color_eyre::eyre::{eyre, Result, WrapErr};
use helpers::*;
use structopt::StructOpt;
use types::*;

mod helpers;
mod types;

/// Simple Chip 8 emulator
#[derive(Debug, StructOpt)]
struct Args {
    /// Path to game
    #[structopt(short, long)]
    game: PathBuf,
}

#[derive(Debug, Clone, Default)]
struct Chip8 {
    pub cpu: CPU,
    pub stack: Stack,
    pub display: Display,
    pub memory: Memory,
}

impl Chip8 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_game(game: Vec<u8>) -> Result<Self> {
        let mut me = Self::new();
        if game.len() > 0x800 {
            return Err(eyre!(
                "Length of file should be not more than 2kb. Got: {}",
                game.len()
            ));
        }

        for (i, b) in game.into_iter().enumerate() {
            me.memory[i as u16 + me.cpu.pc] = b;
        }

        Ok(me)
    }

    fn read_inst(&self, addr: u16) -> Instruction {
        Instruction(u16::from_be_bytes([
            self.memory[addr],
            self.memory[addr + 1],
        ]))
    }

    fn math(&mut self, math: Instruction) {
        let x = math.get_quater(1) as usize;
        let y = math.get_quater(2) as usize;
        match math.get_quater(3) {
            // 8XY0 Store the value of register VY in register VX
            0x0 => self.cpu.v[x] = self.cpu.v[y],
            // 8XY1 Set VX to VX OR VY
            0x1 => self.cpu.v[x] |= self.cpu.v[y],
            // 8XY2 Set VX to VX AND VY
            0x2 => self.cpu.v[x] &= self.cpu.v[y],
            // 8XY3 Set VX to VX XOR VY
            0x3 => self.cpu.v[x] ^= self.cpu.v[y],
            // 8XY4 Add the value of register VY to register VX
            // Set VF to 01 if a carry occurs
            // Set VF to 00 if a carry does not occur"),
            0x4 => {
                let carry = self.cpu.v[x] as isize + self.cpu.v[y] as isize > 0xFF;
                self.cpu.v[x] = self.cpu.v[x].wrapping_add(self.cpu.v[y]);
                self.cpu.v[0xf] = if carry { 0x01 } else { 0x00 };
            }
            // 8XY5 Subtract the value of register VY from register VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            0x5 => {
                let borrow = (self.cpu.v[x] as isize) < self.cpu.v[y] as isize;
                self.cpu.v[x] = self.cpu.v[x].wrapping_sub(self.cpu.v[y]);
                self.cpu.v[0xf] = if borrow { 0x00 } else { 0x01 };
            }
            // 8XY6 Store the value of register VY shifted right one bit in register VX¹
            // Set register VF to the least significant bit prior to the shift
            // VY is unchanged
            0x6 => {
                self.cpu.v[0xF] = self.cpu.v[y] % 2;
                self.cpu.v[y] >>= 1;
                self.cpu.v[x] = self.cpu.v[y];
            }
            // 8XY7 Set register VX to the value of VY minus VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            0x7 => todo!("8XY7"),
            // 8XYE Store the value of register VY shifted left one bit in register VX¹
            // Set register VF to the most significant bit prior to the shift
            // VY is unchanged
            0xE => todo!("8XYE"),

            _ => todo!("Unknown math instruction"),
        }
    }

    fn io(&mut self, io: Instruction) {
        let half = io.get_byte();
        let x = io.get_quater(1);
        match half {
           // FX07     Store the current value of the delay timer in register VX
           0x07 => self.cpu.v[x as usize] = self.cpu.dt.load(SeqCst),
           0x0A => todo!("FX0A     Wait for a keypress and store the result in register VX"),
           // todo!("FX15     Set the delay timer to the value of register VX"),
           0x15 => self.cpu.dt.store(self.cpu.v[x as usize], SeqCst),
           0x18 => self.cpu.st.store(self.cpu.v[x as usize], SeqCst),
           0x1E => todo!("FX1E     Add the value stored in register VX to register I"),
           // FX29     Set I to the memory address of the sprite data corresponding
           // to the hexadecimal digit stored in register VX
           0x29 => self.cpu.i = Memory::digit(self.cpu.v[x as usize]),

           // FX33     Store the binary-coded decimal equivalent of the value stored
           // in register VX at addresses I, I + 1, and I + 2
           0x33 => {
               self.memory[self.cpu.i + 2] = self.cpu.v[x as usize] % 10;
               self.memory[self.cpu.i + 1] = self.cpu.v[x as usize] / 10 % 10;
               self.memory[self.cpu.i] = self.cpu.v[x as usize] / 100;
           }
           0x55 => todo!("FX55     Store the values of registers V0 to VX inclusive in memory starting at address I I is set to I + X + 1 after operation²"),
           // FX65     Fill registers V0 to VX inclusive with the values stored in
           // memory starting at address I I is set to I + X + 1 after operation²
           0x65 => {
               for i in 0..=x {
                   self.cpu.v[i as usize] = self.memory[self.cpu.i + i as u16];
               }
           }

           _ => todo!("Unknown io instruction"),
        }
    }

    /// DXYN     Draw a sprite at position VX, VY with N bytes of sprite data
    /// starting at the address stored in I
    ///
    /// Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
    fn draw_sprite(&mut self, x: u8, y: u8, n: u8) {
        self.cpu.v[0xf] = 0;

        for i in 0..n {
            for j in 0..8 {
                if self.memory[self.cpu.i + i as u16] & (1 << j) != 0 {
                    if self.display.set_pixel(x + 7 - j, y + i) {
                        self.cpu.v[0xf] = 1;
                    }
                }
            }
        }
    }

    pub fn step<'a>(&mut self, kbd: &Keyboard<'a>) {
        let inst = self.read_inst(self.cpu.pc);
        self.cpu.pc += 2;
        match inst.get_quater(0) {
           0x0 if *inst == 0x00E0 => self.display.clear(),
           // Ret
           0x0 if *inst == 0x00EE => {
               self.stack.sp -= 2;
               self.cpu.pc = self.stack.memory[self.stack.sp as usize];
           },
           0x0 => todo!("Execute machine language subroutine at address NNN"),
           0x1 => self.cpu.pc = inst.get_addr(),
           0x2 => {
               self.stack.memory[self.stack.sp as usize] = self.cpu.pc;
               self.stack.sp += 2;
               self.cpu.pc = inst.get_addr();
           },
           // Skip the following instruction if the value of register VX equals NN 
           0x3 => {
               let x = inst.get_quater(1);
               if self.cpu.v[x as usize] == inst.get_byte() {
                   self.cpu.pc += 2;
               }
           }
           // 4XNN Skip the following instruction if the value of register VX is not equal to NN
           0x4 => {
               let nn = inst.get_byte();
               let x = inst.get_quater(1);
               if self.cpu.v[x as usize] != nn {
                   self.cpu.pc += 2;
               }
           },
           // 5XY0 Skip the following instruction if the value of register VX is equal to the value of register VY
           0x5 => {
               let x = inst.get_quater(1);
               let y = inst.get_quater(2);
               if self.cpu.v[x as usize] == self.cpu.v[y as usize] {
                   self.cpu.pc += 2;
               }
           },
           // 6XNN Store number NN in register VX
           0x6 => self.cpu.v[inst.get_quater(1) as usize] = inst.get_byte(),
           // 7XNN Add the value NN to register VX
           0x7 => {
			   let x = inst.get_quater(1) as usize;
			   let nn = inst.get_byte();
			   self.cpu.v[x] = self.cpu.v[x].wrapping_add(nn);
		   }
           0x8 => self.math(inst),
           0x9 if inst.get_quater(3) == 0x0 => todo!("9XY0     Skip the following instruction if the value of register VX is not equal to the value of register VY"),
           0xA => self.cpu.i = inst.get_addr(), // todo!("ANNN     Store memory address NNN in register I"),
           0xB => todo!("BNNN     Jump to address NNN + V0"),
           // CXNN     Set VX to a random number with a mask of NN
           0xC => {
               let x = inst.get_quater(1) as usize;
               self.cpu.v[x] = rand::random();
               self.cpu.v[x] &= inst.get_byte();
           },
           // DXYN     Draw a sprite at position VX, VY with N bytes of sprite data
           // starting at the address stored in I
           //
           // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
           0xD => self.draw_sprite(self.cpu.v[inst.get_quater(1) as usize], self.cpu.v[inst.get_quater(2) as usize], inst.get_quater(3)),
           0xE if inst.get_byte() == 0x9E => todo!("EX9E     Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed"),
           // EXA1     Skip the following instruction if the key corresponding to the hex value
           // currently stored in register VX is not pressed
           0xE if inst.get_byte() == 0xA1 => {
               if !kbd.is_pressed(self.cpu.v[inst.get_quater(1) as usize]) {
                   self.cpu.pc += 2;
               }
           },
           0xF => self.io(inst),

            _ => todo!("Unknown instruction"),
        }
    }
}

const TIMER_FREQ: u32 = 60;

fn timer_callback(
    audio: AudioDevice,
    dt: Arc<AtomicU8>,
    st: Arc<AtomicU8>,
) -> impl FnMut() -> bool + Sync + Send {
    move || {
        if dt.load(SeqCst) > 0 {
            dt.fetch_sub(1, SeqCst);
        }
        if st.load(SeqCst) > 0 {
            st.fetch_sub(1, SeqCst);
        }
        audio.beep(st.load(SeqCst) > 0);
        true
    }
}

fn main_loop(chip8: &mut Chip8, mut window: Window, mut events: Events) -> Result<()> {
    while !events.is_exited() {
        let batch = 16;

        let start = Instant::now();

        let kbd = Keyboard::new(&events);
        for _ in 0..batch {
            chip8.step(&kbd);
        }

        chip8
            .display
            .update_screen(&mut window)
            .wrap_err("Failed to update screen")?;
        window.present();

        std::thread::sleep(batch * (Duration::from_secs(1) / CPU::FREQ) - start.elapsed());
    }
    Ok(())
}

fn main() -> Result<()> {
    let (window, events, timer, audio) = setup(Display::SURW as u32, Display::SURH as u32)?;

    let Args { game } = Args::from_args();
    let game = std::fs::read(&game)
        .wrap_err_with(|| eyre!("Failed to read file with game: {:?}", game))?;
    let mut chip8 = Chip8::from_game(game).wrap_err("Failed to init chip8")?;

    let _timer = timer.add_timer(
        Duration::from_secs(1) / TIMER_FREQ,
        timer_callback(audio, Arc::clone(&chip8.cpu.dt), Arc::clone(&chip8.cpu.st)),
    );

    main_loop(&mut chip8, window, events)
}
