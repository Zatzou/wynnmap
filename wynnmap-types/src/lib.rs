use serde::{Deserialize, Serialize};

pub mod ws;

pub mod guild;
pub mod maptile;
pub mod terr;

pub mod api;

/// A rectangular region in the minecraft world
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Region {
    /// Start positions of the region. First value is the X and second is the Z
    pub start: [i32; 2],
    /// End positions of the region. First value is the X and second is the Z
    pub end: [i32; 2],
}

impl Region {
    pub const fn left_side(&self) -> i32 {
        if self.start[0] < self.end[0] {
            self.start[0]
        } else {
            self.end[0]
        }
    }

    pub const fn right_side(&self) -> i32 {
        if self.start[0] > self.end[0] {
            self.start[0]
        } else {
            self.end[0]
        }
    }

    pub const fn top_side(&self) -> i32 {
        if self.start[1] < self.end[1] {
            self.start[1]
        } else {
            self.end[1]
        }
    }

    pub const fn bottom_side(&self) -> i32 {
        if self.start[1] > self.end[1] {
            self.start[1]
        } else {
            self.end[1]
        }
    }

    pub const fn width(&self) -> u32 {
        self.start[0].abs_diff(self.end[0])
    }

    pub const fn height(&self) -> u32 {
        self.start[1].abs_diff(self.end[1])
    }

    /// calculate midpoint on x (horizontal scale)
    pub const fn midpoint_x(&self) -> i32 {
        i32::midpoint(self.start[0], self.end[0])
    }

    /// calculate midpoint on z (vertical scale)
    pub const fn midpoint_y(&self) -> i32 {
        i32::midpoint(self.start[1], self.end[1])
    }

    pub const fn get_midpoint(&self) -> (i32, i32) {
        (self.midpoint_x(), self.midpoint_y())
    }
}
