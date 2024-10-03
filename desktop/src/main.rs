mod interface;

use std::{thread::sleep, time::Duration};

use app::{game::Flow, util::Result, Application};
use assets::include_resources;
use interface::SdlInterface;
use skia_safe::{Data, FontMgr, Image};

fn main() -> Result {
    pretty_env_logger::init();

    let font_mgr = FontMgr::new();
    let resources = include_resources! {
        x.png => Image::from_encoded(unsafe { Data::new_bytes(x.bytes) }).expect("Failed to load png image"),
        x.ttf => font_mgr.new_from_data(x.bytes, None).expect("Failed to load font"),
    };
    let interface = SdlInterface::new()?;
    let mut application = Application {
        game: Default::default(),
        interface,
        resources,
    };

    loop {
        application.interface.reset();
        match application.tick()? {
            Flow::Continue => (),
            Flow::Redraw => application.interface.swap(),
            Flow::Exit => break,
        }
        sleep(Duration::from_millis(4));
    }

    Ok(())
}
