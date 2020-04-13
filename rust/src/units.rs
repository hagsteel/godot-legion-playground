use euclid::Size2D;
use gdextras::movement::Move2D;
use gdnative::{KinematicBody2D, Point2, Rect2, Vector2};
use legion::prelude::*;

use crate::gameworld::{Selected, WorldNode};
use crate::input::{MouseButton, MousePos};
use crate::spawner::{create_player_sprite, create_unit};

pub type Size2 = Size2D<f32, euclid::UnknownUnit>;

pub struct Unit(pub KinematicBody2D);

unsafe impl Send for Unit {}
unsafe impl Sync for Unit {}

pub struct UnitPos(pub Vector2);

pub struct UnitRect(pub Rect2);

impl UnitRect {
    pub fn new(pos: Vector2, width: f32, height: f32) -> Self {
        let origin = Vector2::new(pos.x - width / 2., pos.y - height / 2.);

        let rect = Rect2::new(origin.to_point(), Size2::new(width, height));
        Self(rect)
    }

    pub fn update(&mut self, pos: Vector2) {
        self.0.origin = Vector2::new(
            pos.x - self.0.size.width / 2.,
            pos.y - self.0.size.height / 2.,
        ).to_point();
    }
}

pub struct Destination(pub Vector2);

pub fn spawn_unit() -> Box<dyn Runnable> {
    SystemBuilder::new("spaw unit")
        .write_resource::<WorldNode>()
        .write_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .build_thread_local(|cmd, _world, (world_node, mouse_btn, mouse_pos), _query| {
            if !mouse_btn.button_pressed(2) {
                return;
            }

            mouse_btn.consume();

            let mut unit = create_unit();
            let sprite = create_player_sprite();

            unsafe {
                unit.0.add_child(Some(sprite.to_node()), false);
                world_node.add_child(unit.0.to_node());
                unit.0.set_global_position(mouse_pos.global());

                let unit_pos = UnitPos(unit.0.get_global_position());
                let unit_rect = UnitRect::new(unit_pos.0, 7., 29.);
                cmd.insert((), vec![(unit, unit_pos, unit_rect)]);
            };
        })
}

pub fn select_unit() -> Box<dyn Schedulable> {
    SystemBuilder::new("select unit")
        .write_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .with_query(<Read<UnitRect>>::query())
        .build(|cmd, world, (mouse_btn, mouse_pos), query| {
            if !mouse_btn.button_pressed(1) {
                return;
            }

            eprintln!("{:?}", "try to select unit");

            for (entity, rect) in query.iter_entities(world) {
                if rect.0.contains(mouse_pos.global().to_point()) {
                    cmd.add_tag(entity, Selected);
                    mouse_btn.consume();
                    eprintln!("{:?}", "select unit");
                    return;
                }
            }
        })
}

pub fn set_unit_destination() -> Box<dyn Schedulable> {
    SystemBuilder::new("give units a destination")
        .write_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .with_query(<Read<UnitPos>>::query().filter(tag::<Selected>()))
        .build(|cmd, world, (mouse_btn, mouse_pos), query| {
            if !mouse_btn.button_pressed(1) {
                return;
            }

            mouse_btn.consume();

            for (entity, _) in query.iter_entities(world) {
                cmd.add_component(entity, Destination(mouse_pos.global()));
                cmd.remove_tag::<Selected>(entity);
            }
        })
}

pub fn move_units() -> Box<dyn Runnable> {
    SystemBuilder::new("move units")
        .with_query(<(
            Write<Unit>,
            Write<UnitPos>,
            Write<UnitRect>,
            Read<Destination>,
        )>::query())
        .build_thread_local(|cmd, world, _, query| {
            for (entity, (mut unit, mut unit_pos, mut unit_rect, dest)) in
                query.iter_entities_mut(world)
            {
                let direction = (dest.0 - unit_pos.0).normalize();
                let speed = 100f32;
                let velocity = direction * speed;

                unit.0.move_and_slide_default(velocity, Vector2::zero());
                unsafe { unit_pos.0 = unit.0.get_global_position() };
                unit_rect.update(unit_pos.0);

                if (dest.0 - unit_pos.0).length() < 4. {
                    cmd.remove_component::<Destination>(entity);
                }
            }
        })
}
