//!
//! `yakui-macroquad` integrates yakui with macroquad.
//!
//! # Usage
//! To use this library, you call [`start`] when you wish to begin
//! submitting ui draw commands and [`finish`] when you are done.
//!
//! Though, there's also the [`ui`] helper that takes a closure and will call [`start`] before your code and [`finish`] after.
//! But using [`start`] and [`finish`] is closer to how yakui itself does it, so that's probably what you should do.
//!
//! To then render your ui, simply call [`draw`]!
//!
//! ```no_run
//! use macroquad::prelude::*;
//! use yakui_macroquad::*;
//! use yakui::*;
//!
//! #[macroquad::main("yakui-macroquad-example")]
//! async fn main() {
//!    
//!     loop {
//!
//!         clear_background(WHITE);
//!
//!         yakui_macroquad::start();
//!
//!         yakui::center(|| {
//!             let mut text_box = yakui::widgets::Text::new(32.0, "hello, world!");
//!             text_box.style.color = yakui::Color::BLACK;
//!             text_box.show();
//!         });
//!
//!         yakui_macroquad::finish();
//!
//!         yakui_macroquad::draw();
//!
//!         next_frame().await;
//!
//!     }
//!    
//! }
//!```

use send_wrapper::SendWrapper;
use std::sync::{RwLock, RwLockWriteGuard};

use macroquad::miniquad as mq;
use macroquad::window::get_internal_gl;
use yakui_miniquad::*;

pub use macroquad;

struct Yakui(YakuiMiniQuad, usize);

// Global variable and global functions because it's more like macroquad way
static YAKUI: RwLock<Option<SendWrapper<Yakui>>> = RwLock::new(None);

fn get_yakui() -> RwLockWriteGuard<'static, Option<SendWrapper<Yakui>>> {
    match YAKUI.try_write() {
        Ok(mut yakui) => {
            if yakui.is_some() {
                yakui
            } else {
                *yakui = Some(SendWrapper::new(Yakui::new()));
                yakui
            }
        }
        Err(_) => panic!(
            "tried to borrow yakui mutably twice, did you accidentally nest ui or cfg calls?"
        ),
    }
}

impl Yakui {
    fn new() -> Self {
        Self(
            YakuiMiniQuad::new(unsafe { get_internal_gl() }.quad_context),
            macroquad::input::utils::register_input_subscriber(),
        )
    }

    fn start(&mut self) {
        macroquad::input::utils::repeat_all_miniquad_input(self, self.1);
        self.0.start();
    }

    fn finish(&mut self) {
        self.0.finish();
    }

    fn ui<F>(&mut self, f: F)
    where
        F: FnOnce(&mut yakui_core::Yakui),
    {
        macroquad::input::utils::repeat_all_miniquad_input(self, self.1);

        self.0.run(f);
    }

    fn draw(&mut self) {
        let mut gl = unsafe { get_internal_gl() };
        // Ensure that macroquad's shapes are not going to be lost, and draw them now
        gl.flush();
        self.0.draw(gl.quad_context);
    }
}

/// Returns true if the last mouse or keyboard event was sunk by yakui, and should not be handled by your game.
pub fn has_input_focus() -> bool {
    get_yakui().as_ref().unwrap().0.has_input_focus()
}

/// Returns true if the last keyboard event was sunk by yakui, and should not be handled by your game.
pub fn has_keyboard_focus() -> bool {
    get_yakui().as_ref().unwrap().0.has_keyboard_focus()
}

/// Returns true if the last mouse event was sunk by yakui, and should not be handled by your game.
pub fn has_mouse_focus() -> bool {
    get_yakui().as_ref().unwrap().0.has_mouse_focus()
}

/// Binds the yakui context to the current thread.
pub fn start() {
    get_yakui().as_mut().unwrap().start();
}

/// Finishes the current yakui context and prepares it for rendering.
pub fn finish() {
    get_yakui().as_mut().unwrap().finish();
}

/// Allows you to submit commands to the yakui context inside the scope of the closure passed, calls [`start`] and [`finish`] for you.
pub fn ui<F: FnOnce(&mut yakui_core::Yakui)>(f: F) {
    get_yakui().as_mut().unwrap().ui(|ctx| f(ctx))
}

/// Allows you configure the yakui context within the scope of the closure passed, if you need to.
pub fn cfg<F: FnOnce(&mut yakui_core::Yakui)>(f: F) {
    f(get_yakui().as_mut().unwrap().0.ctx());
}

/// Draws the yakui ui. Must be called after `finish`/`ui` and once per frame.
pub fn draw() {
    get_yakui().as_mut().unwrap().draw()
}

impl mq::EventHandler for Yakui {
    fn update(&mut self) {}

    fn draw(&mut self) {}

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.0.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, dx: f32, dy: f32) {
        self.0.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.0.mouse_button_down_event(mb, x, y);
    }

    fn mouse_button_up_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.0.mouse_button_up_event(mb, x, y);
    }

    fn char_event(&mut self, character: char, keymods: mq::KeyMods, repeat: bool) {
        self.0.char_event(character, keymods, repeat);
    }

    fn key_down_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods, repeat: bool) {
        self.0.key_down_event(keycode, keymods, repeat);
    }

    fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.0.key_up_event(keycode, keymods);
    }
}
