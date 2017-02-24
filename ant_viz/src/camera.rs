use ant_lib::World;
use view::{CELL_WIDTH, ROW_HEIGHT};

#[derive(Clone, Copy, Default)]
pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub world_width: f64,
    pub world_height: f64,
    pub scr_width: f64,
    pub scr_height: f64,
}

impl Camera {
    pub fn new(scr_width: f64, scr_height: f64, world: &World) -> Camera {
        Camera {
            scr_width,
            scr_height,
            world_width: CELL_WIDTH * world.width as f64 + CELL_WIDTH / 2.0,
            world_height: ROW_HEIGHT * world.height as f64 + ROW_HEIGHT / 3.0,
            ..Camera::default()
        }
    }

    pub fn move_x(&mut self, units: f64) {
        self.x += units;
        self.adjust_x();
    }

    pub fn move_y(&mut self, units: f64) {
        self.y += units;
        self.adjust_y();
    }

    pub fn resize(&mut self, scr_width: u32, scr_height: u32) {
        self.scr_width = scr_width as f64;
        self.scr_height = scr_height as f64;
        self.adjust_x();
        self.adjust_y();
    }

    fn adjust_x(&mut self) {
        if self.scr_width < self.world_width {
            let right_limit = self.world_width - self.scr_width - 1.0;
            self.x = self.x.max(0.0);
            self.x = self.x.min(right_limit);
        } else {
            // If the screen is bigger than the world, we should fix the camera to 0.0
            self.x = 0.0;
        }
    }

    fn adjust_y(&mut self) {
        if self.scr_height < self.world_height {
            let bottom_limit = self.world_height - self.scr_height - 1.0;
            self.y = self.y.max(0.0);
            self.y = self.y.min(bottom_limit);
        } else {
            // If the screen is bigger than the world, we should fix the camera to 0.0
            self.y = 0.0;
        }
    }
}
