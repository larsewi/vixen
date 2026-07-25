use bevy::prelude::*;
use vixen::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .run();
}
