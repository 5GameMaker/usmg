mod interface;

use app::Application;
use assets::include_resources;
use interface::WebInterface;
use wasm_bindgen::prelude::*;
use web_sys::{FontFace, HtmlImageElement};

#[wasm_bindgen]
extern "C" {
    async fn load_image(s: &str) -> HtmlImageElement;
    async fn load_font(s: &str) -> FontFace;
}

#[wasm_bindgen]
pub struct AppWrap(pub(crate) Application<WebInterface>);
#[wasm_bindgen]
impl AppWrap {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.0.interface.resize(width, height);
    }

    pub fn tick(&mut self, delta: f32) {
        self.0.interface.pre_update();
        self.0
            .interface
            .events
            .borrow_mut()
            .push_back(app::Event::Redraw(delta));
        self.0.tick().unwrap_throw();
    }
}

#[wasm_bindgen]
pub async fn init() -> AppWrap {
    console_log::init_with_level(log::Level::Trace).unwrap_throw();

    let resources = include_resources! {
        x.png => load_image(x.path),
        x.ttf => load_font(x.path),

        +{
            x.png => x.await,
            x.ttf => x.await,
        }
    };
    let interface = WebInterface::new();

    AppWrap(Application {
        interface,
        resources,
        game: Default::default(),
    })
}
