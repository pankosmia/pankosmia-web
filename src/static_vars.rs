use std::sync::atomic::AtomicBool;

pub(crate) static NET_IS_ENABLED: AtomicBool = AtomicBool::new(false);
pub(crate) static DEBUG_IS_ENABLED: AtomicBool = AtomicBool::new(false);
