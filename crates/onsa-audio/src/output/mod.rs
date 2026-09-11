//! Where the engine's audio goes: a real device through cpal, or the offline
//! sink the tests render into (SPEC §3.1). Both drive the same
//! [`stage::OutputStage`].

pub mod device;
pub mod offline;
pub mod stage;
