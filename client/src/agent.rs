use server::Point;

pub struct Agent {
    pub(crate) current_position: Point,
}

impl Agent {
    pub fn update_position(&mut self, new_position: Point) {
        self.current_position = new_position;
    }
}
