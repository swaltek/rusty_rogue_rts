use specs::System;
use specs::{Read, Write, ReadStorage, WriteStorage};
use specs::{Component, VecStorage};
use specs::{Entity};
use specs::{Join};

use super::{Transform, Worker, WorkerTask};

use specs::{World, WorldExt};
use specs::{RunNow};

pub fn run_systems(ecs: &World) {
    let mut mh = MouseHandler{};
    let mut wih = WorkerInputHandler{};
    mh.run_now(&ecs);
    wih.run_now(&ecs);
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Selectable{
    pub selected: bool,
}

#[derive(Default)]
pub struct MouseEvent(pub MouseEventT);
#[derive(Default, Debug)]
pub enum MouseEventT{
    #[default]
    Empty,
    BoxSelect(u32, u32, u32, u32),// r, c, w, h
    MoveTo(u32, u32),// r, c
    Activate(Entity),// r, c
}

#[derive(Default)]
pub struct IsSomeSelected(pub bool);
pub struct MouseHandler;

impl<'a> System<'a> for MouseHandler{
    type SystemData = ( Write<'a, MouseEvent>,
                        Write<'a, IsSomeSelected>,
                        ReadStorage<'a, Transform>,
                        WriteStorage<'a, Selectable>);

    fn run(&mut self, data: Self::SystemData){
        let (mut mouse_event,mut some_selected, trans,mut selectable) = data;
        let MouseEvent(event) = &*mouse_event;

        match *event {
            MouseEventT::BoxSelect(select_r, select_c, select_w, select_h) => {
                *some_selected = IsSomeSelected(false);
                for (transform, select) in (&trans, &mut selectable).join() {
                    if transform.r >= select_r && transform.r <= select_r + select_w
                        && transform.c >= select_c && transform.c <= select_c + select_h {
                            select.selected = true;
                            *some_selected = IsSomeSelected(true);
                    }
                    else {
                        select.selected = false;
                    }
                }
                *mouse_event = MouseEvent(MouseEventT::Empty);
            },
            _ => {},
        }
    }
}

pub struct WorkerInputHandler;
impl<'a> System<'a> for WorkerInputHandler{
    type SystemData = ( Write<'a, MouseEvent>,
                        Read<'a, IsSomeSelected>,
                        WriteStorage<'a, Worker>,
                        ReadStorage<'a, Selectable>);

    fn run(&mut self, data: Self::SystemData){
        let (mut mouse_event, some_selected,mut workers, selectable) = data;
        if let IsSomeSelected(false) = *some_selected {
            return;
        }
        let MouseEvent(event) = &*mouse_event;

        for (worker, select) in (&mut workers, &selectable).join() {
            if select.selected {
                match *event {
                    MouseEventT::Activate(entity) => {
                        worker.task = WorkerTask::Mine(entity);
                    },
                    MouseEventT::MoveTo(r, c) => {
                        worker.task = WorkerTask::MoveTo(r, c);
                    },
                    _ => {},
                }
            }
        }
    }
}
