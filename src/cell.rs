#[derive(Clone)]
pub struct Cell {
    pub id: usize,
    pub source: String,
    pub output: String,
}

impl Cell {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            source: String::new(),
            output: String::new(),
        }
    }
    pub fn run(&mut self) {
        todo!();
        /*match engine::execute(self.source) {
            Ok(res) => Cell.output = res,
            Err(e) => panic!(e),
        }*/

        self.source = "This is new source".into();
    }
}
