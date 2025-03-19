pub mod widget;
pub mod pixel_util;

mod macros;


#[cfg(test)]
mod tests {
    

    use std::{thread, time::Duration};

    use xkbcommon::xkb::Keysym;

    use crate::{pixel_util, widget::{self, Events}};

    #[test]
    fn test() {
        use std::{f32::consts::PI, fs::File, io::Read};

        use pixel_util::Vector2;
        use rusttype::Font;
        use wayland_client::Connection;
        use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1::Layer, zwlr_layer_surface_v1::Anchor};
        use widget::WidgetBuilder;

        let conn = Connection::connect_to_env().unwrap();
        let mut widget = WidgetBuilder::new(&conn, 300, 300)
            .layer(Layer::Overlay)
            .anchor(Anchor::Top)
            .exclusive_zone(1)
            .kb_interactivity(wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand,
                Events::new(move |key, comp| {
                    println!("pressed: {:?}", key);
                    if key == Keysym::Escape {
                        println!("Escape!");
                        comp.unwrap().write().unwrap().close();
                    }
                }, |key, comp| println!("released: {:?}", key)))
            .build();

        widget.create_surface("rust-widget".into()).unwrap();

    
        let path = "/media/Drive2/idea-IC-233.14475.28/jbr/lib/fonts/JetBrainsMono-Medium.ttf";
        let mut file = File::open(path).expect("Failed to load font");
        let mut font_data = Vec::new();
        file.read_to_end(&mut font_data).expect("Failed to read font file");
        let font = Font::try_from_vec(font_data).expect("Failed to parse font");
    

        widget.get_comp().unwrap().draw_rect(0x00FF0000.into(), Vector2::new(0.0,0.0), Vector2::new(300.0,100.0)).unwrap();
        widget.get_comp().unwrap().draw_text("Hello, World!".into(), Vector2::new(0.0, 40.0), 32.0, font, |_,_| 0xFF0000FF.into(), 0x00FF0000.into()).unwrap();
        //widget.draw_line(0xFF00FF00.into(), Vector2::new(0.0,0.0), Vector2::new(100.0,100.0), 10.0).unwrap();
        widget.get_comp().unwrap().draw_arc(0xFF00FF00.into(), Vector2::new(50.0, 50.0), 50.0, 0.0, PI, 10.0).unwrap();

        widget.get_comp().unwrap().update_blocking().unwrap();

        while widget.get_comp().unwrap().running {
            widget.get_comp().unwrap().update().unwrap();
            thread::sleep(Duration::from_millis(100));
        };
    }
}
