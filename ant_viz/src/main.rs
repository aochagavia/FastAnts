extern crate ant_lib;
extern crate opengl_graphics;
extern crate piston_window;
extern crate structopt;
#[macro_use] extern crate structopt_derive;

mod camera;
mod cli;
mod view;

use std::fs::File;
use std::io::BufReader;
use std::process;

use ant_lib::{test_data, Instruction, Outcome, Simulator, World};
use opengl_graphics::GlGraphics;
use piston_window::{Button, EventLoop, Input, Key, OpenGL, PistonWindow, WindowSettings};
use structopt::StructOpt;

use camera::Camera;
use cli::Options;
use view::View;

const SCR_WIDTH: u32 = 1024;
const SCR_HEIGHT: u32 = 600;
const UPS: u32 = 5;

fn init_window() -> (PistonWindow, GlGraphics) {
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("AntViz", [SCR_WIDTH, SCR_HEIGHT])
        .opengl(opengl).samples(8).exit_on_esc(true).build().unwrap();

    // One round per update, 5 rounds per second
    window.set_ups(UPS as u64);
    window.set_max_fps(30);

    (window, GlGraphics::new(opengl))
}

fn fatal_error(msg: &str) -> ! {
    println!("Fatal error: {}", msg);
    process::exit(1);
}

fn open_file_or_die(path: &str) -> BufReader<File> {
    File::open(&path).map(|file| BufReader::new(file))
                     .unwrap_or_else(|_| fatal_error(&format!("unable to open file: {}", path)))
}

fn load_ant_instructions(path: &str) -> Vec<Instruction> {
    Instruction::parse(open_file_or_die(path))
}

fn load_world(path: &str) -> World {
    World::parse(open_file_or_die(path))
}

fn main() {
    let options = Options::from_args();

    let red = options.red.map(|p| load_ant_instructions(&p)).unwrap_or_else(|| {
        println!("No file specified for red ant instructions. Using defaults.");
        test_data::ant1()
    });

    let black = options.black.map(|p| load_ant_instructions(&p)).unwrap_or_else(|| {
        println!("No file specified for black ant instructions. Using defaults.");
        test_data::ant1()
    });

    let world = options.world.map(|p| load_world(&p)).unwrap_or_else(|| {
        println!("No world file specified. Using default world.");
        test_data::sample0()
    });

    let mut simulator = Simulator::new(world, red, black, options.rounds);
    let mut partial_outcome = Outcome::default();
    let mut view = View::new(Camera::new(SCR_WIDTH as f64, SCR_HEIGHT as f64, &simulator.world));

    // Per update, run one round of the simulation. This results in 5 iterations per second.
    let mut rounds_per_update: u32 = 1;

    let (mut window, mut gl) = init_window();
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
                simulator.run_rounds(rounds_per_update);
                if view.show_score {
                    partial_outcome = simulator.partial_outcome();
                }
            }

            _ => {}
        }
    }
}
