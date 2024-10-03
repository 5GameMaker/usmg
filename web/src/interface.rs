use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use app::{Event, Interface, Rgba};
use js_sys::Object;
use wasm_bindgen::prelude::*;
use web_sys::{
    window, CanvasRenderingContext2d, FontFace, HtmlCanvasElement, HtmlImageElement,
    HtmlStyleElement,
};

fn perf_to_system(amt: f64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs_f64(amt / 1000.0)
}

fn now() -> SystemTime {
    perf_to_system(window().unwrap().performance().unwrap().now())
}

pub struct WebInterface {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    time: SystemTime,
    pub(crate) events: Rc<RefCell<VecDeque<Event<Self>>>>,
    size: app::ScreenSize,
    fill_style: Rgba,
}
impl WebInterface {
    pub fn new() -> Self {
        let window = window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let style: HtmlStyleElement = document.create_element("style").unwrap().unchecked_into();
        style.set_inner_html("canvas{position:absolute;inset:0;image-rendering:pixelated}");
        body.append_child(&style).unwrap();

        let canvas: HtmlCanvasElement = document.create_element("canvas").unwrap().unchecked_into();
        canvas.set_width(window.inner_width().unwrap().as_f64().unwrap() as u32 + 1);
        canvas.set_height(window.inner_height().unwrap().as_f64().unwrap() as u32 + 1);
        body.append_child(&canvas).unwrap();

        let options = Object::new();
        js_sys::Reflect::set(
            &options,
            &JsValue::from_str("desynchronized"),
            &JsValue::TRUE,
        )
        .unwrap_throw();
        let ctx: CanvasRenderingContext2d = canvas
            .get_context_with_context_options("2d", &options)
            .unwrap_throw()
            .unwrap_throw()
            .unchecked_into();
        ctx.set_image_smoothing_enabled(false);

        let events = Rc::new(RefCell::new(VecDeque::new()));
        events.borrow_mut().push_back(Event::Redraw(0.0));

        Self {
            ctx,
            size: (canvas.width(), canvas.height()).into(),
            canvas,
            time: now(),
            events,
            fill_style: 0.into(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.size = (width, height).into();
        self.ctx.set_image_smoothing_enabled(false);
    }

    pub fn pre_update(&mut self) {
        self.time = now();
    }

    fn update_fill_style(&mut self, rgba: Rgba) {
        if self.fill_style == rgba {
            return;
        }
        self.ctx.set_fill_style(&JsValue::from_str(&format!(
            "#{:01$x}",
            u32::from_be_bytes([rgba.0, rgba.1, rgba.2, rgba.3]),
            8
        )));
        self.fill_style = rgba;
    }
}
impl Interface for WebInterface {
    type Key = String;
    type Tex = HtmlImageElement;
    type Font = FontFace;
    type CursorId = ();
    type OtherCursorButton = String;

    fn now(&self) -> std::time::SystemTime {
        self.time
    }

    fn poll(&mut self) -> Option<app::Event<Self>> {
        self.events.borrow_mut().pop_front()
    }

    fn size(&self) -> app::ScreenSize {
        self.size
    }

    fn held(&self, _key: &Self::Key) -> bool {
        todo!()
    }

    fn focused(&self) -> bool {
        todo!()
    }

    fn target_framerate(&self) -> u16 {
        60
    }

    fn clear_raw(&mut self, color: app::Rgba) {
        self.update_fill_style(0.into());
        self.ctx
            .fill_rect(0.0, 0.0, self.size.0 as f64, self.size.1 as f64);
        self.update_fill_style(color);
        self.ctx
            .fill_rect(0.0, 0.0, self.size.0 as f64, self.size.1 as f64);
    }

    fn fill_text_raw(
        &mut self,
        rgba: app::Rgba,
        size: u16,
        pos: app::ScreenPos,
        font: &Self::Font,
        text: &str,
    ) {
        self.update_fill_style(rgba);
        self.ctx
            .set_font(&format!("600 {size}px {}", font.family()));
        self.ctx
            .fill_text(text, pos.0 as f64, pos.1 as f64)
            .unwrap_throw();
    }

    fn copy_raw(&mut self, sprite: assets::Sprite<Self::Tex>, dest: app::ScreenRect) {
        self.ctx
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                sprite.tex,
                sprite.rect.0 as f64,
                sprite.rect.1 as f64,
                sprite.rect.2 as f64,
                sprite.rect.3 as f64,
                *dest.x1() as f64,
                *dest.y1() as f64,
                (*dest.x2() - *dest.x1()) as f64,
                (*dest.y2() - *dest.y1()) as f64,
            )
            .unwrap_throw();
    }
}
