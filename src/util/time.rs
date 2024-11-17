#![allow(static_mut_refs)]

use crate::prelude::*;
use std::time::{Duration, Instant};

static mut PROGRAM_START: Option<Instant> = None;

pub fn initialize() {
    unsafe {
        if PROGRAM_START.is_some() {
            panic!("PROGRAM_START already initialized");
        }

        PROGRAM_START = Some(Instant::now());
    }
}

pub fn instant_of(dur: Duration) -> Instant {
    let Some(start) = (unsafe { PROGRAM_START.as_ref() }) else {
        panic!("PROGRAM_START not initialized");
    };

    *start + dur
}
