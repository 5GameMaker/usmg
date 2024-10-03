#[macro_use]
extern crate log;

pub mod game;
pub mod interface;
pub mod util;

pub use interface::*;

use assets::Resources;
use game::{Flow, Game};
use util::Result;

pub struct Application<I: Interface> {
    pub interface: I,
    pub game: Game<I>,
    pub resources: Resources<I::Tex, I::Font>,
}
impl<I: Interface> Application<I> {
    pub fn tick(&mut self) -> Result<Flow> {
        self.game
            .process_events(&mut self.interface, &self.resources)
    }
}
