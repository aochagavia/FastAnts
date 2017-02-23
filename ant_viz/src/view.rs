use ant_lib::{AntColor, AntDirection, Cell, World};
use opengl_graphics::GlGraphics;
use piston_window::{self, Context, Transformed};

use camera::Camera;

const CELL_WIDTH: f64 = 20.0;
const CELL_BORDER: f64 = 1.0;
const ANT_WIDTH: f64 = 3.0;

pub fn render(world: &World, cam: Camera, c: Context, g: &mut GlGraphics) {
    piston_window::clear([0.0, 0.0, 0.0, 1.0], g);

    // The cells
    let outer_polygon = cell_polygon(CELL_WIDTH);
    let inner_polygon = cell_polygon(CELL_WIDTH - 2.0 * CELL_BORDER);
    let ant_polygon = ant_polygon();
    let trans = c.transform.trans(cam.x, cam.y);
    for (i, cell) in world.cells.iter().enumerate() {
        let (x, y) = World::index_to_coords(world.width, i);
        let x_offset = if y % 2 == 0 { CELL_WIDTH / 2.0 } else { 0.0 };

        let (x, y) = (x as f64, y as f64);

        // The outer border
        piston_window::polygon(
            [1.0, 1.0, 1.0, 0.1],
            &outer_polygon,
            trans.trans(x_offset + x * CELL_WIDTH, y * (3.0 * CELL_WIDTH / 4.0)),
            g);

        // The fill of the polygon
        piston_window::polygon(
            cell_color(cell),
            &inner_polygon,
            trans.trans(x_offset + x * CELL_WIDTH + CELL_BORDER, y * (3.0 * CELL_WIDTH / 4.0) - CELL_BORDER),
            g);

        // Ants
        if let Some(ref ant) = cell.ant {
            piston_window::polygon(
                ant_color(ant.color),
                &ant_polygon,
                trans.trans(x_offset + x * CELL_WIDTH + CELL_WIDTH / 2.0,
                            y * (3.0 * CELL_WIDTH / 4.0) + CELL_WIDTH / 2.0 - ANT_WIDTH / 2.0)
                     .rot_rad(ant_rotation(ant.direction)),
                g);
        }

        // Food
    }

    // Information bar?
}

fn ant_color(color: AntColor) -> [f32; 4] {
    match color {
        AntColor::Red => [0.5, 0.0, 0.0, 1.0],
        AntColor::Black => [0.0, 0.0, 0.0, 1.0]
    }
}

fn ant_rotation(dir: AntDirection) -> f64 {
    (dir as usize as f64) / 2.0 * ::std::f64::consts::PI
}

fn cell_color(cell: &Cell) -> [f32; 4] {
    if cell.is_rocky {
        [1.0, 1.0, 1.0, 0.1]
    } else {
        [1.0, 1.0, 1.0, 0.3]
    }
}

fn ant_polygon() -> [[f64; 2]; 4] {
    // Ant points to the right
    let length = CELL_WIDTH / 2.0;
    [
        // Top-left corner
        [0.0, 0.0],
        // Top-right corner
        [length, 0.0],
        // Bottom-right corner
        [length, 3.0],
        // Bottom-left corner
        [0.0, 3.0]
    ]
}

fn cell_polygon(width: f64) -> [[f64; 2]; 6] {
    [
        // Top-right corner
        [width, width / 4.0],
        // Bottom-right
        [width, width - width / 4.0],
        // Down
        [width / 2.0, width],
        // Bottom-left
        [0.0, width - width / 4.0],
        // Top-left
        [0.0, width / 4.0],
        // Top
        [width / 2.0, 0.0]
    ]
}

/*
[
        // Top-right corner
        [width / 2.0, width / 4.0],
        // Bottom-right
        [width / 2.0, -width / 4.0],
        // Down
        [0.0, -width / 2.0],
        // Bottom-left
        [-width / 2.0, -width / 4.0],
        // Top-left
        [-width / 2.0, width / 4.0],
        // Top
        [0.0, width / 2.0]
    ]
*/
