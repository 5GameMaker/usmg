use std::{collections::VecDeque, ffi::c_void, time::SystemTime};

use app::{util::Result, Event, Interface, KeyState};
use gl::types::GLint;
use sdl2::{
    keyboard::Scancode,
    video::{GLContext, Window},
    EventPump, Sdl, VideoSubsystem,
};
use skia_safe::{
    canvas::SrcRectConstraint,
    gpu::{
        backend_render_targets, direct_contexts,
        gl::{FramebufferInfo, Interface as GlInterface},
        surfaces, DirectContext, SurfaceOrigin,
    },
    Color4f, ColorType, Font, Image, Paint, Surface, TextBlob, Typeface,
};

fn create_surface(
    window: &Window,
    fb_info: FramebufferInfo,
    gr_context: &mut skia_safe::gpu::DirectContext,
    num_samples: usize,
    stencil_size: usize,
) -> Surface {
    let size = window.size();
    let size = (
        size.0.try_into().expect("Could not convert width"),
        size.1.try_into().expect("Could not convert height"),
    );
    let backend_render_target =
        backend_render_targets::make_gl(size, num_samples, stencil_size, fb_info);

    surfaces::wrap_backend_render_target(
        gr_context,
        &backend_render_target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .expect("Could not create skia surface")
}

pub struct SdlInterface {
    startup: SystemTime,
    event_queue: VecDeque<Event<Self>>,
    last_frame_time: SystemTime,
    time: SystemTime,
    events: EventPump,
    surface: Surface,
    dctx: DirectContext,
    fb_info: FramebufferInfo,
    _gl_ctx: GLContext,
    window: Window,
    video: VideoSubsystem,
    _sdl: Sdl,
}
impl SdlInterface {
    pub fn new() -> Result<Self> {
        let startup = SystemTime::now();

        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let events = sdl.event_pump()?;
        let window = video
            .window("sdl window", 800, 600)
            .opengl()
            .resizable()
            .build()?;

        video.gl_load_library_default()?;
        let ctx = window.gl_create_context()?;
        window.gl_make_current(&ctx)?;

        gl::load_with(|x| video.gl_get_proc_address(x) as *const c_void);
        let int = GlInterface::new_load_with(|x| {
            if x == "eglGetCurrentDisplay" {
                return std::ptr::null();
            }
            video.gl_get_proc_address(x) as *const c_void
        })
        .expect("Failed to create gl interface");

        let mut dctx =
            direct_contexts::make_gl(int, None).expect("Failed to create direct context");

        let fb_info = {
            let mut fboid: GLint = 0;
            unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

            FramebufferInfo {
                fboid: fboid.try_into().unwrap(),
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
                ..Default::default()
            }
        };

        let surface = create_surface(
            &window,
            fb_info,
            &mut dctx,
            video.gl_attr().multisample_samples() as usize,
            video.gl_attr().stencil_size() as usize,
        );

        Ok(Self {
            _sdl: sdl,
            video,
            _gl_ctx: ctx,
            surface,
            dctx,
            fb_info,
            events,
            window,
            startup,
            time: SystemTime::now(),
            last_frame_time: SystemTime::now(),
            event_queue: VecDeque::with_capacity(32),
        })
    }

    pub fn reset(&mut self) {
        self.event_queue.clear();

        let now = SystemTime::now();
        self.time = now;

        if self.last_frame_time == SystemTime::UNIX_EPOCH {
            self.event_queue.push_front(Event::Redraw(0.0));
            self.last_frame_time = now;
        } else {
            let delay = now
                .duration_since(self.last_frame_time)
                .unwrap_or_default()
                .as_secs_f32();
            if delay >= 1.0 / self.target_framerate() as f32 {
                self.event_queue.push_front(Event::Redraw(delay));
                self.last_frame_time = now;
            }
        };

        if self.window.size() != (self.surface.width() as u32, self.surface.height() as u32) {
            self.surface = create_surface(
                &self.window,
                self.fb_info,
                &mut self.dctx,
                self.video.gl_attr().multisample_samples() as usize,
                self.video.gl_attr().stencil_size() as usize,
            );
        }

        while let Some(x) = self.events.poll_event() {
            use sdl2::event::Event as E;
            match x {
                E::Quit { .. } => self.event_queue.push_front(Event::Quit),
                E::KeyDown {
                    scancode: Some(scancode),
                    repeat,
                    ..
                } => {
                    if scancode == Scancode::Escape {
                        self.event_queue
                            .push_front(Event::Input(app::GenericKey::Esc));
                    }
                    if scancode == Scancode::Backspace {
                        self.event_queue
                            .push_front(Event::Input(app::GenericKey::Backspace));
                    }
                    if scancode == Scancode::Return || scancode == Scancode::Return2 {
                        self.event_queue
                            .push_front(Event::Input(app::GenericKey::Send));
                    }
                    self.event_queue.push_front(Event::Key {
                        key: scancode,
                        state: KeyState::Pressed,
                        repeat,
                    });
                }
                E::KeyUp {
                    scancode: Some(scancode),
                    repeat,
                    ..
                } => {
                    self.event_queue.push_front(Event::Key {
                        key: scancode,
                        state: KeyState::Released,
                        repeat,
                    });
                }
                E::TextInput { text, .. } => {
                    self.event_queue
                        .push_front(Event::Input(app::GenericKey::Text(text)));
                }
                _ => (),
            }
        }
    }

    pub fn swap(&mut self) {
        self.dctx.flush_submit_and_sync_cpu();
        self.window.gl_swap_window();
    }
}
impl Interface for SdlInterface {
    type Key = Scancode;
    type CursorId = ();
    type OtherCursorButton = u8;
    type Tex = Image;
    type Font = Typeface;

    fn now(&self) -> std::time::SystemTime {
        self.time
            - unsafe {
                self.startup
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_unchecked()
            }
    }

    fn target_framerate(&self) -> u16 {
        // TODO: Make it editable.
        60
    }

    fn poll(&mut self) -> Option<Event<Self>> {
        self.event_queue.pop_back()
    }

    fn focused(&self) -> bool {
        // TODO: Actually track mouse focus.
        true
    }

    fn held(&self, key: &Self::Key) -> bool {
        // TODO: Handle keyboard keys
        false
    }

    fn size(&self) -> app::ScreenSize {
        self.window.size().into()
    }

    fn fill_text_raw(
        &mut self,
        rgba: app::Rgba,
        size: u16,
        pos: app::ScreenPos,
        font: &Self::Font,
        text: &str,
    ) {
        let color = Color4f::new(
            rgba.0 as f32 / 255.0,
            rgba.1 as f32 / 255.0,
            rgba.2 as f32 / 255.0,
            rgba.3 as f32 / 255.0,
        );
        let paint = Paint::new(color, None);
        let font = Font::new(font, Some(size as f32));
        if let Some(x) = TextBlob::new(text, &font) {
            self.surface
                .canvas()
                .draw_text_blob(x, (pos.0, pos.1), &paint);
        }
    }

    fn clear_raw(&mut self, color: app::Rgba) {
        let color = Color4f::new(
            color.0 as f32 / 255.0,
            color.1 as f32 / 255.0,
            color.2 as f32 / 255.0,
            color.3 as f32 / 255.0,
        );
        self.surface.canvas().clear(color);
    }

    fn copy_raw(&mut self, sprite: assets::Sprite<Self::Tex>, dest: app::ScreenRect) {
        self.surface.canvas().draw_image_rect(
            sprite.tex,
            Some((
                &skia_safe::Rect::new(
                    sprite.rect.0 as f32,
                    sprite.rect.1 as f32,
                    sprite.rect.2 as f32 + sprite.rect.0 as f32,
                    sprite.rect.3 as f32 + sprite.rect.1 as f32,
                ),
                SrcRectConstraint::Fast,
            )),
            skia_safe::Rect::new(
                *dest.x1() as f32,
                *dest.y1() as f32,
                *dest.x2() as f32,
                *dest.y2() as f32,
            ),
            &Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None),
        );
    }
}
