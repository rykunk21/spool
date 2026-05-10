pub struct Viewport {
    pub offset: usize,
    pub height: usize, // how many cells fit on screen
}

impl Viewport {
    /// Constructor for a viewport
    pub fn new(height: usize) -> Self {
        Viewport { offset: 0, height }
    }
    pub fn visible<'a, T>(&self, cells: &'a [T]) -> &'a [T] {
        let end = (self.offset + self.height).min(cells.len());
        &cells[self.offset..end]
    }
    pub fn visible_mut<'a, T>(&self, cells: &'a mut [T]) -> &'a mut [T] {
        let end = (self.offset + self.height).min(cells.len());
        &mut cells[self.offset..end]
    }
    pub fn scroll_down(&mut self, total: usize) {
        if self.offset + self.height < total {
            self.offset += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }
}
