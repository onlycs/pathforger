#![feature(
    never_type,
    error_generic_member_access,
    trait_alias,
    const_float_methods,
    array_windows
)]

extern crate futures;
extern crate itertools;
extern crate nt_client;
extern crate thiserror;
extern crate tokio;
extern crate uom;

mod error;
mod game;
mod networktables;
mod photon_serde;
mod prelude;
mod util;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
