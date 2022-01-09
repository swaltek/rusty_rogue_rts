use rltk::RGB;

#[derive(Clone)]
pub struct Tile {
    pub fg: RGB,
    pub bg: RGB,
    pub ch: u16,
    pub walkable: bool,
}

pub struct Map {
    vec: Vec<Vec<Tile>>,
    rows: usize,
    cols: usize,
}

impl Map{
    pub fn new(rows: u32, cols: u32) -> Self {
        let blank_tile = Tile {
            fg: RGB::named(rltk::GREY),
            bg: RGB::named(rltk::BLACK),
            ch: '.' as u16,
            walkable: true
        };

        let (rows, cols) = (rows.try_into().unwrap(), cols.try_into().unwrap());
        let vec = vec![vec![blank_tile ; cols]; rows];
        Self { vec ,rows, cols }
    }

    pub fn rows(&self) -> &usize {
        &self.rows
    }
    pub fn cols(&self) -> &usize {
        &self.cols
    }
    pub fn at(&self, y: usize, x: usize) -> &Tile{
        &self.vec[y][x]
    }
}

pub struct MapGenerator{
    pub rows: u32,
    pub cols: u32,
}

impl MapGenerator{
    pub fn new(rows: u32,cols: u32) -> Self{
        Self{ rows, cols }
    }
    pub fn generate_blank(&self) -> Map{
        Map::new(self.rows, self.cols)
    }
}
