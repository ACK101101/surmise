pub struct Win {
    win: Window
}

impl Win {
    pub fn new() -> Result<Window> {
        const WINDOW_WIDTH: usize = 960;
        const WINDOW_HEIGHT: usize = 540;
        log::debug!("Window Dims: ({}, {})", WINDOW_WIDTH, WINDOW_HEIGHT);

        let mut pixel_dims: Rect = Rect {
            width: 32,
            height: 16,
        };
        log::debug!("Pixel Chunk Dims: ({}, {})", pixel_dims.width, pixel_dims.height);

        let mut win = Window::new(
            "surmise", 
            WINDOW_WIDTH, WINDOW_HEIGHT, 
            WindowOptions { 
                borderless: true, resize: true, transparency: true, ..WindowOptions::default()
            },
        ).unwrap(); 
    }
}