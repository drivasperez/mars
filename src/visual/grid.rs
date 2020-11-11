use tui::widgets::Widget;

use super::{ColorMap, VisualiserPixel};

struct PlayGrid<'a, 'b> {
    grid: &'a Vec<VisualiserPixel>,
    colors: &'b ColorMap,
}

impl<'a, 'b> PlayGrid<'a, 'b> {
    pub fn new(grid: &'a Vec<VisualiserPixel>, colors: &'b ColorMap) -> Self {
        Self { grid, colors }
    }
}

impl<'a, 'b> Widget for PlayGrid<'a, 'b> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        // see https://docs.rs/tui/0.12.0/src/tui/widgets/canvas/mod.rs.html#422-502 for impl example
        todo!()
    }
}
