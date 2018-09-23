//! Re-exports most types with an extra `Lua*` prefix to prevent name clashes.

pub use {
    Error as LuaError, ExternalError as LuaExternalError,
    ExternalResult as LuaExternalResult, FromLua, FromLuaMulti, Function as LuaFunction,
    Integer as LuaInteger, LightUserData as LuaLightUserData, Lua,
    MultiValue as LuaMultiValue, Nil as LuaNil, Number as LuaNumber, RegistryKey as LuaRegistryKey,
    Result as LuaResult, Scope as LuaScope, String as LuaString, Table as LuaTable,
    TablePairs as LuaTablePairs, TableSequence as LuaTableSequence, Thread as LuaThread,
    ThreadStatus as LuaThreadStatus, ToLua, ToLuaMulti,
    Value as LuaValue,
};
