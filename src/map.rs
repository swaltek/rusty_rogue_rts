use rltk::RGB;
use rltk::RandomNumberGenerator;

#[derive(Clone, Copy)]
pub struct Tile {
    pub fg: RGB,
    pub bg: RGB,
    pub ch: u16,
    pub walkable: bool,
}

pub fn blank_tile() -> Tile{
    Tile {
        fg: RGB::named(rltk::WHITE),
        bg: RGB::named(rltk::GREY),
        ch: ' ' as u16,
        walkable: true
    }
}

pub fn default_wall() -> Tile{
    Tile {
        fg: RGB::named(rltk::WHITE),
        bg: RGB::named(rltk::GREY),
        ch: '#' as u16,
        walkable: false
    }
}

#[derive(Clone)]
pub struct Map {
    vec: Vec<Vec<Tile>>,
    rows: usize,
    cols: usize,
}

impl Map{
    pub fn new(rows: usize, cols: usize) -> Self {

        let vec = vec![vec![default_wall() ; cols]; rows];
        Self { vec ,rows, cols }
    }

    pub fn rows(&self) -> u32 {
        self.rows.try_into().unwrap()
    }
    pub fn cols(&self) -> u32 {
        self.cols.try_into().unwrap()
    }
    pub fn at(&self, r: u32, c: u32) -> &Tile{
        let (y, x) : (usize, usize) = (r.try_into().unwrap(), c.try_into().unwrap());
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

fn clear_room(map: Map, y: u32, x: u32, rows: u32, cols: u32) -> Map{
    let mut ret_map = map.clone();
    for xi in x..(x+cols){
        for yi in y..(y+rows){
            ret_map.set(yi.try_into().unwrap(), xi.try_into().unwrap(), blank_tile());
        }
    }
    ret_map
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

        const INIT_ROOM_SIZE: u32 = 10;
        clear_room(map,
            self.rows as u32/2 - (INIT_ROOM_SIZE/2),
            self.cols as u32/2 - (INIT_ROOM_SIZE/2),
            INIT_ROOM_SIZE, INIT_ROOM_SIZE)
    }

}
