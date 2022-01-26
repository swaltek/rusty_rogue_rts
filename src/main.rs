use rltk::{Rltk, GameState};
//use rltk::RGB;
use hecs::World;

mod map;
use map::Map;
use map::MapGenerator;

struct State {
    world: World,
    map: Map,
}

#[derive(Debug)]
struct Name ( String );

/*
struct Transform{
    x: i32,
    y: i32,
    c: char,
}
*/


impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();
        ctx.print(1, 1, "Hello Rust World");


        for r in (0..).take_while(|i| i < self.map.rows()) {
            for c in (0..).take_while(|i| i < self.map.cols()) {
                let tile = &self.map.at(r,c);
                ctx.set(r,c,tile.fg, tile.bg, tile.ch);
            }
        }

        let mut line_num = 2;
        for (id, name) in &mut self.world.query::<&Name>() {
            ctx.print(1, line_num, format!("Hello, {}! ID: {:?}", name.0, id));
            line_num += 1;
        }
    }
}

fn main() -> rltk::BError{
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Rougelike Tutorial")
        .build()?;

    let mut world = World::new();

    let (size_x, size_y) = context.get_char_size();
    let mut map_gen = MapGenerator::new(size_x, size_y);
    map_gen.gold_count = 64;
    map_gen.gold_size = 6;
    let map = map_gen.generate();
    //let mut map = Map::new(size_x, size_y);

    world.spawn((Name("Karen".to_string()),));
    world.spawn((Name("Loren".to_string()),));
    world.spawn((Name("Charlie".to_string()),));
    world.spawn((Name("Justin".to_string()),));

    let gs = State{world: world, map: map};


    rltk::main_loop(context, gs)
}
