extern crate ant_lib;
extern crate piston_window;
extern crate opengl_graphics;

mod camera;
mod view;

use ant_lib::{test_data, Simulator};
use piston_window::{Button, EventLoop, Input, Motion, OpenGL, PistonWindow, WindowSettings};
use opengl_graphics::GlGraphics;

use camera::Camera;

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("AntViz", [1024, 600])
        .opengl(opengl).samples(8).exit_on_esc(true).build().unwrap();

    // One round per update, 5 rounds per second
    window.set_ups(5);
    window.set_max_fps(30);

    let mut gl = GlGraphics::new(opengl);

    // FIXME: allow the user to specify the world and the programs
    let mut simulator = Simulator::new(test_data::sample0(), test_data::default_program(), test_data::default_program());
    let mut cam = Camera::default();

    while let Some(e) = window.next() {
        // Event handling
        match e {
            Input::Press(Button::Keyboard(key)) => {
                //game.key_press(key);
            }

            Input::Release(Button::Keyboard(key)) => {
                //game.key_release(key);
            }

            Input::Render(args) => {
                gl.draw(args.viewport(), |c, g| view::render(&simulator.world, cam, c, g));
            }

            Input::Update(args) => {
                // 5 rounds per second
                simulator.one_round();
            }

            _ => {}
        }
    }
}
