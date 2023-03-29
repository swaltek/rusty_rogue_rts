use specs::{Component, VecStorage};

use specs::System;
use specs::{ReadExpect, WriteStorage};
use specs::Join;

use super::{Transform, MoveMap};

use std::time as time;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Actor {
    speed: i32,
    action: Option<Action>,
}

impl Actor {
    pub fn new(speed: i32) -> Self{
        Self {
            speed: speed,
            action: None,
        }
    }

    pub fn get_execution_time(&self) -> time::Duration { time::Duration::from_millis((1000 / self.speed).try_into().unwrap()) }
    pub fn new_action(&mut self, action: ActionType) -> bool {
        match self.action {
            Some(_) => false,
            None => {
                self.action = Some(Action{
                    start_time: time::Instant::now(),
                    execution_time: self.get_execution_time(),
                    t: action,
                });
                true
            },
        }
    }

    pub fn is_busy(&self) -> bool {
        self.action.is_some()
    }
}

struct Action {
    start_time: time::Instant,
    execution_time: time::Duration,
    t: ActionType,
}

pub enum ActionType {
    Move(i32, i32),
    MoveTo(u32, u32),
}

fn add( u: u32, i: i32) -> u32{
    let abs_i: u32 = i.abs().try_into().unwrap();
    match i {
        _ if i < 0 => u - abs_i,
        _ if i > 0 => u + abs_i,
        _ => u,
    }
}
fn normalize(i: i32) -> i32 {
    match i {
        _ if i < 0 => -1,
        _ if i > 0 => 1,
        _ => 0,
    }
}

pub struct TimeManager;
impl<'a> System<'a> for TimeManager{
    type SystemData = (
            ReadExpect<'a, MoveMap>,
            WriteStorage<'a, Actor>,
            WriteStorage<'a, Transform>,
        );

    fn run(&mut self, data: Self::SystemData){
        let (mmap, mut actors, mut transforms) = data;
        let now = time::Instant::now();

        for (actor, transform) in (&mut actors, &mut transforms).join() {
            if let Some(action) = &mut actor.action {
                if now > ( action.start_time + action.execution_time) {
                    match action.t {
                        ActionType::Move(dr, dc) => {
                            let (new_r, new_c) = (add(transform.r, dr), add(transform.c, dc));
                            if mmap.map[new_r as usize][new_c as usize] {
                                transform.r = new_r;
                                transform.c = new_c;
                            }
                            actor.action = None;
                        },
                        ActionType::MoveTo(dr, dc) => {
                            let (diff_r, diff_c) = (dr as i32 - transform.r as i32, dc as i32 - transform.c as i32);
                            let (change_r, change_c) = ( normalize(diff_r), normalize(diff_c));
                            let (next_r, next_c) = (add(transform.r, change_r), add(transform.c, change_c));

                            if mmap.map[next_r as usize][next_c as usize] {
                                transform.r = next_r;
                                transform.c = next_c;
                                if next_r == dr || next_c == dc {
                                    actor.action = None;
                                } else {
                                    action.start_time = now;
                                }
                            } else {
                                actor.action = None;
                            }

                        },
                    }
                }
            }
        }
    }
}
