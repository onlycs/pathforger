#![feature(never_type, error_generic_member_access, trait_alias)]

extern crate futures;
extern crate nt_client;
extern crate thiserror;
extern crate tokio;

mod error;
mod networktables;
mod photon_serde;
mod prelude;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
