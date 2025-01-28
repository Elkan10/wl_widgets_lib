use std::{fmt::Display, fs::File, io::Write, os::fd::AsFd};

use rusttype::{point, Scale};
use wayland_client::{globals::{registry_queue_init, GlobalListContents}, protocol::{wl_buffer::WlBuffer, wl_compositor::WlCompositor, wl_registry::{self, WlRegistry}, wl_shm::WlShm, wl_shm_pool::WlShmPool, wl_surface::WlSurface}, Connection, Dispatch, DispatchError, EventQueue};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1::ZwlrLayerShellV1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1};
use tempfile::tempfile;

reexport!(wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer, "wayland-protocols-wlr-reexport");
reexport!(wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor, "wayland-protocols-wlr-reexport");
reexport!(wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::KeyboardInteractivity, "wayland-protocols-wlr-reexport");
reexport!(rusttype::Font, "rusttype-reexport");


use crate::{pixel_util::{dist_to_arc, dist_to_line, Vector2}, reexport};

pub struct WidgetData;


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

    pub fn kb_interactivity(mut self, i: KeyboardInteractivity) -> Self {
        self.kb_interactivity = i;
        self
    }

    pub fn build(&self) -> Widget<'a> {
        Widget { 
            buffer: vec![0u8; (self.width * self.height * 4) as usize],
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
        }
    }
}

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
}

pub struct Widget<'a> {
    buffer: Vec<u8>,
    conn: &'a Connection,
    width: u32,
    height: u32,
    layer: Layer,
    anchor: Anchor,
    exclusive_edge: Anchor,
    exclusive_zone: i32,
    margin: Margin,
    kb_interactivity: KeyboardInteractivity,
    
    comp: Option<WidgetComponents>,    
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


impl std::error::Error for WidgetError {}

impl<'a> Widget<'a> {
    pub fn create_surface(&mut self, namespace: String) {
        println!("Creating surface...");
        let (globals, mut queue) = registry_queue_init::<WidgetData>(&self.conn).unwrap();
        let qh = queue.handle();

        queue.roundtrip(&mut WidgetData).unwrap();

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
        queue.roundtrip(&mut WidgetData).unwrap();

        self.comp = Some(WidgetComponents { surface, shm, queue });

    }

    fn get_buffer(&self) -> Result<File, WidgetError> {
        let comp = self.comp.as_ref().ok_or(WidgetError::UninitializedWidget)?;
        let qh = comp.queue.handle();
        let file = tempfile()?;
        let size = self.width * self.height * 4;
        file.set_len(size as u64)?;
        let pool = comp.shm.create_pool(file.as_fd(), size as i32, &qh, ());
        let buffer = pool.create_buffer(0, self.width as i32, self.height as i32, self.width as i32 * 4, wayland_client::protocol::wl_shm::Format::Argb8888, &qh, ());

        comp.surface.attach(Some(&buffer), 0, 0);
        comp.surface.commit();

        Ok(file)
    }

    pub fn draw_text(&mut self, text: String, pos: Vector2, size: f32, font: Font, color: Color) -> Result<(), WidgetError> {
        let mut file = self.get_buffer()?;

        
        let scale = Scale::uniform(size);  // Font size
        let start = point(pos.x, pos.y);     // Position to start rendering text

        // Render each character of the text
        for glyph in Into::<Font>::into(font).layout(&text, scale, start) {
            glyph.draw(|x, y, v| {
                if v < 0.5 {
                    return;
                }
                let rect = glyph.pixel_bounding_box().unwrap();
                let ny = rect.min.y as u32 + y;
                let nx = rect.min.x as u32 + x;
                let pixel_index = (ny as usize * self.width as usize + nx as usize) * 4;
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
        let comp = self.comp.as_ref()?;
        comp.surface.destroy();
        self.comp = None;
        Some(())
    }

    pub fn update(&mut self) -> Result<(), WidgetError> {
        self.comp.as_mut().ok_or(WidgetError::UninitializedWidget)?.queue.dispatch_pending(&mut WidgetData)?;
        Ok(())
    }

    pub fn update_blocking(&mut self) -> Result<(), WidgetError> {
        self.comp.as_mut().ok_or(WidgetError::UninitializedWidget)?.queue.blocking_dispatch(&mut WidgetData)?;
        Ok(())
    }
}

pub fn make_connection() -> Connection {
    Connection::connect_to_env().unwrap()
}
