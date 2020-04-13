use gdnative::{KinematicBody2D, Sprite, ResourceLoader, PackedScene};

use crate::units::Unit;

pub fn create_unit() -> Unit {
    let body = KinematicBody2D::new();
    Unit(body)
}

pub fn create_player_sprite() -> Sprite {
    let mut loader = ResourceLoader::godot_singleton();

    loader.load("res://PlayerSprite.tscn".into(), "PackedScene".into(), false)
        .and_then(|res| res.cast::<PackedScene>())
        .and_then(|scn| scn.instance(0))
        .and_then(|nod| unsafe { nod.cast::<Sprite>() })
        .unwrap()
}
