extern crate rlua;

use std::panic::catch_unwind;

use rlua::Lua;

fn main() {
    let lua = Lua::new();
    let table = lua.create_table().unwrap();
    let _ = catch_unwind(move || table.set("a", "b").unwrap());
    //~^ error: the type `std::cell::UnsafeCell<()>` may contain interior mutability and a reference
    // may not be safely transferrable across a catch_unwind boundary
}
