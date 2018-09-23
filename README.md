# rlua51 -- High level bindings between Rust and Lua 5.1

This is a port from Lua5.3-based [rlua](https://github.com/kyren/rlua) to Lua5.1.

Changes:

- Added Lua 5.1 source and [lua-compat-5.3](https://github.com/keplerproject/lua-compat-5.3)
- Removed UserData because of [its limited compatibility to Lua 5.1](https://github.com/keplerproject/lua-compat-5.3/wiki/lua_setuservalue)

Tests are not yet green:  [![Build Status](https://travis-ci.org/kyren/rlua.svg?branch=master)](https://travis-ci.org/kyren/rlua)

---

[![Build Status](https://travis-ci.org/rkusa/rlua.svg?branch=master)](https://travis-ci.org/rkusa/rlua)

<!--
[![Latest Version](https://img.shields.io/crates/v/rlua.svg)](https://crates.io/crates/rlua)
[![API Documentation](https://docs.rs/rlua/badge.svg)](https://docs.rs/rlua)
-->

Original Readme:

---

[Guided Tour](examples/guided_tour.rs)

This library is a high level interface between Rust and Lua.  Its major goals
are to expose as easy to use, practical, and flexible of an API between Rust and
Lua as possible, while also being *completely* safe.

`rlua` is NOT designed to be a perfect zero cost wrapper over the Lua C API,
because such a wrapper cannot maintain the safety guarantees that `rlua` is
designed to have.  Every place where the Lua C API may trigger an error longjmp
in any way is protected by `lua_pcall`, and the user of the library is protected
from directly interacting with unsafe things like the Lua stack, and there is
overhead associated with this safety.  However, performance *is* a focus of the
library to the extent possible while maintaining safety, so if you encounter
something that egregiously worse than using the Lua C API directly, or simply
something you feel could perform better, feel free to file a bug report.

There are currently a few missing pieces of this API:

  * Security limits on Lua code such as total instruction limits / memory limits
    and control over which potentially dangerous libraries (e.g. io) are
    available to scripts.
  * Lua profiling support
  * "Context" or "Sandboxing" support.  There should be the ability to set the
    `_ENV` upvalue of a loaded chunk to a table other than `_G`, so that you can
    have different environments for different loaded chunks.
  * Quantifying performance differences to direct use of the Lua C API.

Additionally, there are ways I would like to change this API, once support lands
in rustc.  For example:

  * Currently, variadics are handled entirely with tuples and traits implemented
    by macro for tuples up to size 12, it would be great if this was replaced
    with real variadic generics when this is available in Rust.

## API stability

This library is very much Work In Progress, so there is a some API churn.
Currently, it follows a pre-1.0 semver, so all API changes should be accompanied
by 0.x version bumps.

## Safety and panics

The goal of this library is complete safety, it should not be possible to cause
undefined behavior whatsoever with the API, even in edge cases.  There is,
however, QUITE a lot of unsafe code in this crate, and I would call the current
safety level of the crate "Work In Progress".  Still, I am not *currently* aware
of any way to cause UB, and UB is considered the most serious kind of bug, so if
you find the ability to cause UB with this API *at all*, please file a bug
report.

Another goal of this library is complete protection from panics and aborts.
Currently, it should not be possible for a script to trigger a panic or abort
(with some important caveats described below).  Similarly to the safety goal,
there ARE several internal panics and even aborts in `rlua` source, but they
should not be possible to trigger, and if you trigger them this should be
considered a bug.

Caveats to the panic / abort guarantee:

  * `rlua` reserves the right to panic on API usage errors.  Currently, the only
    time this will happen is when passed a handle type from a `Lua` instance
    that does not share the same main state.
  * Currently, there are no memory or execution limits on scripts, so untrusted
    scripts can always at minimum infinite loop or allocate arbitrary amounts of
    memory.
  * The internal Lua allocator is set to use `realloc` from `libc`, but it is
    wrapped in such a way that OOM errors are guaranteed to *abort*.  This is
    not currently such a huge deal outside of untrusted scripts, as this matches
    the behavior of Rust itself.  Doing this allows the internals of `rlua` to,
    in certain cases, call 'm' Lua C API functions with the garbage collector
    disabled and know that these cannot error.  Eventually, `rlua` will support
    memory limits on scripts, and those memory limits will cause regular memory
    errors rather than OOM aborts.
  * `rustc` version `1.24.0` on Windows contains a
    [bug](https://github.com/rust-lang/rust/issues/48251) which affects `rlua`
    error handling, turning any Lua script error into an abort.  If you are
    using Rust `1.24.0` on windows, please upgrade to `1.24.1`.

Yet another goal of the library is to, in all cases, safely handle panics
generated by Rust callbacks.  Panic unwinds in Rust callbacks should currently
be handled correctly -- the unwind is caught and carried across the Lua API
boundary as a regular Lua error in a way that prevents Lua from catching it.
This is done by overriding the normal Lua 'pcall' and 'xpcall' with custom
versions that cannot catch errors that are actually from Rust panics, and by
handling panic errors on the receiving Rust side by resuming the panic.

`rlua` should also be panic safe in another way as well, which is that any `Lua`
instances or handles should remain usable after a user triggered panic, and such
panics should not break internal invariants or leak Lua stack space.  This is
mostly important to safely use `rlua` types in Drop impls, as you should not be
using panics for general error handling.

In summary, here is a list of `rlua` behaviors that should be considered a bug.
If you encounter them, a bug report would be very welcome:

  * If you can cause UB at all with `rlua` without typing the word "unsafe",
    this is absolutely 100% a bug.
  * If your code panics / aborts with a message that contains the string "rlua
    internal error", this is a bug.
  * The above is true even for the internal panic about running out of stack
    space!  There are a few ways to generate normal script errors by running out
    of stack, but if you encounter a *panic* based on running out of stack, this
    is a bug.
  * If you load the "debug" library (which requires typing "unsafe"), every
    safety / panic / abort guarantee goes out the window.  The debug library can
    be used to do extremely scary things.  If you use the debug library and
    encounter a bug, it may still very well be a bug, but try to find a
    reproduction that does not involve the debug library first.
  * When the internal version of Lua is built using the `gcc` crate, and
    `cfg!(debug_assertions)` is true, Lua is built with the `LUA_USE_APICHECK`
    define set.  Any abort caused by this internal Lua API checking is
    *absolutely* a bug, particularly because without `LUA_USE_APICHECK` it would
    generally cause UB.
  * Lua C API errors are handled by lonjmp.  *ALL* instances where the Lua C API
    would longjmp should be protected from Rust, except in internal callbacks
    where this is intentional.  If you detect that `rlua` is triggering a
    longjmp over your Rust stack frames, this is a bug!
  * If you can somehow handle a panic in a Rust callback from Lua, this is a
    bug.
  * If you detect that, after catching a panic, a `Lua` or handle method is
    triggering other bugs or there is a Lua stack space leak, this is a bug.
    `rlua` instances are supposed to remain fully usable in the face of user
    triggered panics.  This guarantee does NOT extend to panics marked with
    "rlua internal error" simply because that is already indicative of a
    separate bug.
