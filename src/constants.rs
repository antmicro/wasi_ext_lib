pub type Wasi_events = u32;
pub const WASI_EVENTS_NUM: usize = 1;
pub const WASI_EVENTS_MASK_SIZE: usize = 4; // number of bytes

// Hterm events
pub const WASI_EVENT_WINCH: Wasi_events = 1 << 0;
