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
}

const TIMER_FREQ: u32 = 60;

fn timer_callback(
    audio: AudioDevice,
    dt: Arc<AtomicU8>,
    st: Arc<AtomicU8>,
) -> impl FnMut() -> bool + Sync + Send {
    move || {
        todo!("Add callback for timer");
        true
    }
}

fn main_loop(chip8: &mut Chip8, mut window: Window, mut events: Events) -> Result<()> {
    while !events.is_exited() {
        todo!("Do some things with chip8");
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
