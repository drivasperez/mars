use tui::style::Color;
use tui::widgets::{Block, Widget};

use super::VisualiserPixel;

pub struct PlayGrid<'a, 'b> {
    grid: &'a Vec<VisualiserPixel>,
    block: Option<Block<'b>>,
}

impl<'a, 'b> PlayGrid<'a, 'b> {
    pub fn new(grid: &'a Vec<VisualiserPixel>) -> Self {
        Self { grid, block: None }
    }

    pub fn block(mut self, block: Block<'b>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a, 'b> Widget for PlayGrid<'a, 'b> {
    fn render(mut self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        let width = usize::from(area.width);
        for (i, cell) in self.grid.iter().enumerate() {
            let (color, ch) = match cell {
                VisualiserPixel::Uninitialised => (Color::White, '.'),
                VisualiserPixel::Initialised(c) => (*c, '-'),
                VisualiserPixel::Touched(c) => (*c, '+'),
                VisualiserPixel::Executing => (Color::LightRed, 'o'),
            };

            if ch != ' ' && ch != '\u{2800}' {
                let (x, y) = (i % width, i / width);
                buf.get_mut(x as u16 + area.left(), y as u16 + area.top())
                    .set_char(ch)
                    .set_fg(color);
            }
        }
    }
}
