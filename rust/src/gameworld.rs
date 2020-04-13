use gdextras::input::InputEventExt;
use gdnative::{
    godot_error, godot_wrap_method, godot_wrap_method_inner, godot_wrap_method_parameter_count,
    methods, InputEvent, InputEventMouseButton, NativeClass, Node, Node2D,
};
use lazy_static::lazy_static;
use legion::prelude::*;
use std::sync::Mutex;

use crate::input::{MouseButton, MousePos};
use crate::units::{move_units, spawn_unit, select_unit, set_unit_destination};

// -----------------------------------------------------------------------------
//     - World  -
// -----------------------------------------------------------------------------
lazy_static! {
    static ref WORLD: Mutex<World> = Mutex::new(Universe::new().create_world());
}

pub fn with_world<F>(mut f: F)
where
    F: FnMut(&mut World),
{
    let _ = WORLD.try_lock().map(|mut world| f(&mut world));
}

// -----------------------------------------------------------------------------
//     - Schedules -
// -----------------------------------------------------------------------------
struct Process {
    resources: Resources,
    schedule: Schedule,
}

impl Process {
    fn new() -> Self {
        let mut resources = Resources::default();
        resources.insert(Delta(0.));
        resources.insert(MousePos::zero());
        resources.insert(MouseButton::Empty);

        let schedule = Schedule::builder()
            .add_system(select_unit())
            .add_system(set_unit_destination())
            .add_thread_local(spawn_unit())
            .build();

        Self {
            resources,
            schedule,
        }
    }

    fn execute(&mut self, delta: f64) {
        self.resources
            .get_mut::<Delta>()
            .map(|mut d| d.0 = delta as f32);

        with_world(|mut world| {
            self.schedule.execute(&mut world, &mut self.resources);
        })
    }
}

struct Physics {
    resources: Resources,
    schedule: Schedule,
}

impl Physics {
    fn new() -> Self {
        let mut resources = Resources::default();
        resources.insert(Delta(0.));

        let schedule = Schedule::builder()
            .add_thread_local(move_units())
            .build();

        Self {
            resources,
            schedule,
        }
    }

    fn execute(&mut self, delta: f64) {
        self.resources
            .get_mut::<Delta>()
            .map(|mut d| d.0 = delta as f32);
        with_world(|mut world| {
            self.schedule.execute(&mut world, &mut self.resources);
        })
    }
}

// -----------------------------------------------------------------------------
//     - Resources -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Delta(pub f32);

pub struct WorldNode(pub Node2D);

impl WorldNode {
    pub unsafe fn add_child(&mut self, node: Node) {
        self.0.add_child(Some(node), false);
    }
}

unsafe impl Send for WorldNode {}
unsafe impl Sync for WorldNode {}

// -----------------------------------------------------------------------------
//     - Tags -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct Selected;

// -----------------------------------------------------------------------------
//     - Godot node -
// -----------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct GameWorld {
    process: Process,
    physics: Physics,
}

#[methods]
impl GameWorld {
    pub fn _init(_owner: Node2D) -> Self {
        Self {
            process: Process::new(),
            physics: Physics::new(),
        }
    }

    #[export]
    pub fn _ready(&mut self, owner: Node2D) {
        self.process.resources.insert(WorldNode(owner));
    }

    #[export]
    pub fn _unhandled_input(&self, owner: Node2D, event: InputEvent) {
        if event.action_pressed("ui_cancel") {
            unsafe { owner.get_tree().map(|mut tree| tree.quit(0)) };
        }

        // Mouse position
        self.process
            .resources
            .get_mut::<MousePos>()
            .map(|mut mouse| {
                mouse.set_global(unsafe { owner.get_global_mouse_position() });
            });

        // Mouse button event
        if let Some(ev) = event.cast::<InputEventMouseButton>() {
            self.process
                .resources
                .get_mut::<MouseButton>()
                .map(|mut mouse| {
                    *mouse = MouseButton::from_event(ev);
                });
        }
    }

    #[export]
    pub fn _process(&mut self, _: Node2D, delta: f64) {
        self.process.execute(delta);
    }

    #[export]
    pub fn _physics_process(&mut self, _: Node2D, delta: f64) {
        self.physics.execute(delta);
    }
}
