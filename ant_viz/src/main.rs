extern crate ant_lib;
extern crate piston_window;
extern crate opengl_graphics;

mod camera;
mod mouse;
mod view;

use ant_lib::{test_data, Instruction, Outcome, Simulator, TurnDir};
use piston_window::{Button, EventLoop, Input, Key, Motion, MouseButton, OpenGL, PistonWindow, WindowSettings};
use opengl_graphics::GlGraphics;

use camera::Camera;
use mouse::Mouse;
use view::View;

const SCR_WIDTH: u32 = 1024;
const SCR_HEIGHT: u32 = 600;
const UPS: u32 = 5;

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("AntViz", [SCR_WIDTH, SCR_HEIGHT])
        .opengl(opengl).samples(8).exit_on_esc(true).build().unwrap();

    // One round per update, 5 rounds per second
    window.set_ups(UPS);
    window.set_max_fps(30);

    let mut gl = GlGraphics::new(opengl);

    // FIXME: allow the user to specify the world and the programs
    let go_right = vec![Instruction::Move(0, 1), Instruction::Mark(0, 0)];
    let mut simulator = Simulator::new(test_data::sample0(), go_right, test_data::default_program());
    let mut partial_outcome = Outcome::default();
    let mut view = View::new(Camera::new(SCR_WIDTH as f64, SCR_HEIGHT as f64, &simulator.world));
    let mut mouse = Mouse::default();

    // Per update, run one round of the simulation. This results in 5 iterations per second.
    let mut rounds_per_update: u32 = 1;

    while let Some(e) = window.next() {
        // Event handling
        match e {
            Input::Press(Button::Keyboard(key)) => {
                match key {
                    Key::Right => view.cam.move_x(100.0),
                    Key::Left => view.cam.move_x(-100.0),
                    Key::Up => view.cam.move_y(-100.0),
                    Key::Down => view.cam.move_y(100.0),
                    _ => {}
                }
            }

            Input::Text(s) => {
                match s.as_str() {
                    "+" => {
                        rounds_per_update += 1;
                        println!("[UPDATE] rounds per update: {}", rounds_per_update);
                    }
                    "-" => {
                        if rounds_per_update > 0 {
                            rounds_per_update -= 1;
                        }
                        println!("[UPDATE] rounds per second: {}", rounds_per_update * UPS);
                    }
                    "m" => {
                        view.toggle_marks();
                        println!("[UPDATE] toggle marks: {:?}", view.show_marks);
                    }
                    "t" => {
                        view.show_score = !view.show_score;
                    }
                    _   => ()
                }

            }

            Input::Resize(width, height) => {
                view.cam.resize(width, height);
            }

            Input::Render(args) => {
                gl.draw(args.viewport(), |c, g| view.render(&simulator.world, &partial_outcome, c, g));
            }

            Input::Update(_) => {
                // 5 rounds per second
                partial_outcome = simulator.run_rounds(rounds_per_update);
            }

            _ => {}
        }
    }
}
