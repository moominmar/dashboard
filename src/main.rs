use notan::prelude::*;
use notan::text::*;

mod draw;
mod state;
mod update;

#[notan_main]
fn main() -> Result<(), String> {
    let win_config = WindowConfig::new()
        .resizable(true)
        .maximized(true)
        .fullscreen(true);
    notan::init_with(state::setup)
        .add_config(win_config)
        .add_config(TextConfig)
        .update(update::update)
        .draw(draw::draw)
        .build()
}
