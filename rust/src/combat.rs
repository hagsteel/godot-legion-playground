use gdnative::{Vector2, TextureRect};
use legion::prelude::*;

use crate::units::{UnitRect, UnitPos};
use crate::input::{MousePos, MouseButton};
use crate::gameworld::{Selected, WorldNode, Delta};
use crate::spawner;

const COOLDOWN: f32 = 1.;

// -----------------------------------------------------------------------------
//     - Tags -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
struct Firing;

// -----------------------------------------------------------------------------
//     - Components -
// -----------------------------------------------------------------------------
pub struct Target(Entity);

#[derive(Debug)]
pub struct Hitpoints(pub u32);

#[derive(Debug)]
pub struct Bullet(pub TextureRect);

unsafe impl Send for Bullet {}
unsafe impl Sync for Bullet {}

impl Drop for Bullet {
    fn drop(&mut self) {
        unsafe { self.0.queue_free() };
    }
}

struct Cooldown(f32);

// -----------------------------------------------------------------------------
//     - Systems -
// -----------------------------------------------------------------------------
pub fn target_unit() -> Box<dyn Schedulable> {
    SystemBuilder::new("target unit")
        .read_resource::<MousePos>()
        .write_resource::<MouseButton>()
        .with_query(<Read<UnitRect>>::query().filter(tag::<Selected>()))
        .with_query(<Read<UnitRect>>::query().filter(!tag::<Selected>()))
        .build(|cmd, world, (mouse_pos, mouse_btn), (query, target_query)| {
            if !mouse_btn.button_pressed(1) {
                return
            }

            let attackers = query
                .iter_entities(world)
                .map(|(ent, _)| ent).collect::<Vec<_>>();

            let attacker = match attackers.first() {
                None => return,
                Some(a) => a,
            };

            for (target_entity, rect) in target_query.iter_entities(world) {
                if rect.0.contains(mouse_pos.global().to_point()) {
                    // Have our target
                    cmd.add_component(*attacker, Target(target_entity));
                    mouse_btn.consume();
                    return
                }
            }
        })
}

pub fn attack_targets() -> Box<dyn Schedulable> {
    SystemBuilder::new("attack targets")
        .write_component::<Hitpoints>()
        .with_query(<Read<Target>>::query().filter(!component::<Cooldown>()))
        .build(|cmd, world, _, query| {
            let targets = query.iter_entities(world).map(|(entity, target)| {
                (entity, target.0)
            }).collect::<Vec<_>>();

            for (entity, target_ent) in targets {
                match world.get_component_mut::<Hitpoints>(target_ent) {
                    None => { /* how can there be a unit without hitpoints? */ }
                    Some(mut hp) => {
                        cmd.add_tag(entity, Firing);
                        cmd.add_component(entity, Cooldown(COOLDOWN));
                        hp.0 -= 1;

                        if hp.0 <= 0 {
                            cmd.remove_component::<Target>(entity);
                            cmd.delete(target_ent);
                        }
                    }
                }
            }
        })
}

pub fn cooldown_units() -> Box<dyn Schedulable> {
    SystemBuilder::new("cooldown")
        .read_resource::<Delta>()
        .with_query(<Write<Cooldown>>::query())
        .build(|cmd, world, delta, query| {

            for (entity, mut cooldown) in query.iter_entities_mut(world) {
                cooldown.0 -= delta.0;

                if cooldown.0 <= 0. {
                    cmd.remove_component::<Cooldown>(entity);
                }
            }
        })
}

pub fn spawn_bullets() -> Box<dyn Runnable> {
    SystemBuilder::new("spawn bullets")
        .write_resource::<WorldNode>()
        .read_component::<UnitPos>()
        .with_query(<(Read<UnitPos>, Read<Target>)>::query().filter(tag::<Firing>()))
        .build_thread_local(|cmd, world, world_node, query| {
            for (entity, (attacker_pos, target)) in query.iter_entities(world) {
                let target_pos = match world.get_component::<UnitPos>(target.0) {
                    None => continue,
                    Some(pos) => pos
                };

                // Create bullet
                let mut bullet_tex = spawner::create_bullet(2);

                // Add bullet to scene tree
                unsafe { world_node.0.add_child(Some(bullet_tex.to_node()), false) };

                // Position and scale bullet
            
                cmd.remove_tag::<Firing>(entity);

                unsafe {
                    bullet_tex.set_global_position(attacker_pos.0, false);
                    let direction = (target_pos.0 - attacker_pos.0).normalize();
                    let distance = (target_pos.0 - attacker_pos.0).length();

                    let scale = Vector2::new(distance, 1.);
                    let rot = direction.y.atan2(direction.x);

                    bullet_tex.set_rotation(rot as f64);
                    bullet_tex.set_size(scale, false);

                    cmd.insert(
                        (),
                        vec![(Bullet(bullet_tex), )]
                    );
                }
            }
        })
}

pub fn despawn_bullets() -> Box<dyn Runnable> {
    SystemBuilder::new("despawn bullets")
        .read_resource::<Delta>()
        .with_query(<Write<Bullet>>::query())
        .build_thread_local(|cmd, world, delta, query| {

            for (entity, mut bullet) in query.iter_entities_mut(world) {
                unsafe {
                    let mut modulate = bullet.0.get_modulate();
                    modulate.a -= delta.0 * 4.;
                    bullet.0.set_modulate(modulate);

                    if modulate.a <= 0. {
                        cmd.delete(entity);
                    }
                }
            }
        })
}

#[cfg(feature = "godot_test")]
pub mod tests {
    use crate::assert_gd;
    use gdnative::{Vector2, Rect2};
    use crate::Size2;
    use super::*;

    // Unit should be marked as selected
    pub fn test_target_unit() -> bool {
        let mut world = Universe::new().create_world();
        let mut resources = Resources::default();
        let target_pos = Vector2::new(100., 100.);
        let mut mouse_pos = MousePos::zero();
        mouse_pos.set_global(target_pos);
        resources.insert(mouse_pos);
        resources.insert(MouseButton::Mouse { pressed: true, button_index: 1 });

        let entity = world.insert((Selected,), vec![(
                UnitRect(Rect2::new(Vector2::zero().to_point(), Size2::new(10., 10.,))),
        ),])[0];

        let target_entity = world.insert((), vec![(
                UnitRect(Rect2::new(target_pos.to_point(), Size2::new(10., 10.,))),
        ),])[0];

        let mut sched = Schedule::builder()
            .add_system(target_unit())
            .flush()
            .build();

        sched.execute(&mut world, &mut resources);

        assert_gd!(world.get_component::<Target>(entity).is_some())
    }

    pub fn test_attack_target() -> bool {
        let mut world = Universe::new().create_world();
        let mut resources = Resources::default();
        let target_pos = Vector2::new(100., 100.);
        let mut mouse_pos = MousePos::zero();
        mouse_pos.set_global(target_pos);
        resources.insert(mouse_pos);
        resources.insert(MouseButton::Mouse { pressed: true, button_index: 1 });

        let target_entity = world.insert((), vec![(
                Hitpoints(10),
        ),])[0];

        let entity = world.insert((), vec![(
                Target(target_entity), Hitpoints(10),
        ),])[0];

        let mut sched = Schedule::builder()
            .add_system(attack_targets())
            .flush()
            .build();

        sched.execute(&mut world, &mut resources);

        let hitpoints = world.get_component::<Hitpoints>(target_entity).unwrap();
        assert_gd!(hitpoints.0 == 9)
    }
}

