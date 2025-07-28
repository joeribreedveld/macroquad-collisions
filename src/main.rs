use hecs::World;
use macroquad::prelude::*;

// Constants
const DRAW_SIZE: f32 = 64.0;

// Components
struct Position(Vec2);

struct Player;

// Window config
fn window_conf() -> Conf {
    Conf {
        window_title: "Collisions".to_owned(),
        sample_count: 4,
        ..Default::default()
    }
}

// Main
#[macroquad::main(window_conf())]
async fn main() {
    let mut world = World::new();

    world.spawn((Position(Vec2::ZERO), Player));

    loop {
        clear_background(BLACK);

        movement(&mut world);

        render(&mut world);

        next_frame().await
    }
}

// Render system
fn render(world: &mut World) {
    for (_, pos) in world.query::<&Position>().iter() {
        let draw_pos = pos.0 - DRAW_SIZE / 2.0;

        draw_rectangle(draw_pos.x, draw_pos.y, DRAW_SIZE, DRAW_SIZE, WHITE);
    }
}

// Movement system
fn movement(world: &mut World) {
    for (_, pos) in world.query_mut::<&mut Position>().with::<&Player>() {
        let mouse_pos = Vec2::from(mouse_position());
        let mouse_draw_pos = mouse_pos - DRAW_SIZE / 2.0;

        draw_line(pos.0.x, pos.0.y, mouse_pos.x, mouse_pos.y, 2.0, GRAY);
        draw_rectangle_lines(
            mouse_draw_pos.x,
            mouse_draw_pos.y,
            DRAW_SIZE,
            DRAW_SIZE,
            4.0,
            GRAY,
        );

        if is_mouse_button_pressed(MouseButton::Left) {
            pos.0.x = mouse_pos.x;
            pos.0.y = mouse_pos.y;
        }
    }
}
