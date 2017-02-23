extern crate ant_lib;

//use std::io;
//use world::World;

use ant_lib::{test_data, Simulator};

fn main() {
    // Boilerplate to read lines from stdin
    /*
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();

    // First, read the world
    let world = World::parse(&mut stdin_lock);

    // Then the instructions for the ants
    let red = instruction::parse(&mut stdin_lock);
    let black = instruction::parse(&mut stdin_lock);
    */

    let world = test_data::sample0();
    let instr = test_data::default_program();

    let mut simulator = Simulator::new(world, instr.clone(), instr);
    println!("{:?}", simulator.run_rounds(100_000));
}
