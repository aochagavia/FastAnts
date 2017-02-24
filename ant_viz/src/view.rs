use std::env;

use ant_lib::{AntColor, AntDirection, Cell, World};
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;
use piston_window::{self, Context, Transformed};

use camera::Camera;

pub const CELL_WIDTH: f64 = 20.0;
const CELL_BORDER: f64 = 1.0;
pub const ROW_HEIGHT: f64 = 3.0 * CELL_WIDTH / 4.0;
const ANT_WIDTH: f64 = 3.0;

const MARKER_COLORS: [[f32; 4]; 6] = [
    [1.0, 1.0, 1.0, 0.7],
    [1.0, 0.0, 1.0, 0.7],
    [1.0, 1.0, 0.0, 0.7],
    [0.0, 1.0, 1.0, 0.7],
    [0.0, 0.0, 1.0, 0.7],
    [0.0, 1.0, 0.0, 0.7],
];

pub struct View {
    pub cam: Camera,
    pub font: GlyphCache<'static>,
    pub show_marks: Option<AntColor>
}

impl View {
    pub fn new(cam: Camera) -> View {
        let exe_directory = env::current_exe().unwrap().parent().unwrap().to_owned();
        let font = GlyphCache::new(exe_directory.join("resources/FiraMono-Bold.ttf")).unwrap();
        View { cam, font, show_marks: None }
    }

    pub fn toggle_marks(&mut self) {
        let next_color = match self.show_marks {
            None => Some(AntColor::Red),
            Some(AntColor::Red) => Some(AntColor::Black),
            Some(AntColor::Black) => None,
        };

        self.show_marks = next_color;
    }

    pub fn render(&mut self, world: &World, c: Context, g: &mut GlGraphics) {
        let trans = c.transform.trans(-self.cam.x, -self.cam.y);

        piston_window::clear([0.0, 0.0, 0.0, 1.0], g);
        piston_window::rectangle([1.0, 1.0, 1.0, 0.1],
                                 [0.0, 0.0, self.cam.world_width, self.cam.world_height],
                                 trans,
                                 g);

        // The cells
        let outer_polygon = cell_polygon(CELL_WIDTH);
        let inner_polygon = cell_polygon(CELL_WIDTH - 2.0 * CELL_BORDER);
        let ant_polygon = ant_polygon();
        let marker_polygon = marker_polygon();
        for (i, cell) in world.cells.iter().enumerate() {
            let (x, y) = World::index_to_coords(world.width, i);
            let x_offset = if y % 2 != 0 { CELL_WIDTH / 2.0 } else { 0.0 };

            let (x, y) = (x as f64, y as f64);
            let (border_color, fill_color) = cell_color(cell);

            // The outer border
            piston_window::polygon(
                border_color,
                &outer_polygon,
                trans.trans(x_offset + x * CELL_WIDTH, y * ROW_HEIGHT),
                g);

            // The fill of the polygon
            piston_window::polygon(
                fill_color,
                &inner_polygon,
                trans.trans(x_offset + x * CELL_WIDTH + CELL_BORDER, y * ROW_HEIGHT + CELL_BORDER),
                g);

            // Markers per color
            if let Some(markers) = self.show_marks.map(|color| cell.markers(color)) {
                for mark in markers.iter() {
                    // Triangle in the right direction
                    piston_window::polygon(MARKER_COLORS[mark as usize],
                                        &marker_polygon,
                                        trans.trans(x_offset + x * CELL_WIDTH + CELL_WIDTH / 2.0, y * ROW_HEIGHT + ROW_HEIGHT / 3.0 * 2.0)
                                                .rot_rad(rotation(mark)),
                                        g);
                }
            }

            // Food
            if cell.food > 0 {
                piston_window::polygon(
                    fill_color,
                    &inner_polygon,
                    trans.trans(x_offset + x * CELL_WIDTH + CELL_BORDER, y * ROW_HEIGHT + CELL_BORDER),
                    g);

                piston_window::text([0.6, 0.0, 1.0, 1.0],
                                    10,
                                    &cell.food.to_string(),
                                    &mut self.font,
                                    trans.trans(x_offset + (x + 0.2) * CELL_WIDTH, (y + 1.0) * ROW_HEIGHT),
                                    g);
            }

            // Ants
            if let Some(ref ant) = cell.ant {
                piston_window::polygon(
                    ant_color(ant.color),
                    &ant_polygon,
                    trans.trans(x_offset + x * CELL_WIDTH + CELL_WIDTH / 2.0,
                                y * ROW_HEIGHT + CELL_WIDTH / 2.0 - ANT_WIDTH / 2.0)
                        .rot_rad(ant_rotation(ant.direction)),
                    g);
            }


        }

        // Information bar? Show information when clicking something
    }
}

pub fn coords_to_index(cam: Camera, x: f64, y: f64) -> usize {
    // First, map the x in the screen to the x in the world size
    // ?
    unimplemented!()
}

fn ant_color(color: AntColor) -> [f32; 4] {
    match color {
        AntColor::Red => [0.5, 0.0, 0.0, 1.0],
        AntColor::Black => [0.0, 0.0, 0.0, 1.0]
    }
}

fn ant_rotation(dir: AntDirection) -> f64 {
    // 0 -> 0
    // 6 -> 2 * PI
    (dir as usize as f64) / 6.0 * (2.0 * ::std::f64::consts::PI)
}

fn rotation(dir: u8) -> f64 {
    // 0 -> 0
    // 6 -> 2 * PI
    (dir as f64) / 6.0 * (2.0 * ::std::f64::consts::PI)
}

fn cell_color(cell: &Cell) -> ([f32; 4], [f32; 4]) {
    let border = match cell.anthill {
        Some(AntColor::Red) => [0.5, 0.0, 0.0, 1.0],
        Some(AntColor::Black) => [0.0, 0.0, 0.0, 1.0],
        None => [0.0, 0.0, 0.0, 0.0]
    };

    let fill = if cell.is_rocky {
        [1.0, 1.0, 1.0, 0.1]
    } else {
        [1.0, 1.0, 1.0, 0.3]
    };

    (border, fill)
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

fn marker_polygon() -> [[f64; 2]; 3] {
    [
        // Center of the hexagon
        [0.0, 0.0],
        // Top-right corner
        [CELL_WIDTH / 2.0 - CELL_BORDER, -CELL_WIDTH / 3.0 + 2.0 * CELL_BORDER],
        // Bottom-right corner
        [CELL_WIDTH / 2.0 - CELL_BORDER, CELL_WIDTH / 3.0 - 2.0 * CELL_BORDER],
    ]
}
