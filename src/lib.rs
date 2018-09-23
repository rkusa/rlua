//! # High-level bindings to Lua
//!
//! The `rlua` crate provides safe high-level bindings to the [Lua programming language].
//!
//! # The `Lua` object
//!
//! The main type exported by this library is the [`Lua`] struct. In addition to methods for
//! [executing] Lua chunks or [evaluating] Lua expressions, it provides methods for creating Lua
//! values and accessing the table of [globals].
//!
//! # Converting data
//!
//! The [`ToLua`] and [`FromLua`] traits allow conversion from Rust types to Lua values and vice
//! versa. They are implemented for many data structures found in Rust's standard library.
//!
//! For more general conversions, the [`ToLuaMulti`] and [`FromLuaMulti`] traits allow converting
//! between Rust types and *any number* of Lua values.
//!
//! Most code in `rlua` is generic over implementors of those traits, so in most places the normal
//! Rust data structures are accepted without having to write any boilerplate.
//!
//! # Custom Userdata
//!
//! The [`UserData`] trait can be implemented by user-defined types to make them available to Lua.
//! Methods and operators to be used from Lua can be added using the [`UserDataMethods`] API.
//!
//! [Lua programming language]: https://www.lua.org/
//! [`Lua`]: struct.Lua.html
//! [executing]: struct.Lua.html#method.exec
//! [evaluating]: struct.Lua.html#method.eval
//! [globals]: struct.Lua.html#method.globals
//! [`ToLua`]: trait.ToLua.html
//! [`FromLua`]: trait.FromLua.html
//! [`ToLuaMulti`]: trait.ToLuaMulti.html
//! [`FromLuaMulti`]: trait.FromLuaMulti.html
//! [`UserData`]: trait.UserData.html
//! [`UserDataMethods`]: struct.UserDataMethods.html

// Deny warnings inside doc tests / examples. When this isn't present, rustdoc doesn't show *any*
// warnings at all.
#![doc(test(attr(deny(warnings))))]

#[cfg_attr(test, macro_use)]
extern crate failure;
extern crate libc;

mod error;
mod ffi;
#[macro_use]
mod macros;
mod conversion;
mod function;
mod lua;
mod multi;
mod scope;
mod string;
mod table;
mod thread;
mod types;
mod util;
mod value;

#[cfg(test)]
mod tests;

pub use error::{Error, ExternalError, ExternalResult, Result};
pub use function::Function;
pub use lua::Lua;
pub use multi::Variadic;
pub use scope::Scope;
pub use string::String;
pub use table::{Table, TablePairs, TableSequence};
pub use thread::{Thread, ThreadStatus};
pub use types::{Integer, LightUserData, Number, RegistryKey};
pub use value::{FromLua, FromLuaMulti, MultiValue, Nil, ToLua, ToLuaMulti, Value};

pub mod prelude;
