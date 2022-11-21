use rltk::{Rltk, GameState};
use rltk::RandomNumberGenerator;
use specs::{World, WorldExt, Builder};
use specs::{Component, VecStorage};
use specs::Join;

mod map;
use map::Map;
use map::MapGenerator;

#[derive(Component, Debug)]
#[storage(VecStorage)]
struct Transform{
    r: usize,
    c: usize,
    ch: u16,
    color: rltk::RGB
}

enum Task{
    Idle,
}

#[derive(Component)]
#[storage(VecStorage)]
struct Worker{
    task: Task,
}

use specs::System;
use specs::{ReadExpect, ReadStorage, WriteStorage};
use specs::{RunNow};
struct WorkManager;

impl<'a> System<'a> for WorkManager{
    type SystemData = ( ReadExpect<'a, Map>,
                        ReadStorage<'a, Worker>,
                        WriteStorage<'a, Transform>);

    fn run(&mut self, data: Self::SystemData){
        let mut rand = RandomNumberGenerator::new();
        let (map, worker, mut trans) = data;

        for (worker, transform) in (&worker, &mut trans).join() {
            let (r ,c ) = (&mut transform.r, &mut transform.c);
            match &worker.task {
                Task::Idle => {
                    let new_pos = match rand.range::<i32>(0, 4) {
                        0 => (*r + 1, *c),
                        1 => (*r - 1, *c),
                        2 => (*r, *c + 1),
                        3 => (*r, *c - 1),
                        _ => panic!("rand.range in worker move returned weird value")
                    };

                    if map.at(new_pos.0, new_pos.1).walkable {
                       (*r, *c) = new_pos;
                    }
                },
            }
        }
    }
}
/*
struct Event{
    entity_id: u32,
    name: String,
    execute_time: std::time::Instant,
    action: fn(),
}
*/

struct State {
    ecs: World,
    is_mining: bool,
}

impl State {
    fn run_systems(&mut self) {
        let mut wm = WorkManager{};
        wm.run_now(&self.ecs);
        self.ecs.maintain();
    }
    fn player_input(&mut self, ctx: &mut Rltk){
        use rltk::VirtualKeyCode;
        match ctx.key {
            None => {},
            Some(key) =>
                match key {
                    VirtualKeyCode::M => self.is_mining = !self.is_mining,
                    _ => {},
                }
        }
    }
    /*pub fn add_event(entity_id: u32, event: Event){
    }*/
    /*
    fn random_walk(&self,transform: &mut Transform, rand: RandomNumberGenerator) {
            let (r, c) = (transform.r, transform.c);
            let new_pos = match rand.range::<i32>(0, 4) {
                0 => (r + 1, c),
                1 => (r - 1, c),
                2 => (r, c + 1),
                3 => (r, c - 1),
                _ => panic!("rand.range in worker move returned weird value")
            };

            if self.map.at(new_pos.0, new_pos.1).walkable {
                (transform.r, transform.c) = new_pos;
            }
    }
    */
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();
        self.player_input(ctx);
        self.run_systems();

        //let rand = RandomNumberGenerator::new();
        let map = self.ecs.fetch::<Map>();

        for r in (0..).take_while(|i| i < map.rows()) {
            for c in (0..).take_while(|i| i < map.cols()) {
                let tile = &map.at(r,c);
                ctx.set(r,c,tile.fg, tile.bg, tile.ch);
            }
        }

        if self.is_mining {
            ctx.print_color_centered_at(SCREEN_WIDTH /2, 0,  rltk::RGB::named(rltk::RED), rltk::RGB::named(rltk::BLACK)," * Mining * ");
        }

        for transform in self.ecs.read_storage::<Transform>().join(){
            let (r, c) =  (transform.r , transform.c);
            ctx.set(r, c, transform.color, map.at(r, c).bg, transform.ch);
        }
    }
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

fn main() -> rltk::BError{
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Rougelike Tutorial")
        .with_fps_cap(30.0)
        .build()?;

    let (size_x, size_y) = context.get_char_size();
    let mut map_gen = MapGenerator::new(size_x, size_y);
    map_gen.gold_count = 64;
    map_gen.gold_size = 6;
    let map = map_gen.generate();
    //let mut map = Map::new(size_x, size_y);

    let mut world = World::new();
    world.register::<Transform>();
    world.register::<Worker>();
    world.insert(map);

    world.create_entity().with(Transform {
                    r: usize::try_from(size_x).unwrap() / 2,
                    c: usize::try_from(size_y).unwrap() / 2,
                    ch: '@' as u16,
                    color: rltk::RGB::named(rltk::RED)
                }).with(
                    Worker {
                        task: Task::Idle
                    },
                ).build();

    let gs = State{
        ecs: world,
        is_mining: false,
    };


    rltk::main_loop(context, gs)
}
