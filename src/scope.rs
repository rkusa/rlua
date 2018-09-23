use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;

use error::{Error, Result};
use ffi;
use function::Function;
use lua::Lua;
use types::Callback;
use util::{assert_stack, take_userdata, StackGuard};
use value::{FromLuaMulti, ToLuaMulti};

/// Constructed by the [`Lua::scope`] method, allows temporarily passing to Lua userdata that is
/// !Send, and callbacks that are !Send and not 'static.
///
/// See [`Lua::scope`] for more details.
///
/// [`Lua::scope`]: struct.Lua.html#method.scope
pub struct Scope<'scope> {
    lua: &'scope Lua,
    destructors: RefCell<Vec<Box<Fn() -> Box<Any> + 'scope>>>,
    // 'scope lifetime must be invariant
    _scope: PhantomData<&'scope mut &'scope ()>,
}

impl<'scope> Scope<'scope> {
    pub(crate) fn new(lua: &'scope Lua) -> Scope {
        Scope {
            lua,
            destructors: RefCell::new(Vec::new()),
            _scope: PhantomData,
        }
    }

    /// Wraps a Rust function or closure, creating a callable Lua function handle to it.
    ///
    /// This is a version of [`Lua::create_function`] that creates a callback which expires on scope
    /// drop.  See [`Lua::scope`] for more details.
    ///
    /// [`Lua::create_function`]: struct.Lua.html#method.create_function
    /// [`Lua::scope`]: struct.Lua.html#method.scope
    pub fn create_function<'lua, A, R, F>(&'lua self, func: F) -> Result<Function<'lua>>
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'scope + Fn(&'lua Lua, A) -> Result<R>,
    {
        unsafe {
            let f = Box::new(move |lua, args| {
                func(lua, A::from_lua_multi(args, lua)?)?.to_lua_multi(lua)
            });
            let f = mem::transmute::<Callback<'lua, 'scope>, Callback<'lua, 'static>>(f);
            let f = self.lua.create_callback(f)?;

            let mut destructors = self.destructors.borrow_mut();
            let f_destruct = f.0.clone();
            destructors.push(Box::new(move || {
                let state = f_destruct.lua.state;
                let _sg = StackGuard::new(state);
                assert_stack(state, 2);
                f_destruct.lua.push_ref(&f_destruct);

                ffi::lua_getupvalue(state, -1, 1);
                let ud = take_userdata::<Callback>(state);

                ffi::lua_pushnil(state);
                ffi::lua_setupvalue(state, -2, 1);

                ffi::lua_pop(state, 1);
                Box::new(ud)
            }));
            Ok(f)
        }
    }

    /// Wraps a Rust mutable closure, creating a callable Lua function handle to it.
    ///
    /// This is a version of [`Lua::create_function_mut`] that creates a callback which expires on
    /// scope drop.  See [`Lua::scope`] and [`Scope::create_function`] for more details.
    ///
    /// [`Lua::create_function_mut`]: struct.Lua.html#method.create_function_mut
    /// [`Lua::scope`]: struct.Lua.html#method.scope
    /// [`Scope::create_function`]: #method.create_function
    pub fn create_function_mut<'lua, A, R, F>(&'lua self, func: F) -> Result<Function<'lua>>
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'scope + FnMut(&'lua Lua, A) -> Result<R>,
    {
        let func = RefCell::new(func);
        self.create_function(move |lua, args| {
            (&mut *func
                .try_borrow_mut()
                .map_err(|_| Error::RecursiveMutCallback)?)(lua, args)
        })
    }
}

impl<'scope> Drop for Scope<'scope> {
    fn drop(&mut self) {
        // We separate the action of invalidating the userdata in Lua and actually dropping the
        // userdata type into two phases.  This is so that, in the event a userdata drop panics, we
        // can be sure that all of the userdata in Lua is actually invalidated.

        let to_drop = self
            .destructors
            .get_mut()
            .drain(..)
            .map(|destructor| destructor())
            .collect::<Vec<_>>();
        drop(to_drop);
    }
}
