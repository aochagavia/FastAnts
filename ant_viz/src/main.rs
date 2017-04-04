extern crate ant_lib;
extern crate opengl_graphics;
extern crate piston_window;
extern crate structopt;
#[macro_use] extern crate structopt_derive;

mod camera;
mod cli;
mod view;

use std::cmp;
use std::fs::File;
use std::io::BufReader;
use std::process;

use ant_lib::{test_data, AntColor, Instruction, Outcome, Simulator, World};
use opengl_graphics::GlGraphics;
use piston_window::{AdvancedWindow, Button, EventLoop, Input, Key, OpenGL, PistonWindow, WindowSettings};
use structopt::StructOpt;

use camera::Camera;
use cli::Options;
use view::View;

const SCR_WIDTH: u32 = 1024;
const SCR_HEIGHT: u32 = 600;
const UPS: u32 = 2;

fn init_window() -> (PistonWindow, GlGraphics) {
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("AntViz", [SCR_WIDTH, SCR_HEIGHT])
        .opengl(opengl).samples(8).exit_on_esc(true).build().unwrap();

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

enum JumpToRound {
    No,
    Later(String),
    Now(String)
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

    let mut simulator = Simulator::new(world.clone(), red.clone(), black.clone(), options.rounds);
    let mut partial_outcome = Outcome::default();
    let mut view = View::new(Camera::new(SCR_WIDTH as f64, SCR_HEIGHT as f64, &simulator.world));

    let mut rounds_per_update: u32 = 1;

    let (mut window, mut gl) = init_window();
    let mut jump_to_finish = false;
    let mut jump_to_round = JumpToRound::No;
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
                if s.as_str().len() == 0 {
                    // Enter key
                    if let JumpToRound::Later(s) = jump_to_round {
                        jump_to_round = JumpToRound::Now(s);
                    }

                    continue;
                }

                match s.as_str().as_bytes()[0] as char {
                    '+' => {
                        rounds_per_update += 100;
                        println!("[UPDATE] rounds per update: {}", rounds_per_update);
                    }
                    '-' => {
                        rounds_per_update = cmp::max(0, rounds_per_update - 100);
                        println!("[UPDATE] rounds per update: {}", rounds_per_update * UPS);
                    }
                    'm' => {
                        view.toggle_marks();
                        let title = match view.show_marks {
                            None => "AntViz".to_string(),
                            Some(AntColor::Red) => "AntViz - Showing red markers".to_string(),
                            Some(AntColor::Black) => "AntViz - Showing black markers".to_string()
                        };
                        window.set_title(title);
                    }
                    't' => {
                        view.show_score = !view.show_score;
                    }
                    'f' => {
                        jump_to_finish = true;
                    }
                    'j' => {
                        // Initialize jump to round
                        jump_to_round = JumpToRound::Later(String::new());
                    }
                    'c' => {
                        // Cancel jump to round
                        jump_to_round = JumpToRound::No;
                    }
                    x if x.is_numeric() => {
                        if let JumpToRound::Later(ref mut s) = jump_to_round {
                            s.push(x);
                        }
                    }
                    _   => ()
                }

            }

            Input::Resize(width, height) => {
                view.cam.resize(width, height);
            }

            Input::Render(args) => {
                gl.draw(args.viewport(), |c, g| view.render(simulator.max_rounds, &simulator.world, &partial_outcome, c, g));
            }

            Input::Update(_) => {
                if jump_to_finish {
                    partial_outcome = simulator.run();
                } else {
                    if let JumpToRound::Now(round) = jump_to_round {
                        if round.len() > 0 {
                            let round: u32 = round.parse().unwrap();

                            if round >= simulator.round {
                                // If the round is in the future, fast forward
                                let sim_round = simulator.round;
                                simulator.run_rounds(round - sim_round);
                            } else {
                                // If the round is in the past, rerun the simulation
                                simulator = Simulator::new(world.clone(), red.clone(), black.clone(), options.rounds);
                                simulator.run_rounds(round);
                            }
                        }

                        jump_to_round = JumpToRound::No;
                    } else {
                        simulator.run_rounds(rounds_per_update);
                    }

                    if view.show_score {
                        partial_outcome = simulator.partial_outcome();
                    }
                }
            }

            _ => {}
        }
    }
}
