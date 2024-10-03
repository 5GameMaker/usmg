use std::{marker::PhantomData, ops::Mul, time::SystemTime};

use assets::Resources;

use crate::{
    interface::{Interface, InterfaceExt},
    util::Result,
    Event, GenericKey,
};

pub enum Flow {
    Continue,
    Redraw,
    Exit,
}

pub struct Game<I: Interface + ?Sized> {
    _phantom: PhantomData<I>,
}
impl<I: Interface + ?Sized> Game<I> {
    pub fn process_events(
        &mut self,
        int: &mut I,
        res: &Resources<I::Tex, I::Font>,
    ) -> Result<Flow> {
        let mut flow = Flow::Continue;
        while let Some(x) = int.poll() {
            match x {
                Event::Quit => return Ok(Flow::Exit),
                Event::Input(GenericKey::Esc) => return Ok(Flow::Exit),
                Event::Redraw(_) => {
                    let shift = int
                        .now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f32()
                        .sin()
                        .mul(100.0)
                        .ceil() as i32;
                    int.clear(0x000000ff);
                    int.fill_text(0xffffffff, 20, (100, 100), &res.hack_regular_ttf, "Hello!");
                    int.copy_center(res.terrain_sprites_csv_sand(), (120 + shift, 120, 64, 64));
                    flow = Flow::Redraw;
                }
                _ => (),
            }
        }
        Ok(flow)
    }
}
impl<I: Interface + ?Sized> Default for Game<I> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}
