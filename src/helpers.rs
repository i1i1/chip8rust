use color_eyre::{
    eyre::{eyre, WrapErr},
    Result,
};
use derive_more::{Deref, DerefMut};
use sdl2::{
    audio::AudioSpecDesired, event::Event, pixels, rect::Rect, render::Canvas, timer::Timer,
    EventPump, VideoSubsystem,
};

use std::{ops::Not, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl From<Color> for pixels::Color {
    fn from(clr: Color) -> Self {
        match clr {
            Color::White => Self::WHITE,
            Color::Black => Self::BLACK,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::Black
    }
}

#[derive(DerefMut, Deref)]
pub struct Window(Canvas<sdl2::video::Window>);

impl Window {
    const PIXELSZ: u32 = 16;

    pub fn new(video: &VideoSubsystem, w: u32, h: u32) -> Result<Self> {
        let win = video
            .window("CHIP-8", w * Self::PIXELSZ, h * Self::PIXELSZ)
            .position_centered()
            .build()
            .wrap_err("Failed to create a window")?;

        let mut out = win
            .into_canvas()
            .build()
            .map(Self)
            .wrap_err("Failed to build canvas")?;

        out.set_draw_color(pixels::Color::BLACK);
        out.0.clear();
        out.present();
        Ok(out)
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, clr: Color) -> Result<()> {
        self.set_draw_color(pixels::Color::from(clr));
        self.fill_rect(Rect::new(
            x * Self::PIXELSZ as i32,
            y * Self::PIXELSZ as i32,
            Self::PIXELSZ,
            Self::PIXELSZ,
        ))
        .map_err(|e| eyre!("{}", e))
    }
}

#[derive(DerefMut, Deref)]
pub struct Events(EventPump);

impl Events {
    pub fn new(events: EventPump) -> Self {
        Self(events)
    }

    pub fn is_exited(&mut self) -> bool {
        for e in self.poll_iter() {
            if let Event::Quit { .. } = e {
                return true;
            }
        }
        false
    }
}

#[derive(DerefMut, Deref)]
pub struct TimerSubsystem(sdl2::TimerSubsystem);

impl TimerSubsystem {
    pub fn new(timer: sdl2::TimerSubsystem) -> Self {
        Self(timer)
    }

    #[must_use]
    pub fn add_timer<'a, 'b>(
        &'a self,
        every: Duration,
        mut x: impl FnMut() -> bool + Sync + Send + 'b,
    ) -> Timer<'a, 'b> {
        self.0.add_timer(
            every.as_millis() as u32,
            Box::new(move || if x() { every.as_millis() as u32 } else { 0 }),
        )
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AudioCallback {
    passed: f32,
}

impl sdl2::audio::AudioCallback for AudioCallback {
    type Channel = f32;

    fn callback(&mut self, stream: &mut [f32]) {
        const F: f32 = 250.0;
        const PERIOD: f32 = 48000.0 / F;

        for s in stream.iter_mut() {
            self.passed = (self.passed + 1.0) % PERIOD;
            if self.passed >= (PERIOD / 2.0) {
                *s = 0.0;
            } else {
                *s = 50.;
            }
        }
    }
}

pub struct AudioDevice {
    dev: sdl2::audio::AudioDevice<AudioCallback>,
    playing: bool,
}

unsafe impl Send for AudioDevice {}
unsafe impl Sync for AudioDevice {}

impl AudioDevice {
    pub fn new(dev: sdl2::audio::AudioDevice<AudioCallback>) -> Self {
        Self {
            dev,
            playing: false,
        }
    }

    pub fn beep(&self, enable: bool) {
        match (enable, self.playing) {
            (true, false) => self.dev.resume(),
            (false, true) => self.dev.pause(),
            _ => (),
        }
    }
}

pub fn setup(w: u32, h: u32) -> Result<(Window, Events, TimerSubsystem, AudioDevice)> {
    color_eyre::install()?;
    let sdl = sdl2::init().map_err(|e| eyre!("Failed to init sdl2: {}", e))?;
    let video = sdl
        .video()
        .map_err(|e| eyre!("Failed to init video subsystem: {}", e))?;
    let events = sdl
        .event_pump()
        .map_err(|e| eyre!("Failed to init event subsystem: {}", e))?;
    let timer = sdl
        .timer()
        .map_err(|e| eyre!("Failed to init timer subsystem: {}", e))?;
    let audio = sdl
        .audio()
        .map_err(|e| eyre!("Failed to init audio subsystem: {}", e))?;
    let dev = audio
        .open_playback(
            None,
            &AudioSpecDesired {
                freq: Some(48000),
                channels: Some(1),
                samples: Some(4096),
            },
            |_| AudioCallback::default(),
        )
        .map_err(|e| eyre!("Failed to init audio device: {}", e))?;

    Ok((
        Window::new(&video, w, h)?,
        Events::new(events),
        TimerSubsystem::new(timer),
        AudioDevice::new(dev),
    ))
}
