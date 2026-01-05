//! EVM Settings.

use crate::settings::{self, detail, Builder, Value};
use core::fmt;

include!(concat!(env!("OUT_DIR"), "/settings-evm.rs"));
