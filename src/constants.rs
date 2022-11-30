pub type Wasi_events = u32;
pub const WASI_EVENTS_NUM: usize = 1;

// Hterm events
pub const WASI_EVENT_WINCH: Wasi_events = 1 << 0;
