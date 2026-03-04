use bevy_ecs::{prelude::*, system::ScheduleSystem};

pub struct AetherCore {
    pub world: World,
    pub schedule: Schedule,
}

impl AetherCore {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            schedule: Schedule::default(),
        }
    }
    pub fn add_systems<M>(&mut self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) {
        self.schedule.add_systems(systems);
    }

    pub fn tick(&mut self) {
        self.schedule.run(&mut self.world);
    }
}

#[derive(Component, Default)]
pub struct Transform {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Component, Default)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Bundle)]
pub struct TestBundle {
    pub transform: Transform,
    pub velocity: Velocity,
}
