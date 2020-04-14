use gdextras::movement::Move2D;
use gdnative::{KinematicBody2D, Rect2, Vector2};
use legion::prelude::*;

use crate::gameworld::{Selected, WorldNode};
use crate::input::{MouseButton, MousePos};
use crate::spawner::{create_player_sprite, create_unit};
use crate::Size2;
use crate::combat::Hitpoints;

pub struct Unit(pub KinematicBody2D);

impl Drop for Unit {
    fn drop(&mut self) {
        unsafe { self.0.queue_free() };
    }
}

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
                let hitpoints = Hitpoints(10);
                cmd.insert((), vec![(unit, unit_pos, unit_rect, hitpoints)]);
            };
        })
}

pub fn select_unit() -> Box<dyn Schedulable> {
    SystemBuilder::new("select unit")
        .write_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .with_query(<Read<UnitRect>>::query())
        .with_query(<Read<UnitRect>>::query().filter(tag::<Selected>()))
        .build(|cmd, world, (mouse_btn, mouse_pos), (deselected_query, selected_query)| {
            // Only one selected unit at a time
            if selected_query.iter(world).count() > 0 {
                return;
            }

            if !mouse_btn.button_pressed(1) {
                return;
            }

            for (entity, rect) in deselected_query.iter_entities(world) {
                if rect.0.contains(mouse_pos.global().to_point()) {
                    cmd.add_tag(entity, Selected);
                    mouse_btn.consume();
                    return;
                }
            }
        })
}

pub fn set_unit_destination() -> Box<dyn Schedulable> {
    SystemBuilder::new("give units a destination")
        .write_resource::<MouseButton>()
        .read_resource::<MousePos>()
        .with_query(<Read<UnitRect>>::query())
        .with_query(<Read<UnitRect>>::query().filter(tag::<Selected>()))
        .build(|cmd, world, (mouse_btn, mouse_pos), (all_query, query)| {
            if !mouse_btn.button_pressed(1) {
                return;
            }

            for unit_rect in all_query.iter(world) {
                if unit_rect.0.contains(mouse_pos.global().to_point()) {
                    return
                }
            }

            for (entity, _) in query.iter_entities(world) {
                cmd.add_component(entity, Destination(mouse_pos.global()));
                cmd.remove_tag::<Selected>(entity);
            }

            mouse_btn.consume();
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

#[cfg(feature = "godot_test")]
pub mod tests {
    use crate::assert_gd;
    use super::*;

    // Unit should be marked as selected
    pub fn test_move_units() -> bool {
        let mut world = Universe::new().create_world();
        let mut resources = Resources::default();
        resources.insert(MousePos::zero());
        resources.insert(MouseButton::Mouse { pressed: true, button_index: 1 });

        let entity = world.insert((), vec![(
                UnitRect(Rect2::new(Vector2::zero().to_point(), Size2::new(10., 10.,))),
        ),])[0];

        assert_gd!(world.get_tag::<Selected>(entity).is_none());

        let mut sched = Schedule::builder()
            .add_system(select_unit())
            .build();

        sched.execute(&mut world, &mut resources);

        assert_gd!(world.get_tag::<Selected>(entity).is_some())
    }
}
