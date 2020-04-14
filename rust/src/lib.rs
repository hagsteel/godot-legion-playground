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


#[cfg(feature = "godot_test")]
macro_rules! run_test {
    ($test:expr) => {

        if $test() {
            println!("{} [Ok]", stringify!($test));
            true
        } else {
            println!("{} [Failed]", stringify!($test));
            false
        }
    }
}

#[cfg(feature = "godot_test")]
#[macro_export]
macro_rules! assert_gd {
    ($assert_exp:expr) => {
        if !$assert_exp {
            let line = std::line!();
            let file = std::file!();
            eprintln!("{}: {}", file, line);
            return false
        } else {
            true
        }
    }
}

#[no_mangle]
#[cfg(feature = "godot_test")]
pub extern fn run_tests() -> sys::godot_variant {
    let mut status = true;

    eprintln!("Running tests");
    status &= run_test!(units::tests::test_move_units);

    gdnative::Variant::from_bool(status).forget()
}
