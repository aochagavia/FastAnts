use std::usize;

use ant::{Ant, AntColor, AntDirection, AntState};
use instruction::{Instruction, SenseDir, TurnDir};
use util::Rng;
use world::World;

pub struct Simulator {
    pub world: World,
    red_instructions: Vec<Instruction>,
    black_instructions: Vec<Instruction>,
    pub ants: Vec<usize>,
    rng: Rng,
    pub round: u32,
    pub max_rounds: u32,
}

impl Simulator {
    pub fn new(mut world: World,
               red_instructions: Vec<Instruction>,
               black_instructions: Vec<Instruction>,
               max_rounds: u32,
               seed: u32) -> Simulator {
        let ants = world.populate();

        Simulator {
            world,
            red_instructions,
            black_instructions,
            ants,
            rng: Rng::new(seed as usize),
            round: 0,
            max_rounds
        }
    }

    pub fn reset(self, world: World, seed: u32) -> Simulator {
        Simulator::new(world,
                       self.red_instructions,
                       self.black_instructions,
                       self.max_rounds,
                       seed)
    }

    pub fn one_round(&mut self) {
        if self.round < self.max_rounds {
            self.round += 1;
        }

        // For each ant, run its current instruction
        let mut position_updates = Vec::new();
        for i in 0..self.ants.len() {
            let ant_position = self.ants[i];

            // Ignore dead ants
            if ant_position == usize::MAX {
                continue;
            }

            let (state, color) = {
                let ant = self.ant_mut(ant_position);

                if ant.resting > 0 {
                    ant.resting -= 1;
                    continue;
                }

                (ant.state, ant.color)
            };

            // Get and run the corresponding instruction
            let instruction = self.get_instruction(state, color);
            self.run_instruction(ant_position, instruction, &mut position_updates);

            // An instruction generates position updates, which are processed afterwards
            // Note: ant deaths are also propagated as position updates
            for &(old_position, new_position) in &position_updates {
                assert!(old_position != usize::MAX);
                *self.ants.iter_mut().find(|&&mut pos| pos == old_position).unwrap() = new_position;
            }

            // Clear the position updates
            position_updates.clear();
        }
    }

    pub fn run(&mut self) -> Outcome {
        for _ in self.round..self.max_rounds {
            self.one_round();
        }

        self.partial_outcome()
    }

    pub fn run_rounds(&mut self, rounds: u32) {
        if self.round >= self.max_rounds {
            return;
        }

        for _ in 0..rounds {
            self.one_round();
        }
    }

    pub fn partial_outcome(&self) -> Outcome {
        let red_score = self.world.count_red_food();
        let black_score = self.world.count_black_food();
        let red_alive = self.world.count_red_ants();
        let black_alive = self.world.count_black_ants();
        let food_left = self.world.count_food();
        let round = self.round;

        Outcome { red_score, red_alive, black_score, black_alive, food_left, round }
    }

    fn get_instruction(&self, state: AntState, color: AntColor) -> Instruction {
        match color {
            AntColor::Red => self.red_instructions[state as usize].clone(),
            AntColor::Black => self.black_instructions[state as usize].clone()
        }
    }

    fn ant(&self, pos: usize) -> &Ant {
        self.world.cells[pos].ant.as_ref().unwrap()
    }

    fn ant_mut(&mut self, pos: usize) -> &mut Ant {
        self.world.cells[pos].ant.as_mut().unwrap()
    }

    fn run_instruction(&mut self, ant_pos: usize, instruction: Instruction, position_updates: &mut Vec<(usize, usize)>) {
        // Run the instruction
        use self::Instruction::*;
        match instruction {
            Sense(sense_dir, st1, st2, cond) => {
                let (ant_dir, ant_color) = {
                    let ant = self.ant(ant_pos);
                    (ant.direction, ant.color)
                };

                let new_state = {
                    let sensed_position = sensed_position(self.world.width, ant_pos, ant_dir, sense_dir);
                    let sensed_cell = &self.world.cells[sensed_position];
                    if cond.eval(sensed_cell, ant_color) {
                        st1
                    } else {
                        st2
                    }
                };

                self.ant_mut(ant_pos).state = new_state
            }
            Mark(mark, new_state) => {
                let cell = &mut self.world.cells[ant_pos];
                let ant_color = cell.ant.as_ref().unwrap().color;
                cell.markers_mut(ant_color).set_bit(mark);
                cell.ant.as_mut().unwrap().state = new_state
            }
            Unmark(mark, new_state) => {
                let cell = &mut self.world.cells[ant_pos];
                let ant = cell.ant.as_mut().unwrap();
                if let AntColor::Red = ant.color {
                    cell.markers_red.clear(mark);
                } else {
                    cell.markers_black.clear(mark);
                }

                ant.state = new_state
            }
            PickUp(success_state, failure_state) => {
                let cell = &mut self.world.cells[ant_pos];
                let ant = cell.ant.as_mut().unwrap();
                if ant.has_food || cell.food == 0 {
                    ant.state = failure_state;
                } else {
                    cell.food -= 1;
                    ant.has_food = true;
                    ant.state = success_state;
                }
            }
            Drop(new_state) => {
                let cell = &mut self.world.cells[ant_pos];
                let ant = cell.ant.as_mut().unwrap();
                if ant.has_food {
                    cell.food += 1;
                    ant.has_food = false;
                    ant.state = new_state;
                }

                ant.state = new_state;
            }
            Turn(turn_direction, new_state) => {
                let ant = self.ant_mut(ant_pos);
                ant.direction = ant.direction.turn(turn_direction);
                ant.state = new_state;
            }
            Move(success_state, failure_state) => {
                let ant_dir = self.ant(ant_pos).direction;
                let target_pos = World::adjacent_position(self.world.width, ant_pos, ant_dir);

                let target_occupied = {
                    let target_cell = &self.world.cells[target_pos];
                    target_cell.is_rocky || target_cell.ant.is_some()
                };

                // Stop here if the target cell is occupied
                if target_occupied {
                    self.ant_mut(ant_pos).state = failure_state;
                    return;
                }

                // Take the ant from the current place and put it in the target cell
                let ant = self.world.cells[ant_pos].ant.take();
                self.world.cells[target_pos].ant = ant;

                // Generate a position update
                position_updates.push((ant_pos, target_pos));

                {
                    // Don't forget to rest and update the state
                    let ant = self.ant_mut(target_pos);
                    ant.resting = 14;
                    ant.state = success_state;
                }

                self.kill_surrounded_ants(target_pos, position_updates);
            }
            Flip(n, st1, st2) => {
                let random = self.rng.random_int(n as usize);
                let new_state = if random == 0 { st1 } else { st2 };
                self.ant_mut(ant_pos).state = new_state;
            }
        }
    }

    fn kill_surrounded_ants(&mut self, position: usize, position_updates: &mut Vec<(usize, usize)>) {
        // Check if this ant is surrounded
        self.kill_surrounded_ant(position, position_updates);

        // For each adjacent cell, check if there is a surrounded ant
        for direction in AntDirection::all() {
            let adjacent_position = World::adjacent_position(self.world.width, position, direction);
            self.kill_surrounded_ant(adjacent_position, position_updates);
        }
    }

    fn kill_surrounded_ant(&mut self, position: usize, position_updates: &mut Vec<(usize, usize)>) {
        let mut this_ant_dead = false;
        if let Some(ref this_ant) = self.world.cells[position].ant {
            this_ant_dead = self.world.adjacent_enemies(position, this_ant.color) >= 5;
        }

        if this_ant_dead {
            // Remove from cell and drop food
            let cell = &mut self.world.cells[position];
            let ant = cell.ant.take().unwrap();
            cell.food += 3;
            if ant.has_food {
                cell.food += 1;
            }

            // Set the position to an invalid value
            position_updates.push((position, usize::MAX));
        }
    }
}

fn sensed_position(width: usize, ant_index: usize, ant_direction: AntDirection, sense_direction: SenseDir) -> usize {
    use self::SenseDir::*;
    match sense_direction {
        Here       => ant_index,
        Ahead      => World::adjacent_position(width, ant_index, ant_direction),
        LeftAhead  => World::adjacent_position(width, ant_index, ant_direction.turn(TurnDir::Left)),
        RightAhead => World::adjacent_position(width, ant_index, ant_direction.turn(TurnDir::Right))
    }
}

#[derive(Debug, Default)]
pub struct Outcome {
    pub red_score: u16,
    pub red_alive: u16,
    pub black_score: u16,
    pub black_alive: u16,
    pub food_left: u16,
    pub round: u32
}
