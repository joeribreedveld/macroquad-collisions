use hecs::World;
use macroquad::prelude::*;

// Constants
const DRAW_SIZE: f32 = 64.0;

// Components
struct Position(Vec2);

struct Player;

struct Collider(Vec2);

struct PreviousPosition(Vec2);

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

    let screen_size = Vec2::new(screen_width(), screen_height());
    let screen_center = screen_size / 2.0;

    let entity_size = Vec2::new(DRAW_SIZE, DRAW_SIZE);

    // Player
    world.spawn((
        Position(screen_center),
        Player,
        Collider(entity_size),
        PreviousPosition(Vec2::ZERO),
    ));

    // Wall
    let wall_pos = Vec2::new(screen_center.x + 256.0, screen_center.y);

    world.spawn((Position(wall_pos), Collider(entity_size)));

    // Game loop
    loop {
        clear_background(BLACK);

        movement(&mut world);

        collision(&mut world);

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
    for (_, (pos, prev_pos)) in world
        .query_mut::<(&mut Position, &mut PreviousPosition)>()
        .with::<&Player>()
    {
        /* Show movement guide */
        let mouse_pos = Vec2::from(mouse_position());
        let mouse_draw_pos = mouse_pos - DRAW_SIZE / 2.0;

        draw_line(pos.0.x, pos.0.y, mouse_pos.x, mouse_pos.y, 2.0, DARKGRAY);
        draw_rectangle_lines(
            mouse_draw_pos.x,
            mouse_draw_pos.y,
            DRAW_SIZE,
            DRAW_SIZE,
            4.0,
            DARKGRAY,
        );

        /* Update position */
        if is_mouse_button_pressed(MouseButton::Left) {
            prev_pos.0 = pos.0;

            pos.0.x = mouse_pos.x;
            pos.0.y = mouse_pos.y;
        }
    }
}

// Collision system
fn collision(world: &mut World) {
    let mut rect2 = Rect::default();

    let extra = 0.0001;

    // rect2 (wall)
    for (_, (pos, collider)) in world
        .query::<(&Position, &Collider)>()
        .without::<&Player>()
        .iter()
    {
        /* Revert origin to rect standard */
        let top_left_pos = pos.0 - DRAW_SIZE / 2.0;

        rect2 = Rect::new(top_left_pos.x, top_left_pos.y, collider.0.x, collider.0.y);
    }

    // Check which one of the entities is movable
    for (_, (pos, collider, prev_pos)) in world
        .query_mut::<(&mut Position, &Collider, &mut PreviousPosition)>()
        .with::<&Player>()
    {
        let top_left_pos = pos.0 - DRAW_SIZE / 2.0;

        let rect1 = Rect::new(top_left_pos.x, top_left_pos.y, collider.0.x, collider.0.y);

        // Check if any entity overlaps one another
        if rect1.overlaps(&rect2) {
            println!("Collision detected");

            // Check which axis has biggest difference
            if pos.0.x - prev_pos.0.x > pos.0.y - prev_pos.0.y {
                if prev_pos.0.x < pos.0.x {
                    // Left
                    pos.0.x = rect2.left() - DRAW_SIZE / 2.0 - extra;
                } else {
                    // Right
                    pos.0.x = rect2.right() + DRAW_SIZE / 2.0 + extra;
                }
            } else {
                if prev_pos.0.y < pos.0.y {
                    // Top
                    pos.0.y = rect2.top() - DRAW_SIZE / 2.0 - extra;
                } else {
                    // Bottom
                    pos.0.y = rect2.bottom() + DRAW_SIZE / 2.0 + extra;
                }
            }
        }
    }
}
