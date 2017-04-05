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

enum JumpToRound {
    No,
    Later(String),
    Now(String)
}

fn main() {
    let options = Options::from_args();
    let (red, black, world) = load(&options);

    let mut simulator = Simulator::new(world.clone(), red, black, options.rounds, options.seed);
    let mut partial_outcome = Outcome::default();
    let mut view = View::new(Camera::new(SCR_WIDTH as f64, SCR_HEIGHT as f64, &simulator.world));

    let mut rounds_per_update: u32 = options.rounds_per_second / UPS;

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
                if s.len() == 0 {
                    // Enter key
                    if let JumpToRound::Later(s) = jump_to_round {
                        jump_to_round = JumpToRound::Now(s);
                    }

                    continue;
                }

                match s.as_bytes()[0] as char {
                    '+' => {
                        rounds_per_update += 100;
                        println!("[UPDATE] rounds per update: {}", rounds_per_update);
                    }
                    '-' => {
                        rounds_per_update = rounds_per_update.saturating_sub(100);
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
                        jump_to_round = JumpToRound::Later(String::new());
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
                    jump_to_finish = false;
                    partial_outcome = simulator.run();
                    continue;
                }

                if let JumpToRound::Now(round) = jump_to_round {
                    match round.parse::<u32>() {
                        Ok(round) => {
                            if round >= simulator.round {
                                // If the round is in the future, fast forward
                                let sim_round = simulator.round;
                                simulator.run_rounds(round - sim_round);
                            } else {
                                // If the round is in the past, rerun the simulation
                                simulator = simulator.reset(world.clone(), options.seed);
                                simulator.run_rounds(round);
                            }
                        }
                        Err(e) => {
                            println!("Error parsing round number in jump command: {}", e);
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

            _ => {}
        }
    }
}

// --- Auxiliary code ---

fn init_window() -> (PistonWindow, GlGraphics) {
    let opengl = OpenGL::V3_2;
    let window: Result<PistonWindow, _> =
        WindowSettings::new("AntViz", [SCR_WIDTH, SCR_HEIGHT])
                      .opengl(opengl).samples(8).build();

    match window {
        Ok(mut window) => {
            window.set_ups(UPS as u64);
            window.set_max_fps(30);

            (window, GlGraphics::new(opengl))
        }
        Err(e) => {
            fatal_error(&format!("Unable to create window: {}", e))
        }
    }
}

fn fatal_error(msg: &str) -> ! {
    println!("Fatal error: {}", msg);
    process::exit(1);
}

fn open_file_or_die(path: &str) -> BufReader<File> {
    File::open(&path).map(|file| BufReader::new(file))
                     .unwrap_or_else(|_| fatal_error(&format!("unable to open file: {}", path)))
}

fn load(options: &Options) -> (Vec<Instruction>, Vec<Instruction>, World) {
    let red = options.red.as_ref().map(|p| Instruction::parse(open_file_or_die(p))).unwrap_or_else(|| {
        println!("No file specified for red ant instructions. Using defaults.");
        test_data::ant1()
    });

    let black = options.red.as_ref().map(|p| Instruction::parse(open_file_or_die(p))).unwrap_or_else(|| {
        println!("No file specified for black ant instructions. Using defaults.");
        test_data::ant1()
    });

    let world = options.world.as_ref().map(|p| World::parse(open_file_or_die(p))).unwrap_or_else(|| {
        println!("No world file specified. Using default world.");
        test_data::sample0()
    });

    (red, black, world)
}
