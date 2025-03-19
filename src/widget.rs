use std::{any::Any, fmt::Display, fs::File, io::Write, os::fd::{AsFd, AsRawFd}, sync::{Arc, LockResult, RwLock, RwLockWriteGuard}};

use rusttype::{point, Scale};
use wayland_client::{globals::{registry_queue_init, GlobalListContents}, protocol::{wl_buffer::WlBuffer, wl_compositor::WlCompositor, wl_keyboard::{KeyState, KeymapFormat, WlKeyboard}, wl_registry::{self, WlRegistry}, wl_seat::{Capability, WlSeat}, wl_shm::WlShm, wl_shm_pool::WlShmPool, wl_surface::WlSurface}, Connection, Dispatch, DispatchError, EventQueue};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1::ZwlrLayerShellV1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1};
use tempfile::tempfile;
use xkbcommon::xkb::{self, Keycode, Keymap, Keysym};

reexport!(wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer, "wayland-protocols-wlr-reexport");
reexport!(wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor, "wayland-protocols-wlr-reexport");
reexport!(wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::KeyboardInteractivity, "wayland-protocols-wlr-reexport");
reexport!(rusttype::Font, "rusttype-reexport");


use crate::{pixel_util::{dist_to_arc, dist_to_line, Vector2}, reexport};


type KeyStateRc = Option<Arc<RwLock<xkb::State>>>;

pub struct WidgetData {
    key_state: KeyStateRc,
}

unsafe impl Send for WidgetData {}
unsafe impl Sync for WidgetData {}

impl Dispatch<WlRegistry, GlobalListContents> for WidgetData {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        data: &GlobalListContents,
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            println!("[{}] {} (v{})", name, interface, version);
        }
    }
}

impl Dispatch<WlCompositor, ()> for WidgetData {
    fn event(
        state: &mut Self,
        proxy: &WlCompositor,
        event: <WlCompositor as wayland_client::Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // Has no events
    }
}


impl Dispatch<WlSurface, ()> for WidgetData {
    fn event(
        state: &mut Self,
        proxy: &WlSurface,
        event: wayland_client::protocol::wl_surface::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_surface::Event;
        // We don't care about events
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for WidgetData {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerShellV1,
        event: wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // Has no events
    }
}


impl Dispatch<ZwlrLayerSurfaceV1, ()> for WidgetData {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerSurfaceV1,
        event: wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Event;
        if let Event::Configure { serial, width, height } = event {
            proxy.ack_configure(serial);
        }
        // We don't care about "Closed" event
    }
}



impl Dispatch<WlShm, ()> for WidgetData {
    fn event(
        state: &mut Self,
        proxy: &WlShm,
        event: wayland_client::protocol::wl_shm::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_shm::Event;
        // We don't care about "Format" event
    }
}

impl Dispatch<WlShmPool, ()> for WidgetData {
    fn event(
        state: &mut Self,
        proxy: &WlShmPool,
        event: <WlShmPool as wayland_client::Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // Has no events
    }
}

impl Dispatch<WlBuffer, ()> for WidgetData {
    fn event(
        state: &mut Self,
        proxy: &WlBuffer,
        event: wayland_client::protocol::wl_buffer::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        // We don't care about "Release" event
    }
}

impl Dispatch<WlSeat, Events> for WidgetData {
    fn event(
        state: &mut Self,
        proxy: &WlSeat,
        event: wayland_client::protocol::wl_seat::Event,
        data: &Events,
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_seat::Event;
        println!("Seat Event!");
        match event {
            Event::Capabilities { capabilities } => {
                if capabilities.into_result().unwrap().contains(Capability::Keyboard) {
                    println!("Capabilities");
                    proxy.get_keyboard(&qhandle, data.clone());
                }
            },
            _ => {}
        }
    }
}

impl Dispatch<WlKeyboard, Events> for WidgetData {
    fn event(
        state: &mut Self,
        proxy: &WlKeyboard,
        event: wayland_client::protocol::wl_keyboard::Event,
        data: &Events,
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_keyboard::Event;
        match event {
            Event::Keymap { format, fd, size } => {
                if format.into_result().unwrap() == KeymapFormat::XkbV1 {
                    println!("XKB keyboard found!");
                    unsafe {
                        let map = Keymap::new_from_fd(&xkb::Context::new(0), fd, size as usize, format.into_result().unwrap().into(), 0).unwrap().unwrap();
                        state.key_state = Some(Arc::new(RwLock::new(xkb::State::new(&map))));
                    }
                } else {
                    println!("Non-XKB keyboard, disconnecting!");
                    proxy.release();
                }
            },
            Event::Key { serial, time, key, state: key_state } => {
                let sym = state.key_state.as_ref().unwrap().read().unwrap();
                match key_state.into_result().unwrap() {
                    KeyState::Released => (data.key_released)(sym.key_get_one_sym(Keycode::new(key + 8)), data.comp.clone()),
                    KeyState::Pressed => (data.key_pressed)(sym.key_get_one_sym(Keycode::new(key + 8)), data.comp.clone()),
                    _ => {},
                }
            },
            _ => {},
        }
    }
}

#[derive(Clone)]
pub struct Events {
    pub key_pressed: Arc<dyn Fn(Keysym, ComponentsRc) + Send + Sync>,
    pub key_released: Arc<dyn Fn(Keysym, ComponentsRc) + Send + Sync>,
    pub comp: ComponentsRc,
}

impl Events {
    pub fn none() -> Self {
        Self {key_pressed: Arc::new(|_,_| ()), key_released: Arc::new(|_,_| ()), comp: None}
    }
    pub fn new<F1: Fn(Keysym, ComponentsRc) + Send + Sync + 'static, F2: Fn(Keysym, ComponentsRc) + Send + Sync + 'static>(key_pressed: F1, key_released: F2) -> Self {
        Self {
            key_pressed: Arc::new(key_pressed),
            key_released: Arc::new(key_released),
            comp: None,
        }
    }
}

pub struct WidgetBuilder<'a> {
    buffer: Vec<u8>,
    conn: &'a Connection,
    width: u32,
    height: u32,
    layer: wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer,
    anchor: Anchor,
    exclusive_edge: Anchor,
    exclusive_zone: i32,
    margin: Margin,
    kb_interactivity: KeyboardInteractivity,
    events: Events,
}



impl<'a> WidgetBuilder<'a> {
    pub fn new(conn: &'a Connection, width: u32, height: u32) -> Self {
        Self {
            buffer: vec![],            
            conn,
            width,
            height,
            anchor: Anchor::empty(),
            layer: Layer::Background,
            exclusive_edge: Anchor::empty(),
            exclusive_zone: 0,
            margin: Margin {top: 0, bottom: 0, left: 0, right: 0},
            kb_interactivity: KeyboardInteractivity::None,
            events: Events::none(),
        }
    }

    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = layer.into();
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor.into();
        self
    }

    pub fn exclusive_edge(mut self, anchor: Anchor) -> Self {
        self.exclusive_edge = anchor.into();
        self
    }

    pub fn exclusive_zone(mut self, m: i32) -> Self {
        self.exclusive_zone = m;
        self
    }

    pub fn margin(mut self, m: Margin) -> Self {
        self.margin = m;
        self
    }

    pub fn kb_interactivity(mut self, i: KeyboardInteractivity, e: Events) -> Self {
        self.kb_interactivity = i;
        self.events = e;
        self
    }

    pub fn build(&self) -> Widget<'a> {
        Widget { 
            conn: self.conn, 
            width: self.width, 
            height: self.height, 
            layer: self.layer , 
            anchor: self.anchor, 
            exclusive_edge: self.exclusive_edge, 
            exclusive_zone: self.exclusive_zone,
            comp: None,
            margin: self.margin,
            kb_interactivity: self.kb_interactivity,
            events: self.events.clone(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        let bytes = value.to_ne_bytes();
        
        Self {
           r: bytes[0],
           g: bytes[1],
           b: bytes[2],
           a: bytes[3], 
        }
    }
}

impl Color {
    pub fn lerp(&self, bg: Self, value: f32) -> Self {
        Self {
            r: (value * self.r as f32 + (1.0 - value) * bg.r as f32).round() as u8,
            g: (value * self.g as f32 + (1.0 - value) * bg.g as f32).round() as u8,
            b: (value * self.b as f32 + (1.0 - value) * bg.b as f32).round() as u8,
            a: (value * self.a as f32 + (1.0 - value) * bg.a as f32).round() as u8,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Margin {
    top: u32,
    bottom: u32,
    left: u32,
    right: u32
}

impl Margin {
    pub fn new() -> Self {
        Self {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0
        }
    }

    pub fn top(mut self, top: u32) -> Self {
        self.top = top;
        self
    }

    pub fn bottom(mut self, top: u32) -> Self {
        self.bottom = top;
        self
    }

    pub fn left(mut self, top: u32) -> Self {
        self.left = top;
        self
    }
    
    pub fn right(mut self, top: u32) -> Self {
        self.right = top;
        self
    }
}


pub struct WidgetComponents {
    pub surface: WlSurface,
    pub shm: WlShm,
    pub queue: EventQueue<WidgetData>,
    pub buffer: Vec<u8>,
    width: u32,
    height: u32,
    data: WidgetData,
    pub running: bool,
}

impl WidgetComponents {
    fn get_buffer(&self) -> Result<File, WidgetError> {
        let qh = self.queue.handle();
        let file = tempfile()?;
        let size = self.buffer.len();
        file.set_len(size as u64)?;
        let pool = self.shm.create_pool(file.as_fd(), size as i32, &qh, ());
        let buffer = pool.create_buffer(0, self.width as i32, self.height as i32, self.width as i32 * 4, wayland_client::protocol::wl_shm::Format::Argb8888, &qh, ());

        self.surface.attach(Some(&buffer), 0, 0);
        self.surface.commit();

        Ok(file)
    }
    
    pub fn draw_text<F: Fn(u32, char) -> Color>(&mut self, text: String, pos: Vector2, size: f32, font: Font, colorf: F, bg: Color) -> Result<(), WidgetError> {
        let mut file = self.get_buffer()?;

        
        let scale = Scale::uniform(size);  // Font size
        let start = point(pos.x, pos.y);     // Position to start rendering text

        let mut chars = text.chars();

        // Render each character of the text
        for (index, glyph) in Into::<Font>::into(font).layout(&text, scale, start).enumerate() {
            let char = chars.next().unwrap();
            glyph.draw(|x, y, v| {
                let rect = glyph.pixel_bounding_box().unwrap();
                let ny = rect.min.y as i32 + y as i32;
                let nx = rect.min.x as i32 + x as i32;
                if ny < 0 || nx < 0 {
                    return;
                }
                let pixel_index = (ny as usize * self.width as usize + nx as usize) * 4;
                let color = colorf(index as u32, char).lerp(bg, v);
                if pixel_index + 4 < self.buffer.len() {
                    self.buffer[pixel_index] = color.r;     // Red
                    self.buffer[pixel_index + 1] = color.g; // Green
                    self.buffer[pixel_index + 2] = color.b; // Blue
                    self.buffer[pixel_index + 3] = color.a; // Alpha (fully opaque)
                };
            });
        }

        file.write_all(&self.buffer)?;

        Ok(())
    }

    pub fn draw_rect(&mut self, color: Color, pos: Vector2, size: Vector2) -> Result<(), WidgetError> {
        let mut file = self.get_buffer()?;
        let pos_x = pos.x.round() as i32;
        let pos_y = pos.y.round() as i32;

        let size_x = size.x.round() as i32;
        let size_y = size.y.round() as i32;

        for x in pos_x.max(0)..(pos_x + size_x) {
            for y in pos_y.max(0)..(pos_y + size_y) {
                let pixel_index = (y as usize * self.width as usize + x as usize) * 4;
                if pixel_index + 4 > self.buffer.len() {
                    continue;
                }
                self.buffer[pixel_index] = color.r;
                self.buffer[pixel_index + 1] = color.g;
                self.buffer[pixel_index + 2] = color.b;
                self.buffer[pixel_index + 3] = color.a;
            }
        }
        file.write_all(&self.buffer)?;

        Ok(())
    }

    pub fn draw_line(&mut self, color: Color, a: Vector2, b: Vector2, thickness: f32) -> Result<(), WidgetError> {
        let margin = Vector2::new(thickness, thickness);
        let min = Vector2::new(a.x.min(b.x), a.y.min(b.y)) - margin;
        let max = Vector2::new(a.x.max(b.x), a.y.max(b.y)) + margin;
        self.draw_where(color, |v| dist_to_line(a, b, v) < thickness, min, max)
    }

    pub fn draw_arc(&mut self, color: Color, center: Vector2, radius: f32, start: f32, end: f32, thickness: f32) -> Result<(), WidgetError> {
        let margin = Vector2::new(thickness + radius, thickness + radius);
        let min = center - margin;
        let max = center + margin;
        self.draw_where(color, |v| dist_to_arc(center, radius, start, end, v) < thickness, min, max)        
    }

    pub fn draw_where<F : Fn(Vector2) -> bool>(&mut self, color: Color, predicate: F, min: Vector2, max: Vector2) -> Result<(), WidgetError> {
        let mut file = self.get_buffer()?;
        
        for x in min.x.round().max(0.0) as i32..max.x.round() as i32 {
            for y in min.y.round().max(0.0) as i32..max.y.round() as i32 {
                let pixel_index = (y as usize * self.width as usize + x as usize) * 4;
                if predicate(Vector2::new(x as f32, y as f32)) {
                    self.buffer[pixel_index] = color.r;
                    self.buffer[pixel_index + 1] = color.g;
                    self.buffer[pixel_index + 2] = color.b;
                    self.buffer[pixel_index + 3] = color.a;
                }
            }
        }

        file.write_all(&self.buffer)?;

        Ok(())
    }

    pub fn close(&mut self) -> Option<()> {
        self.running = false;
        Some(())
    }

    pub fn update(&mut self) -> Result<(), WidgetError> {
        self.queue.dispatch_pending(&mut self.data)?;
        Ok(())
    }

    pub fn update_blocking(&mut self) -> Result<(), WidgetError> {
        self.queue.blocking_dispatch(&mut self.data)?;
        Ok(())
    }


    pub fn roundtrip(&mut self) -> Result<(), WidgetError> {
        self.queue.roundtrip(&mut self.data)?;
        Ok(())
    }
}

pub struct Widget<'a> {
    conn: &'a Connection,
    width: u32,
    height: u32,
    layer: Layer,
    anchor: Anchor,
    exclusive_edge: Anchor,
    exclusive_zone: i32,
    margin: Margin,
    kb_interactivity: KeyboardInteractivity,
    events: Events,
    
    pub comp: ComponentsRc,    
}

#[derive(Debug)]
pub enum WidgetError {
    StdIO(std::io::Error),
    UninitializedWidget,
    WlDispatch(DispatchError),
}

impl From<std::io::Error> for WidgetError {
    fn from(value: std::io::Error) -> Self {
        Self::StdIO(value)
    }
}

impl From<DispatchError> for WidgetError {
    fn from(value: DispatchError) -> Self {
        Self::WlDispatch(value)
    }
}

 
impl Display for WidgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UninitializedWidget => f.write_str("Cannot draw to uninitialized widgets!"),
            Self::StdIO(e) => e.fmt(f),
            Self::WlDispatch(e) => e.fmt(f)
        }
    }
}


type ComponentsRc = Option<Arc<RwLock<WidgetComponents>>>;

impl std::error::Error for WidgetError {}

impl<'a> Widget<'a> {
    pub fn create_surface(&mut self, namespace: String) -> Result<(), WidgetError> {
        println!("Creating surface...");
        let (globals, mut queue) = registry_queue_init::<WidgetData>(&self.conn).unwrap();
        let qh = queue.handle();

        let mut state = WidgetData {key_state: None};

        queue.roundtrip(&mut state).unwrap();

        let compositor : WlCompositor = globals.bind(&qh, 5..=6, ()).unwrap();
        let layer_shell : ZwlrLayerShellV1 = globals.bind(&qh, 0..=5 , ()).unwrap();
        let shm : WlShm = globals.bind(&qh, 0..=1, ()).unwrap();


        let surface = compositor.create_surface(&qh, ());
        let layer_surface = layer_shell.get_layer_surface(&surface, None, self.layer, namespace, &qh, ());


        layer_surface.set_size(self.width, self.height);
        layer_surface.set_anchor(self.anchor);
        layer_surface.set_exclusive_edge(self.exclusive_edge);
        layer_surface.set_exclusive_zone(self.exclusive_zone);
        layer_surface.set_margin(self.margin.top as i32,self.margin.right as i32,self.margin.bottom as i32,self.margin.left as i32);
        layer_surface.set_keyboard_interactivity(self.kb_interactivity);

        surface.commit();
        queue.roundtrip(&mut state).unwrap();

        
        self.comp = Some(Arc::new(RwLock::new(WidgetComponents { running: true, surface, shm, queue, buffer: vec![0u8; (self.width * self.height * 4) as usize], width: self.width, height: self.height, data: state })));

        self.events.comp = self.comp.clone();

        let _seat : WlSeat = globals.bind(&qh, 8..=9, self.events.clone()).unwrap();

        self.comp.as_mut().unwrap().write().unwrap().roundtrip().unwrap();
        Ok(())
    }

    pub fn get_comp(&mut self) -> LockResult<RwLockWriteGuard<WidgetComponents>> {
        self.comp.as_mut().unwrap().write()
    }

}

pub fn make_connection() -> Connection {
    Connection::connect_to_env().unwrap()
}
