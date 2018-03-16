#![feature(nll)]
#![feature(crate_in_paths)]
#![feature(crate_visibility_modifier)]
#![feature(match_default_bindings)]

extern crate chrono;
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

extern crate serde;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate env_logger;

pub mod model;
pub mod order_book;
mod order_list;
