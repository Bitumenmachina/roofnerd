// Windows would otherwise open a console behind the application.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    roofnerd_lib::run()
}
