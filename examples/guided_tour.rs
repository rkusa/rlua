extern crate rlua;

use std::iter::FromIterator;

use rlua::{Function, Lua, Result, Variadic};

fn main() -> Result<()> {
    // Create a Lua context with `Lua::new()`.  Eventually, this will allow further control on the
    // lua std library, and will specifically allow limiting Lua to a subset of "safe"
    // functionality.

    let lua = Lua::new();

    // You can get and set global variables.  Notice that the globals table here is a permanent
    // reference to _G, and it is mutated behind the scenes as lua code is loaded.  This API is
    // based heavily around internal mutation (just like lua itself).

    let globals = lua.globals();

    globals.set("string_var", "hello")?;
    globals.set("int_var", 42)?;

    assert_eq!(globals.get::<_, String>("string_var")?, "hello");
    assert_eq!(globals.get::<_, i64>("int_var")?, 42);

    // You can load and evaluate lua code.  The second parameter here gives the chunk a better name
    // when lua error messages are printed.

    lua.exec::<()>(
        r#"
            global = 'foo'..'bar'
        "#,
        Some("example code"),
    )?;
    assert_eq!(globals.get::<_, String>("global")?, "foobar");

    assert_eq!(lua.eval::<i32>("1 + 1", None)?, 2);
    assert_eq!(lua.eval::<bool>("false == false", None)?, true);
    assert_eq!(lua.eval::<i32>("return 1 + 2", None)?, 3);

    // You can create and manage lua tables

    let array_table = lua.create_table()?;
    array_table.set(1, "one")?;
    array_table.set(2, "two")?;
    array_table.set(3, "three")?;
    assert_eq!(array_table.len()?, 3);

    let map_table = lua.create_table()?;
    map_table.set("one", 1)?;
    map_table.set("two", 2)?;
    map_table.set("three", 3)?;
    let v: i64 = map_table.get("two")?;
    assert_eq!(v, 2);

    // You can pass values like `Table` back into Lua

    globals.set("array_table", array_table)?;
    globals.set("map_table", map_table)?;

    lua.eval::<()>(
        r#"
        for k, v in pairs(array_table) do
            print(k, v)
        end

        for k, v in pairs(map_table) do
            print(k, v)
        end
        "#,
        None,
    )?;

    // You can load lua functions

    let print: Function = globals.get("print")?;
    print.call::<_, ()>("hello from rust")?;

    // This API generally handles variadics using tuples.  This is one way to call a function with
    // multiple parameters:

    print.call::<_, ()>(("hello", "again", "from", "rust"))?;

    // But, you can also pass variadic arguments with the `Variadic` type.

    print.call::<_, ()>(Variadic::from_iter(
        ["hello", "yet", "again", "from", "rust"].iter().cloned(),
    ))?;

    // You can bind rust functions to lua as well.  Callbacks receive the Lua state itself as their
    // first parameter, and the arguments given to the function as the second parameter.  The type
    // of the arguments can be anything that is convertible from the parameters given by Lua, in
    // this case, the function expects two string sequences.

    let check_equal = lua.create_function(|_, (list1, list2): (Vec<String>, Vec<String>)| {
        // This function just checks whether two string lists are equal, and in an inefficient way.
        // Lua callbacks return `rlua::Result`, an Ok value is a normal return, and an Err return
        // turns into a Lua 'error'.  Again, any type that is convertible to lua may be returned.
        Ok(list1 == list2)
    })?;
    globals.set("check_equal", check_equal)?;

    // You can also accept runtime variadic arguments to rust callbacks.

    let join = lua.create_function(|_, strings: Variadic<String>| {
        // (This is quadratic!, it's just an example!)
        Ok(strings.iter().fold("".to_owned(), |a, b| a + b))
    })?;
    globals.set("join", join)?;

    assert_eq!(
        lua.eval::<bool>(r#"check_equal({"a", "b", "c"}, {"a", "b", "c"})"#, None)?,
        true
    );
    assert_eq!(
        lua.eval::<bool>(r#"check_equal({"a", "b", "c"}, {"d", "e", "f"})"#, None)?,
        false
    );
    assert_eq!(lua.eval::<String>(r#"join("a", "b", "c")"#, None)?, "abc");

    // Normally, Rust types passed to `Lua` must be `Send`, because `Lua` itself is `Send`, and must
    // be `'static`, because there is no way to tell when Lua might garbage collect them.  There is,
    // however, a limited way to lift both of these restrictions.  You can call `Lua::scope` to
    // create userdata types that do not have to be `Send`, and callback types that do not have to
    // be `Send` OR `'static`.

    let mut rust_val = 0;

    lua.scope(|scope| {
        // We create a 'sketchy' lua callback that modifies the variable `rust_val`.  Outside of a
        // `Lua::scope` call, this would not be allowed.

        lua.globals().set(
            "sketchy",
            scope.create_function_mut(|_, ()| {
                rust_val = 42;
                Ok(())
            })?,
        )?;

        lua.eval::<()>("sketchy()", None)
    })?;

    // We were able to run our 'sketchy' function inside the scope just fine.  However, if we try to
    // run our 'sketchy' function outside of the scope, the function we created will have been
    // destroyed and we will generate an error.

    assert_eq!(rust_val, 42);
    assert!(lua.eval::<()>("sketchy()", None).is_err());

    Ok(())
}
