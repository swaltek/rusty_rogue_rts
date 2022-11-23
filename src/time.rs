use specs::{Component, VecStorage};

use specs::System;
use specs::{ReadExpect, WriteStorage};
use specs::Join;

use super::{Transform, Map};

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

    pub fn new_action(&mut self, action: ActionType) -> bool {
        match self.action {
            Some(_) => false,
            None => {
                self.action = Some(Action{
                    start_time: time::Instant::now(),
                    execution_time: time::Duration::from_secs(self.speed.try_into().unwrap()),
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
}

pub struct TimeManager;
impl<'a> System<'a> for TimeManager{
    type SystemData = (
            ReadExpect<'a, Map>,
            WriteStorage<'a, Actor>,
            WriteStorage<'a, Transform>,
        );

    fn run(&mut self, data: Self::SystemData){
        let (map, mut actors, mut transforms) = data;
        let now = time::Instant::now();

        for (actor, transform) in (&mut actors, &mut transforms).join() {
            if let Some(action) = &actor.action {
                if let ActionType::Move(dr, dc) = action.t {
                    if now > ( action.start_time + action.execution_time) {
                        fn add( u: u32, i: i32) -> u32{
                            let abs_i: u32 = i.abs().try_into().unwrap();
                            match i {
                                _ if i < 0 => u - abs_i,
                                _ if i > 0 => u + abs_i,
                                _ => u,
                            }
                        }
                        let (new_r, new_c) = (add(transform.r, dr), add(transform.c, dc));
                        if map.at(new_r, new_c).walkable {
                            transform.r = new_r;
                            transform.c = new_c;
                        }
                        actor.action = None;
                    }
                }
            }
        }
    }
}
