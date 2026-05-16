// Core infrastructure modules

#[cfg(not(target_family = "wasm"))]
pub mod updater;

#[cfg(not(target_family = "wasm"))]
pub mod credential_manager;

pub mod services;
pub mod event_bus;

// Re-export commonly used types
