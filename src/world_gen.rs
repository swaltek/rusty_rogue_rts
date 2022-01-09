#[derive(Clone)]
pub struct Tile {
    fg: RGB,
    bg: RGB,
    ch: u16,
    walkable: bool,
}

struct Map {
    vec: Vec<Vec<Tile>>,
    rows: usize,
    cols: usize,
}

impl Map{
    fn new(rows: usize, cols: usize) -> Self {
        let blankTile = Tile {
            fg: RGB::named(rltk::GREY),
            bg: RGB::named(rltk::BLACK),
            ch: '.' as u16,
            walkable: true
        };
        let vec = vec![vec![blankTile ; cols]; rows];
        Self { vec ,rows, cols }
    }
}
