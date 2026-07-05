#[derive(Copy, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl From<(isize, isize)> for Point {
    fn from((x, y): (isize, isize)) -> Self {
        Point {
            x: x as i32,
            y: y as i32,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    width: u32,
    height: u32,
}

impl Rect {
    pub fn new(width: u32, height: u32) -> Rect {
        Rect { width, height }
    }

    pub fn get_dims(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    pub fn can_contain(&self, other: &Rect) -> bool {
        self.width >= other.width && self.height >= other.height
    }
}

impl std::ops::Div for Rect {
    type Output = Rect;
    fn div(self, rhs: Rect) -> Rect {
        Rect {
            width: self.width / rhs.width,
            height: self.height / rhs.height,
        }
    }
}

impl From<(usize, usize)> for Rect {
    fn from((width, height): (usize, usize)) -> Self {
        Rect {
            width: width as u32,
            height: height as u32,
        }
    }
}
