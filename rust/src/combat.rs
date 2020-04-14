use legion::prelude::*;

use crate::units::{UnitPos, UnitRect};
use crate::input::{MousePos, MouseButton};
use crate::gameworld::Selected;

// -----------------------------------------------------------------------------
//     - Components -
// -----------------------------------------------------------------------------
pub struct Target(Entity);

#[derive(Debug)]
pub struct Hitpoints(pub u32);

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
        .with_query(<Write<Target>>::query())
        .build(|cmd, world, _, query| {
            for (entity, target) in query.iter_entities_mut(world) {
                let target_ent = target.0;
                match world.get_component_mut::<Hitpoints>(target_ent) {
                    None => { /* how can there be a unit without hitpoints? */ }
                    Some(mut hp) => {
                        hp.0 -= 1;
                        eprintln!("hp: {}", hp.0);

                        if hp.0 <= 0 {
                            cmd.remove_component::<Target>(entity);
                            cmd.delete(target_ent);
                        }
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
}

