use std::{
    fmt,
    ops::{Index, IndexMut},
    sync::{atomic::AtomicU8, Arc},
};

use crate::helpers::*;
use color_eyre::eyre::Result;
use derive_more::{Deref, DerefMut};
use sdl2::{
    keyboard::{KeyboardState, Scancode},
    EventPump,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Registers([u8; 0x10]);

impl Index<u8> for Registers {
    type Output = u8;
    fn index(&self, idx: u8) -> &u8 {
        &self.0[idx as usize]
    }
}

impl IndexMut<u8> for Registers {
    fn index_mut(&mut self, idx: u8) -> &mut u8 {
        &mut self.0[idx as usize]
    }
}

#[derive(Debug, Clone)]
pub struct CPU {
    pub v: Registers,
    pub i: u16,
    pub pc: u16,

    pub dt: Arc<AtomicU8>,
    pub st: Arc<AtomicU8>,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            dt: Arc::new(AtomicU8::new(0)),
            st: Arc::new(AtomicU8::new(0)),
            v: Default::default(),
            i: 0,
            pc: 0x200,
        }
    }
}

impl CPU {
    pub const FREQ: u32 = 400;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stack {
    pub memory: [u16; 100],
    pub sp: u8,
}

impl Stack {
    pub fn push(&mut self, val: u16) {
        self.memory[self.sp as usize] = val;
        self.sp += 2;
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 2;
        self.memory[self.sp as usize]
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            memory: [0; 100],
            sp: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct Display([[Color; 32]; 64]);

impl Index<(u8, u8)> for Display {
    type Output = Color;
    fn index(&self, (x, y): (u8, u8)) -> &Self::Output {
        &self.0[x as usize][y as usize]
    }
}

impl IndexMut<(u8, u8)> for Display {
    fn index_mut(&mut self, (x, y): (u8, u8)) -> &mut Self::Output {
        &mut self.0[x as usize][y as usize]
    }
}

impl Default for Display {
    fn default() -> Self {
        Self([[Color::Black; Self::SURH as usize]; Self::SURW as usize])
    }
}

impl Display {
    pub const SURW: u8 = 64;
    pub const SURH: u8 = 32;

    pub fn set_pixel(&mut self, x: u8, y: u8) -> bool {
        self[(x, y)] = !self[(x, y)];
        self[(x, y)] == Color::Black
    }

    pub fn update_screen(&self, win: &mut Window) -> Result<()> {
        todo!("Do update screen")
    }

    pub fn clear(&mut self) {
        todo!("Do clear screen")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DerefMut, Deref)]
pub struct Memory([u8; 0x1000]);

impl Index<u16> for Memory {
    type Output = u8;
    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl Default for Memory {
    fn default() -> Self {
        let mut me = Self([0; 0x1000]);
        for (i, b) in Self::FONT.iter().enumerate() {
            me[0x50 + i as u16] = *b;
        }
        me
    }
}

impl Memory {
    const FONT: &'static [u8] = &[
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

    pub fn digit(digit: u8) -> u16 {
        0x50 + (digit as u16) * 5
    }
}

#[derive(Deref, DerefMut)]
pub struct Keyboard<'a>(KeyboardState<'a>);

impl<'a> Keyboard<'a> {
    pub fn new(events: &'a EventPump) -> Self {
        Self(events.keyboard_state())
    }

    pub fn is_pressed(&self, k: u8) -> bool {
        use Scancode::*;

        #[rustfmt::skip]
        let scancode = match k {
            0x1 => Num1, 0x2 => Num2, 0x3 => Num3, 0xC => Num4,
            0x4 => Q,    0x5 => W,    0x6 => E,    0xD => R,
            0x7 => A,    0x8 => S,    0x9 => D,    0xE => F,
            0xA => Z,    0x0 => X,    0xB => C,    0xF => F,

            _ => unreachable!(),
        };

        self.is_scancode_pressed(scancode)
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct Instruction(pub u16);

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl Instruction {
    /// Actually returns u4. Indexed from most significant. idx <= 4
    pub const fn get_quater(&self, idx: u8) -> u8 {
        let out = self.0 >> (16 - (idx + 1) * 4);
        (out & 0xf) as u8
    }

    pub const fn get_addr(&self) -> u16 {
        self.0 & 0xFFF
    }

    pub const fn get_byte(&self) -> u8 {
        self.0 as u8
    }
}
