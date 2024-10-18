#![feature(future_join)]
#![feature(async_closure)]
#![feature(type_changing_struct_update)]
#![feature(result_flattening)]
#![feature(map_try_insert)]
#![feature(let_chains)]

extern crate core;

pub use self::common::ExitStatus;

pub mod cached;
pub mod cmd;
pub mod common;
pub mod escape;
pub mod filter;
pub mod find;
pub mod java;
pub mod observe;
pub mod rsh;
pub mod textproc;
pub mod wait;
mod shywolf;
