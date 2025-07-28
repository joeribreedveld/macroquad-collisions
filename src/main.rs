use hecs::World;
use macroquad::prelude::*;

// Components
#[derive(Debug)]
struct Position(Vec2);

// Main
#[macroquad::main("Collisions")]
async fn main() {
    let mut world = World::new();

    world.spawn((Position(Vec2::ZERO),));

    loop {
        clear_background(BLACK);

        render(&mut world);

        next_frame().await
    }
}

// Render system
fn render(world: &mut World) {
    for (_, pos) in world.query::<&Position>().iter() {
        draw_rectangle(pos.0.x, pos.0.y, 16.0, 16.0, WHITE);
    }
}
