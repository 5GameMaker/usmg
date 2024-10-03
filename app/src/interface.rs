use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::Hash,
    mem::swap,
    ops::{Add, Sub},
    time::SystemTime,
};

use assets::Sprite;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);
impl From<u32> for Rgba {
    fn from(value: u32) -> Self {
        let [r, g, b, a] = value.to_be_bytes();
        Rgba(r, g, b, a)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Vec2d<T>(pub T, pub T);
impl<T: PartialOrd> Vec2d<T> {
    pub fn sort(mut self, mut other: Vec2d<T>) -> (Vec2d<T>, Vec2d<T>) {
        if self
            .0
            .partial_cmp(&other.0)
            .is_some_and(|x| x == Ordering::Greater)
        {
            swap(&mut self.0, &mut other.0);
        }
        if self
            .1
            .partial_cmp(&other.1)
            .is_some_and(|x| x == Ordering::Greater)
        {
            swap(&mut self.1, &mut other.1);
        }
        (self, other)
    }
}
impl<T> From<(T, T)> for Vec2d<T> {
    fn from(value: (T, T)) -> Self {
        Self(value.0, value.1)
    }
}
impl<T> From<Vec2d<T>> for (T, T) {
    fn from(value: Vec2d<T>) -> Self {
        (value.0, value.1)
    }
}
impl<T: Add<Output = T>> Add for Vec2d<T> {
    type Output = Vec2d<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl<T: Sub<Output = T>> Sub for Vec2d<T> {
    type Output = Vec2d<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

pub type ScreenPos = Vec2d<i32>;
pub type ScreenSize = Vec2d<u32>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CenterRect<T, Y>(Vec2d<T>, Vec2d<Y>);
impl<T, Y> CenterRect<T, Y> {
    pub fn new(a: Vec2d<T>, b: Vec2d<Y>) -> Self {
        Self(a, b)
    }
}
impl<T, Y> From<(T, T, Y, Y)> for CenterRect<T, Y> {
    fn from(value: (T, T, Y, Y)) -> Self {
        Self::new((value.0, value.1).into(), (value.2, value.3).into())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Rect<T: PartialOrd>(Vec2d<T>, Vec2d<T>);
impl<T: PartialOrd> Rect<T> {
    pub fn new(a: Vec2d<T>, b: Vec2d<T>) -> Self {
        let (a, b) = a.sort(b);
        Self(a, b)
    }

    pub fn x1(&self) -> &T {
        &self.0 .0
    }

    pub fn x2(&self) -> &T {
        &self.1 .0
    }

    pub fn y1(&self) -> &T {
        &self.0 .1
    }

    pub fn y2(&self) -> &T {
        &self.1 .1
    }
}
impl<T: PartialOrd> From<(T, T, T, T)> for Rect<T> {
    fn from(value: (T, T, T, T)) -> Self {
        Self::new((value.0, value.1).into(), (value.2, value.3).into())
    }
}
impl<E, T: PartialOrd + Add<T, Output = T> + Clone, Y: TryInto<T, Error = E>>
    TryFrom<CenterRect<T, Y>> for Rect<T>
{
    type Error = E;

    fn try_from(value: CenterRect<T, Y>) -> Result<Self, Self::Error> {
        let second = Vec2d::<T>(
            value.0 .0.clone() + value.1 .0.try_into()?,
            value.0 .1.clone() + value.1 .1.try_into()?,
        );
        Ok(Self::new(value.0, second))
    }
}

pub type ScreenRect = Rect<i32>;
pub type ScreenCenterRect = CenterRect<i32, u32>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Horizonal,
    Vertical,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CursorButton<I: Interface + ?Sized> {
    Left,
    Right,
    ScroolWheel,
    Other(I::OtherCursorButton),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum GenericKey {
    /// Usually the enter key.
    Send,
    /// Backspace key.
    Backspace,
    /// Escape key or back button.
    Esc,
    /// Arbitrary text.
    Text(String),
}

#[derive(Debug)]
pub enum Event<I: Interface + ?Sized> {
    /// State of OS key has changed.
    Key {
        key: I::Key,
        state: KeyState,
        repeat: bool,
    },
    /// Window focus changed.
    Focused(bool),
    /// Generic keypress input.
    Input(GenericKey),
    /// A cursor has been moved.
    CursorMove(I::CursorId, ScreenPos),
    /// A cursor button state has changed.
    CursorButton(I::CursorId, CursorButton<I>, KeyState),
    /// A mouse has scrolled or a scroll gesture has been performed.
    Scrool(I::CursorId, Direction, u32),
    /// User requested window to close.
    Quit,
    /// System requested window to close.
    Terminate,
    /// Window needs to be redrawn
    Redraw(f32),
}

pub trait Interface {
    type Key: Display + Debug + Eq + Hash;
    type CursorId: Debug + Eq + Hash;
    type OtherCursorButton: Debug + Eq + Hash;
    type Tex;
    type Font;

    /// Poll events.
    fn poll(&mut self) -> Option<Event<Self>>;
    /// Window size.
    fn size(&self) -> ScreenSize;
    /// Current time. Should be cached for performance.
    fn now(&self) -> SystemTime;
    /// Whether window has mouse focus.
    fn focused(&self) -> bool;
    /// Whether a key is being held.
    fn held(&self, key: &Self::Key) -> bool;
    /// Framerate to attempt to average.
    fn target_framerate(&self) -> u16;

    /// Render text on screen.
    fn fill_text_raw(
        &mut self,
        rgba: Rgba,
        size: u16,
        pos: ScreenPos,
        font: &Self::Font,
        text: &str,
    );

    /// Clear all screen content.
    fn clear_raw(&mut self, color: Rgba);

    /// Draw sprite on screen.
    fn copy_raw(&mut self, sprite: Sprite<Self::Tex>, dest: ScreenRect);
}

pub trait InterfaceExt: Interface {
    /// Render text on screen.
    fn fill_text(
        &mut self,
        rgba: impl Into<Rgba>,
        size: u16,
        pos: impl Into<ScreenPos>,
        font: &Self::Font,
        text: impl AsRef<str>,
    );

    /// Clear all screen content.
    fn clear(&mut self, color: impl Into<Rgba>);

    /// Draw sprite on screen.
    fn copy(&mut self, sprite: Sprite<Self::Tex>, dest: impl Into<ScreenRect>);

    /// Draw sprite on screen.
    fn copy_center(&mut self, sprite: Sprite<Self::Tex>, dest: impl Into<ScreenCenterRect>);
}
impl<I: Interface + ?Sized> InterfaceExt for I {
    fn fill_text(
        &mut self,
        rgba: impl Into<Rgba>,
        size: u16,
        pos: impl Into<ScreenPos>,
        font: &Self::Font,
        text: impl AsRef<str>,
    ) {
        self.fill_text_raw(rgba.into(), size, pos.into(), font, text.as_ref())
    }

    fn clear(&mut self, color: impl Into<Rgba>) {
        self.clear_raw(color.into())
    }

    fn copy(&mut self, sprite: Sprite<Self::Tex>, dest: impl Into<ScreenRect>) {
        self.copy_raw(sprite, dest.into());
    }

    fn copy_center(&mut self, sprite: Sprite<Self::Tex>, dest: impl Into<ScreenCenterRect>) {
        let rect: ScreenCenterRect = dest.into();
        let rect: ScreenRect = rect.try_into().unwrap();
        self.copy_raw(sprite, rect);
    }
}
