use std::cell::Cell;
use std::rc::Rc;

use {Error, Function, Lua, String};

#[test]
fn scope_func() {
    let lua = Lua::new();

    let rc = Rc::new(Cell::new(0));
    lua.scope(|scope| {
        let r = rc.clone();
        let f = scope
            .create_function(move |_, ()| {
                r.set(42);
                Ok(())
            })
            .unwrap();
        lua.globals().set("bad", f.clone()).unwrap();
        f.call::<_, ()>(()).unwrap();
        assert_eq!(Rc::strong_count(&rc), 2);
    });
    assert_eq!(rc.get(), 42);
    assert_eq!(Rc::strong_count(&rc), 1);

    match lua
        .globals()
        .get::<_, Function>("bad")
        .unwrap()
        .call::<_, ()>(())
    {
        Err(Error::CallbackError { .. }) => {}
        r => panic!("improper return for destructed function: {:?}", r),
    };
}

#[test]
fn scope_capture() {
    let lua = Lua::new();

    let mut i = 0;
    lua.scope(|scope| {
        scope
            .create_function_mut(|_, ()| {
                i = 42;
                Ok(())
            })
            .unwrap()
            .call::<_, ()>(())
            .unwrap();
    });
    assert_eq!(i, 42);
}

#[test]
fn outer_lua_access() {
    let lua = Lua::new();
    let table = lua.create_table().unwrap();
    lua.scope(|scope| {
        scope
            .create_function_mut(|_, ()| {
                table.set("a", "b").unwrap();
                Ok(())
            })
            .unwrap()
            .call::<_, ()>(())
            .unwrap();
    });
    assert_eq!(table.get::<_, String>("a").unwrap(), "b");
}
