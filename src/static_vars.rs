use std::sync::atomic::{AtomicBool, AtomicUsize};

pub(crate) static NET_IS_ENABLED: AtomicBool = AtomicBool::new(false);
pub(crate) static DEBUG_IS_ENABLED: AtomicBool = AtomicBool::new(false);
pub(crate) static I18N_UPDATE_COUNT: AtomicUsize = AtomicUsize::new(0);