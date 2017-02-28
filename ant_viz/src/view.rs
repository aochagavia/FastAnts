use std::env;

use ant_lib::{AntColor, AntDirection, Cell, Outcome, World};
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;
use piston_window::{self, Context, Transformed};

use camera::Camera;

pub const CELL_WIDTH: f64 = 20.0;
const INNER_CELL_WIDTH: f64 = CELL_WIDTH - 2.0 * CELL_BORDER;
const CELL_BORDER: f64 = 1.0;
pub const ROW_HEIGHT: f64 = 3.0 * CELL_WIDTH / 4.0;

const MARKER_COLORS: [[f32; 4]; 6] = [
    [1.0, 1.0, 1.0, 0.5],
    [1.0, 0.0, 1.0, 0.5],
    [1.0, 1.0, 0.0, 0.5],
    [0.0, 1.0, 1.0, 0.5],
    [0.0, 0.0, 1.0, 0.5],
    [0.0, 1.0, 0.0, 0.5],
];

pub struct View {
    pub cam: Camera,
    pub font: GlyphCache<'static>,
    pub show_marks: Option<AntColor>,
    pub show_score: bool
}

impl View {
    pub fn new(cam: Camera) -> View {
        let exe_directory = env::current_exe().unwrap().parent().unwrap().to_owned();
        let font = GlyphCache::new(exe_directory.join("resources/FiraMono-Bold.ttf")).unwrap();
        View { cam, font, show_marks: None, show_score: false }
    }

    pub fn toggle_marks(&mut self) {
        let next_color = match self.show_marks {
            None => Some(AntColor::Red),
            Some(AntColor::Red) => Some(AntColor::Black),
            Some(AntColor::Black) => None,
        };

        self.show_marks = next_color;
    }

    pub fn render(&mut self, world: &World, outcome: &Outcome, c: Context, g: &mut GlGraphics) {
        let abs_trans = c.transform;
        let trans = c.transform.trans(-self.cam.x, -self.cam.y);

        piston_window::clear([0.0, 0.0, 0.0, 1.0], g);
        piston_window::rectangle([1.0, 1.0, 1.0, 0.1],
                                 [0.0, 0.0, self.cam.world_width, self.cam.world_height],
                                 trans,
                                 g);

        // The cells
        let outer_polygon = cell_polygon(CELL_WIDTH);
        let inner_polygon = cell_polygon(INNER_CELL_WIDTH);
        let ant_polygon = ant_polygon();
        let marker_polygon = marker_polygon();
        for (i, cell) in world.cells.iter().enumerate() {
            let (x, y) = World::index_to_coords(world.width, i);
            let x_offset = if y % 2 != 0 { CELL_WIDTH / 2.0 } else { 0.0 };

            // Convert the coordinates to pixel coordinates
            let (x, y) = (x_offset + x as f64 * CELL_WIDTH, y as f64 * ROW_HEIGHT);
            let (center_x, center_y) = (x + CELL_WIDTH / 2.0, y + ROW_HEIGHT / 3.0 * 2.0);

            let (border_color, fill_color) = cell_color(cell);

            // The outer border
            piston_window::polygon(
                border_color,
                &outer_polygon,
                trans.trans(x, y),
                g);

            // The fill of the polygon
            piston_window::polygon(
                fill_color,
                &inner_polygon,
                trans.trans(x + CELL_BORDER, y + CELL_BORDER),
                g);

            // Markers per color
            if let Some(markers) = self.show_marks.map(|color| cell.markers(color)) {
                for mark in markers.iter() {
                    // Triangle in the right direction
                    piston_window::polygon(
                        MARKER_COLORS[mark as usize],
                        &marker_polygon,
                        trans.trans(center_x, center_y).rot_rad(rotation(mark)),
                        g);
                }
            }

            // Food
            if cell.food > 0 {
                piston_window::polygon(
                    fill_color,
                    &inner_polygon,
                    trans.trans(x + CELL_BORDER, y + CELL_BORDER),
                    g);

                piston_window::text([0.6, 0.0, 1.0, 1.0],
                                    10,
                                    &cell.food.to_string(),
                                    &mut self.font,
                                    trans.trans(x + 0.2 * CELL_WIDTH, y + ROW_HEIGHT),
                                    g);
            }

            // Ants
            if let Some(ref ant) = cell.ant {
                let ant_trans = trans.trans(center_x, center_y).rot_rad(ant_rotation(ant.direction));
                piston_window::polygon(
                    ant_color(ant.color),
                    &ant_polygon,
                    ant_trans,
                    g);

                // Food is shown as a white rectangle on the head of the ant
                if ant.has_food {
                    piston_window::rectangle(
                        [1.0, 1.0, 1.0, 1.0],
                        [0.0, 0.0, 4.0, 4.0],
                        ant_trans.trans(CELL_WIDTH / 2.0 - 2.0, -2.0),
                        g);
                }
            }


        }

        if self.show_score {
            let red_score_str = outcome.red_score.to_string();
            let red_alive_str = outcome.red_alive.to_string();
            let black_score_str = outcome.black_score.to_string();
            let black_alive_str = outcome.black_alive.to_string();
            let food_left_str = format!("Food left: {}", outcome.food_left);
            let food_left_pos = self.cam.scr_height - 50.0;

            // Center the rectangle
            let abs_trans = if self.cam.scr_width > 600.0 {
                let left_margin = (self.cam.scr_width - 600.0) / 2.0;
                abs_trans.trans(left_margin, 0.0)
            } else {
                abs_trans
            };

            // Lightly transparent dark rectangle
            piston_window::rectangle(
                [0.0, 0.0, 0.0, 0.6],
                [0.0, 0.0, 600.0, self.cam.scr_height],
                abs_trans,
                g);

            let white = [1.0, 1.0, 1.0, 1.0];
            let size = 24;

            let mut print = |s, t: &mut [[f64; 3]; 2]| {
                piston_window::text(white, size, s, &mut self.font, *t, g);
                *t = t.trans(0.0, 44.0);
            };

            // Left column
            let mut trans = abs_trans.trans(50.0, 100.0);
            print("Score", &mut trans);
            print("Ants alive", &mut trans);

            // Middle column
            let mut trans = abs_trans.trans(300.0, 50.0);
            print("Black", &mut trans);
            print(&black_score_str, &mut trans);
            print(&black_alive_str, &mut trans);

            // Right column
            let mut trans = abs_trans.trans(450.0, 50.0);
            print("Red", &mut trans);
            print(&red_score_str, &mut trans);
            print(&red_alive_str, &mut trans);

            // Bottom, centered
            let mut trans = abs_trans.trans(200.0, food_left_pos);
            print(&food_left_str, &mut trans);
        }
    }
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
    // The polygon points to the right
    let length = CELL_WIDTH / 2.0;
    [
        // Top-left corner
        [0.0, -1.5],
        // Top-right corner
        [length, -1.5],
        // Bottom-right corner
        [length, 1.5],
        // Bottom-left corner
        [0.0, 1.5]
    ]
}

fn cell_polygon(width: f64) -> [[f64; 2]; 6] {
    // Note: the origin is at the top-left (outside the )
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
