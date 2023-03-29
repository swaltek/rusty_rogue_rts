use rltk::{Rltk, GameState};
use rltk::RandomNumberGenerator;
use specs::{World, WorldExt, Builder};
use specs::{Component, VecStorage};
use specs::Join;

use bracket_lib::prelude::*;

mod map;
use map::Map;
//use map::MapGenerator;

mod time;
use time::{Actor, ActionType};

mod input;
use input::*;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Transform{
    pub r: u32,
    pub c: u32,
    pub ch: u16,
    pub color: rltk::RGB
}


#[derive(Component)]
#[storage(VecStorage)]
pub struct Worker{
    pub task: WorkerTask,
}

use specs::{Entity, Entities};

pub enum WorkerTask{
    Idle,
    Mine(Entity),
    MoveTo(u32, u32),
}

use specs::System;
use specs::{Write, ReadExpect, WriteExpect, ReadStorage, WriteStorage};
use specs::{RunNow};

struct WorkManager;

impl<'a> System<'a> for WorkManager{
    type SystemData = ( ReadStorage<'a, Transform>,
                        ReadStorage<'a, Worker>,
                        WriteStorage<'a, Actor>);

    fn run(&mut self, data: Self::SystemData){
        let mut rand = RandomNumberGenerator::new();
        let (transforms ,worker, mut actors) = data;

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
                WorkerTask::Mine(entity) => {
                    if let Some(trans) = transforms.get(*entity){
                        if !act.is_busy() {
                            act.new_action(ActionType::MoveTo(trans.r, trans.c));
                        }
                    }
                }
                WorkerTask::MoveTo(dr, dc) => {
                    if !act.is_busy() {
                        act.new_action(ActionType::MoveTo(*dr, *dc));
                    }
                },
            }
        }
    }
}

pub struct MoveMap{
    map: Vec<Vec<bool>>,
    rows: usize,
    cols: usize,
}

struct MapManager;

impl<'a> System<'a> for MapManager{
    type SystemData = ( WriteExpect<'a, MoveMap>,
                        ReadExpect<'a, Map>,
                        ReadStorage<'a, Transform>);

    fn run(&mut self, data: Self::SystemData){
        let (mut mmap, map, transforms) = data;
        mmap.map = vec![vec![true; mmap.cols ]; mmap.rows];
        for r in (0..).take_while(|i| i < &map.rows()) {
            for c in (0..).take_while(|i| i < &map.cols()) {
                mmap.map[r as usize][c as usize] = map.at(r, c).walkable;//TODO do this with a map function
            }
        }

        for trans in (transforms).join() {
            mmap.map[trans.r as usize][trans.c as usize] = false;
        }
    }
}

struct State {
    ecs: World,
    is_mining: bool,
    draw_move_map: bool,
    select_start: Option<(i32, i32)>,
}

impl State {
    fn run_systems(&mut self) {
        input::run_systems(&self.ecs);
        let mut map_manager = MapManager{};
        let mut wm = WorkManager{};
        let mut tm = time::TimeManager{};
        map_manager.run_now(&self.ecs);
        wm.run_now(&self.ecs);
        tm.run_now(&self.ecs);
        self.ecs.maintain();
    }
    fn player_input(&mut self, ctx: &mut Rltk){
        let mut input = INPUT.lock();

        while let Some(event) = input.pop(){
            match event {
                BEvent::KeyboardInput{key, pressed, ..} => {
                    match key {
                        VirtualKeyCode::M => if pressed {self.is_mining = !self.is_mining },
                        VirtualKeyCode::D => if pressed {self.draw_move_map = !self.draw_move_map },
                        _ => {},
                    }
                },
                BEvent::MouseClick{button: 1, pressed: true} => {
                    let (mouse_r, mouse_c) = input.mouse_tile_pos(0);
                    let (r, c): (u32, u32) = (mouse_r.try_into().unwrap(), mouse_c.try_into().unwrap());
                    *self.ecs.write_resource::<MouseEvent>() = MouseEvent(MouseEventT::MoveTo(r, c));
                    for (entity, trans) in (&self.ecs.entities(),&self.ecs.read_storage::<Transform>()).join() {
                        if trans.r == r && trans.c == c {
                            *self.ecs.write_resource::<MouseEvent>() = MouseEvent(MouseEventT::Activate(entity));
                        }
                    }
                },
                BEvent::MouseClick{button: 0, ..} => {
                    self.select_start = match self.select_start {
                        None => Some(input.mouse_tile_pos(0)),
                        Some((select_r, select_c)) => {
                            use std::cmp::min;
                            let (mouse_r, mouse_c) = input.mouse_tile_pos(0);
                            let (box_r, box_c) = (min(select_r, mouse_r) ,min(select_c, mouse_c));
                            let (box_w, box_h) = ((select_r - mouse_r).abs(), (select_c - mouse_c).abs());
                            let (r, c, w, h): (u32, u32, u32, u32) = (box_r.try_into().unwrap(), box_c.try_into().unwrap(), box_w.try_into().unwrap(), box_h.try_into().unwrap());
                            *self.ecs.write_resource::<MouseEvent>() = MouseEvent(MouseEventT::BoxSelect(r, c, w, h));
                            None
                        },
                    }
                },
                BEvent::CloseRequested =>{
                    ctx.quitting = true;
                },
                _ => {},
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
        let mmap = self.ecs.fetch::<MoveMap>();

        if !self.draw_move_map{
            for r in (0..).take_while(|i| i < &map.rows()) {
                for c in (0..).take_while(|i| i < &map.cols()) {
                    let tile = &map.at(r,c);
                    ctx.set(r,c,tile.fg, tile.bg, tile.ch);
                }
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

        if self.draw_move_map{
            for r in (0..).take_while(|i| i < &mmap.rows) {
                for c in (0..).take_while(|i| i < &mmap.cols) {
                    let tile = &mmap.map[r][c];
                    ctx.set(r,c,rltk::RGB::named(rltk::YELLOW), rltk::RGB::named(rltk::BLACK), if *tile { ' ' as u16 } else { '#' as u16 } );
                }
            }
        }

    }
}
const SCREEN_WIDTH: i32 = 80;
const _SCREEN_HEIGHT: i32 = 50;

fn create_worker(ecs: &mut World, r_start: u32, c_start: u32) {
    ecs.create_entity().with(Transform {
                    r: r_start,
                    c: c_start,
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
                    Actor::new(2)
                ).build();
}

fn main() -> rltk::BError{
    use rltk::RltkBuilder;
    INPUT.lock().activate_event_queue();
    let context = RltkBuilder::simple80x50()
        .with_title("Rougelike Tutorial")
        .with_fps_cap(30.0)
        .build()?;

    let (size_x, size_y) = context.get_char_size();

    let mut world = World::new();
    world.register::<Actor>();
    world.register::<Transform>();
    world.register::<Worker>();
    world.register::<Selectable>();

    create_worker(&mut world, size_x / 2 - 3, size_y / 2);
    create_worker(&mut world, size_x / 2 + 3, size_y / 2);
    create_worker(&mut world, size_x / 2, size_y / 2);


    //let mut map_gen = MapGenerator::new(size_x, size_y);
    //map_gen.gold_count = 64;
    //map_gen.gold_size = 6;
    let map = Map::basic_80x50(&mut world);
    let rows = map.rows() as usize;
    let cols = map.cols() as usize;
    //let mut map = Map::new(size_x, size_y);

    // RESOURCES
    world.insert(MouseEvent(MouseEventT::Empty));
    world.insert(IsSomeSelected(false));
    world.insert(map);

    world.insert(MoveMap{
        map: vec![vec![true ; cols]; rows],
        rows: rows,
        cols: cols,
    });

    let gs = State{
        ecs: world,
        is_mining: false,
        draw_move_map: false,
        select_start: None,
    };


    rltk::main_loop(context, gs)
}
