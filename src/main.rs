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

    fn math(&mut self, inst: Instruction) {
        match inst.get_quater(3) {
			0x0 => todo!("8XY0 	Assig 	Vx = Vy 	Sets VX to the value of VY."),
			0x1 => todo!("8XY1 	BitOp 	Vx = Vx | Vy 	Sets VX to VX or VY. (Bitwise OR operation);"),
			0x2 => todo!("8XY2 	BitOp 	Vx = Vx & Vy 	Sets VX to VX and VY. (Bitwise AND operation);"),
			0x3 => todo!("8XY3[a] 	BitOp 	Vx = Vx ^ Vy 	Sets VX to VX xor VY."),
			0x4 => todo!("8XY4 	Math 	Vx += Vy 	Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not."),
			0x5 => todo!("8XY5 	Math 	Vx -= Vy 	VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not."),
			0x6 => todo!("8XY6[a] 	BitOp 	Vx >>= 1 	Stores the least significant bit of VX in VF and then shifts VX to the right by 1.[b]"),
			0x7 => todo!("8XY7[a] 	Math 	Vx = Vy - Vx 	Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not."),
			0xE => todo!("8XYE[a] 	BitOp 	Vx <<= 1 	Stores the most significant bit of VX in VF and then shifts VX to the left by 1.[b]"),

			_ => unreachable!()
		}
    }

    // DXYN 	Disp 	draw(Vx, Vy, N)
    // Draws a sprite at coordinate (VX, VY)
    // that has a width of 8 pixels and a height of N+1 pixels.
    // Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction.
    // As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen
    pub fn draw(&mut self, x: u8, y: u8, n: u8) {
        let i = self.cpu.i;
        self.cpu.v[0xf] = 0x0;
        for j in 0..n {
            for k in 0..8 {
                if (self.memory[i + j as u16] >> k) % 2 == 1 {
                    if self.display.set_pixel(x + j, y + k) {
                        self.cpu.v[0xf] = 0x1;
                    }
                }
            }
        }
    }

    pub fn iter(&mut self) {
        let inst = self.read_inst(self.cpu.pc);
        self.cpu.pc += 2;
        match dbg!(inst).get_quater(0) {
			0x0 if *inst == 0x00E0 => self.display.clear(),
			//0NNN 	Call 		Calls machine code routine (RCA 1802 for COSMAC VIP) at address NNN. Not necessary for most ROMs.
			//00E0 	Display 	disp_clear() 	Clears the screen.
			//00EE 	Flow 	return; 	Returns from a subroutine.
			1 => todo!("1NNN 	Flow 	goto NNN; 	Jumps to address NNN.                                                                                              )  //NNN 	Flow 	goto NNN; 	Jumps to address NNN."),
			2 => todo!("2NNN 	Flow 	*(0xNNN)() 	Calls subroutine at NNN.                                                                                          )  //NNN 	Flow 	*(0xNNN)() 	Calls subroutine at NNN."),
			3 => todo!("3XNN 	Cond 	if (Vx == NN) 	Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block);     )  //XNN 	Cond 	if (Vx == NN) 	Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block);"),
			4 => todo!("4XNN 	Cond 	if (Vx != NN) 	Skips the next instruction if VX does not equal NN. (Usually the next instruction is a jump to skip a code block);)  //XNN 	Cond 	if (Vx != NN) 	Skips the next instruction if VX does not equal NN. (Usually the next instruction is a jump to skip a code block);"),
			5 => todo!("5XY0 	Cond 	if (Vx == Vy) 	Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block);     )  //XY0 	Cond 	if (Vx == Vy) 	Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block);"),
			// 6XNN 	Const 	Vx = N 	Sets VX to NN.                                                                                                       )  //XNN 	Const 	Vx = N 	Sets VX to NN.
			6 => {
				let x = inst.get_quater(1);
				let nn = inst.get_byte();
				self.cpu.v[x] = nn;
			}
			7 => todo!("7XNN 	Const 	Vx += N 	Adds NN to VX. (Carry flag is not changed);                                                                         )  //XNN 	Const 	Vx += N 	Adds NN to VX. (Carry flag is not changed);"),
			8 => self.math(inst),
			0x9 => todo!("9XY0 	Cond 	if (Vx != Vy) 	Skips the next instruction if VX does not equal VY. (Usually the next instruction is a jump to skip a code block);"),
			0xA => self.cpu.i = inst.get_addr(),
			0xB => todo!("BNNN 	Flow 	PC = V0 + NNN 	Jumps to the address NNN plus V0."),
			0xC => todo!("CXNN 	Rand 	Vx = rand() & NN 	Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN."),
			// DXYN 	Disp 	draw(Vx, Vy, N)
			// Draws a sprite at coordinate (VX, VY)
			// that has a width of 8 pixels and a height of N+1 pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction. As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen
			0xD => {
				let x = inst.get_quater(1);
				let y = inst.get_quater(2);
				let n = inst.get_quater(3);
				self.draw(self.cpu.v[x], self.cpu.v[y], n + 1);
			}

			_ => panic!("{:?}", inst),
			//EX9E 	KeyOp 	if (key() == Vx) 	Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block);
			//EXA1 	KeyOp 	if (key() != Vx) 	Skips the next instruction if the key stored in VX is not pressed. (Usually the next instruction is a jump to skip a code block);
			//FX07 	Timer 	Vx = get_delay() 	Sets VX to the value of the delay timer.
			//FX0A 	KeyOp 	Vx = get_key() 	A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event);
			//FX15 	Timer 	delay_timer(Vx) 	Sets the delay timer to VX.
			//FX18 	Sound 	sound_timer(Vx) 	Sets the sound timer to VX.
			//FX1E 	MEM 	I += Vx 	Adds VX to I. VF is not affected.[c]
			//FX29 	MEM 	I = sprite_addr[Vx] 	Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
			//FX33 	BCD set_BCD(Vx) *(I+0) = BCD(3); *(I+1) = BCD(2); *(I+2) = BCD(1); Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2. (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.);
			//FX55 	MEM 	reg_dump(Vx, &I) 	Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.[d]
			//FX65 	MEM 	reg_load(Vx, &I) 	Fills V0 to VX (including VX) with values from memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.[d]
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
        //todo!("Add callback for timer");
        true
    }
}

fn main_loop(chip8: &mut Chip8, mut window: Window, mut events: Events) -> Result<()> {
    while !events.is_exited() {
        chip8.iter();
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
