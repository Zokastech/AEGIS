// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Stub for a future NAPI-RS integration.
//! A dedicated Node addon can link this `cdylib` or duplicate the C calls.

/// Compile-time marker when the `nodejs` feature is enabled (`napi` / `napi-derive`).
pub const NAPI_ENABLED: bool = true;
