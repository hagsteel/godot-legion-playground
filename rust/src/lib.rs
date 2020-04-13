use gdnative::*;

mod gameworld;
mod units;
mod spawner;
mod input;

fn init(handle: init::InitHandle) {
    handle.add_class::<gameworld::GameWorld>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();


#[no_mangle]
pub extern fn run_tests() -> sys::godot_variant {
    let status = true;

    eprintln!("Running tests: [add your tests here]");

    gdnative::Variant::from_bool(status).forget()
}
