use rltk::RGB;
use rltk::RandomNumberGenerator;

#[derive(Clone, Copy)]
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
    pub fn new(rows: usize, cols: usize) -> Self {
        let blank_tile = Tile {
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::GREY),
            ch: '#' as u16,
            walkable: false
        };

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
    pub fn set(&mut self, y: usize, x: usize, tile: Tile){
        self.vec[y][x] = tile;
    }
    pub fn is_on(&self, y: i32, x: i32) -> bool{
        if y < 0 || x < 0  { return false; }
        if y >= self.rows as i32 || x >= self.cols as i32 { return false; }
        true
    }
}

pub struct MapGenerator{
    pub rows: usize,
    pub cols: usize,

    pub gold_size: u32,
    pub gold_count: u32,
}

impl MapGenerator{
    pub fn new(rows: u32,cols: u32) -> Self{
        Self{
            rows: rows.try_into().unwrap(),
            cols: cols.try_into().unwrap(),
            gold_size: 0,
            gold_count: 0
        }
    }
    pub fn generate_blank(&self) -> Map{
        Map::new(self.rows, self.cols)
    }

    pub fn generate(&self) -> Map{
        let mut rand = RandomNumberGenerator::new();
        let mut map = self.generate_blank();

        let gold_tile = Tile {
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::GREY),
            ch: '#' as u16,
            walkable: false
        };

        for _ in 0..self.gold_count{
            let (mut y, mut x) = (rand.range(1, self.rows as i32) , rand.range(1, self.cols as i32));
            for _ in 0..=self.gold_size{
                map.set(y.try_into().unwrap(), x.try_into().unwrap(), gold_tile);
                let (new_y, new_x) = match rand.range::<i32>(0, 4) {
                    0 => (y + 1, x),
                    1 => (y - 1, x),
                    2 => (y, x + 1),
                    3 => (y, x - 1),
                    _ => panic!("rand.range in map generation returned weird value")
                };
                if map.is_on(new_y, new_x) {
                    y = new_y;
                    x = new_x;
                }
            }
        }

        map
    }

}
