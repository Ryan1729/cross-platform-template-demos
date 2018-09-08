#![allow(non_snake_case)]
#[macro_use]
extern crate common;

extern crate platform_types;

extern crate rand;

mod game;
pub use game::*;

mod game_state;
pub use game_state::GameState;
