#![feature(never_type, error_generic_member_access)]

extern crate nt_client;

mod error;
mod networktables;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
