use rltk::{Rltk, GameState};
use rltk::RandomNumberGenerator;
use specs::{World, WorldExt, Builder};
use specs::{Component, VecStorage};
use specs::Join;

mod map;
use map::Map;
use map::MapGenerator;

mod time;
use time::{Actor, ActionType};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Transform{
    r: u32,
    c: u32,
    ch: u16,
    color: rltk::RGB
}


#[derive(Component)]
#[storage(VecStorage)]
struct Worker{
    task: WorkerTask,
}

enum WorkerTask{
    Idle,
}

use specs::System;
use specs::{Write, ReadExpect, WriteExpect, ReadStorage, WriteStorage};
use specs::{RunNow};

struct WorkManager;

impl<'a> System<'a> for WorkManager{
    type SystemData = ( ReadExpect<'a, Map>,
                        ReadStorage<'a, Worker>,
                        WriteStorage<'a, Actor>);

    fn run(&mut self, data: Self::SystemData){
        let mut rand = RandomNumberGenerator::new();
        let (map, worker, mut actors) = data;

        for (worker, act) in (&worker, &mut actors).join() {
            match &worker.task {
                WorkerTask::Idle => {
                    let (dr, dc) = match rand.range::<i32>(0, 4) {
                        0 => (1, 0),
                        1 => (-1, 0),
                        2 => (0, 1),
                        3 => (0, -1),
                        _ => panic!("rand.range in worker move returned weird value")
                    };
                    if !act.is_busy() {
                        act.new_action(ActionType::Move(dr, dc));
                    }
                },
            }
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
struct Selectable{
    selected: bool,
}

struct SelectEvent(Option<(u32, u32, u32, u32)>); // r, c, w, h
impl Default for SelectEvent {
    fn default() -> Self {
        Self(None)
    }
}

struct SelectHandler;

impl<'a> System<'a> for SelectHandler{
    type SystemData = ( Write<'a, SelectEvent>,
                        ReadStorage<'a, Transform>,
                        WriteStorage<'a, Selectable>);

    fn run(&mut self, data: Self::SystemData){
        let (mut select_event, trans,mut selectable) = data;
        if let SelectEvent(Some((select_r, select_c, select_w, select_h))) = *select_event {
            for (transform, select) in (&trans, &mut selectable).join() {
                if transform.r >= select_r && transform.r <= select_r + select_w
                    && transform.c >= select_c && transform.c <= select_c + select_h {
                        select.selected = true;
                }
                else {
                    select.selected = false;
                }
            }
            *select_event = SelectEvent(None);
        }
    }
}

struct State {
    ecs: World,
    is_mining: bool,
    select_start: Option<(i32, i32)>,
}

impl State {
    fn run_systems(&mut self) {
        let mut wm = WorkManager{};
        let mut sh = SelectHandler{};
        let mut tm = time::TimeManager{};
        wm.run_now(&self.ecs);
        sh.run_now(&self.ecs);
        tm.run_now(&self.ecs);
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
        if ctx.left_click {
            self.select_start = match self.select_start {
                None => Some(ctx.mouse_pos()),
                Some((select_r, select_c)) => {
                    use std::cmp::min;
                    let (mouse_r, mouse_c) = ctx.mouse_pos();
                    let (box_r, box_c) = (min(select_r, mouse_r) ,min(select_c, mouse_c));
                    let (box_w, box_h) = ((select_r - mouse_r).abs(), (select_c - mouse_c).abs());
                    let dimensions: (u32, u32, u32, u32) = (box_r.try_into().unwrap(), box_c.try_into().unwrap(), box_w.try_into().unwrap(), box_h.try_into().unwrap());
                    *self.ecs.write_resource::<SelectEvent>() = SelectEvent(Some(dimensions));
                    None
                },
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();
        self.player_input(ctx);
        self.run_systems();

        //let rand = RandomNumberGenerator::new();
        let map = self.ecs.fetch::<Map>();

        for r in (0..).take_while(|i| i < &map.rows()) {
            for c in (0..).take_while(|i| i < &map.cols()) {
                let tile = &map.at(r,c);
                ctx.set(r,c,tile.fg, tile.bg, tile.ch);
            }
        }

        if self.is_mining {
            ctx.print_color_centered_at(SCREEN_WIDTH /2, 0,  rltk::RGB::named(rltk::RED), rltk::RGB::named(rltk::BLACK)," * Mining * ");
        }
        if let Some((select_r, select_c)) = self.select_start {
            use std::cmp::min;
            let (mouse_r, mouse_c) = ctx.mouse_pos();
            let (box_r, box_c) = (min(select_r, mouse_r) ,min(select_c, mouse_c));
            let (box_w, box_h) = ((select_r - mouse_r).abs(), (select_c - mouse_c).abs());
            ctx.draw_hollow_box(box_r, box_c, box_w, box_h, rltk::RGB::named(rltk::YELLOW), rltk::RGB::named(rltk::GRAY));
        }


        let (tran_storage, sel_storage) = (self.ecs.read_storage::<Transform>(), self.ecs.read_storage::<Selectable>());
        for (transform, selectable) in (&tran_storage, (&sel_storage).maybe()).join(){
            let (r, c) =  (transform.r , transform.c);
            let mut bg_color = map.at(r, c).bg;
            if let Some(select) = selectable {
                if select.selected {
                    bg_color = rltk::RGB::named(rltk::YELLOW);
                }
            }
            ctx.set(r, c, transform.color, bg_color, transform.ch);
        }

    }
}
const SCREEN_WIDTH: i32 = 80;
const _SCREEN_HEIGHT: i32 = 50;

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
    world.register::<Actor>();
    world.register::<Transform>();
    world.register::<Worker>();
    world.register::<Selectable>();
    world.insert(map);
    world.insert(SelectEvent(None));

    world.create_entity().with(Transform {
                    r: size_x / 2,
                    c: size_y / 2,
                    ch: '@' as u16,
                    color: rltk::RGB::named(rltk::RED)
                }).with(
                    Worker {
                        task: WorkerTask::Idle
                    },
                ).with(
                    Selectable{
                        selected: false,
                    }
                ).with(
                    Actor::new(1)
                ).build();

    let gs = State{
        ecs: world,
        is_mining: false,
        select_start: None,
    };


    rltk::main_loop(context, gs)
}
