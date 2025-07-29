use hecs::World;
use macroquad::prelude::*;

// Constants
const DRAW_SIZE: f32 = 64.0;

const PLAYER_SPEED: f32 = 300.0;

// Components
struct Position(Vec2);

struct Velocity(Vec2);

struct Player;

struct Speed(f32);

struct Collider(Vec2);

#[derive(Debug)]
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
        Velocity(Vec2::ZERO),
        Speed(PLAYER_SPEED),
        WHITE,
    ));

    // Wall
    let wall_start_pos = Vec2::new(screen_center.x + 128.0, screen_center.y);

    for i in -2..4 {
        let wall_pos = Vec2::new(wall_start_pos.x, wall_start_pos.y + DRAW_SIZE * i as f32);

        world.spawn((Position(wall_pos), Collider(entity_size), GRAY));
    }

    // Game loop
    loop {
        let delta_time = get_frame_time();

        clear_background(BLACK);

        input(&mut world);

        movement(&mut world, delta_time);

        mouse_movement(&mut world);

        collision(&mut world);

        render(&mut world);

        next_frame().await
    }
}

// Render system
fn render(world: &mut World) {
    for (_, (pos, color)) in world.query::<(&Position, &Color)>().iter() {
        let draw_pos = pos.0 - DRAW_SIZE / 2.0;

        draw_rectangle(
            draw_pos.x,
            draw_pos.y,
            DRAW_SIZE,
            DRAW_SIZE,
            color.to_owned(),
        );
    }
}

fn input(world: &mut World) {
    for (_, (vel, speed)) in world
        .query_mut::<(&mut Velocity, &Speed)>()
        .with::<&Player>()
    {
        let mut dir = Vec2::ZERO;

        if is_key_down(KeyCode::W) {
            dir.y -= 1.0;
        }

        if is_key_down(KeyCode::A) {
            dir.x -= 1.0;
        }

        if is_key_down(KeyCode::S) {
            dir.y += 1.0;
        }

        if is_key_down(KeyCode::D) {
            dir.x += 1.0;
        }

        vel.0 = dir.normalize_or_zero() * speed.0;
    }
}

fn movement(world: &mut World, delta_time: f32) {
    for (_, (pos, vel, prev_pos)) in world
        .query_mut::<(&mut Position, &Velocity, &mut PreviousPosition)>()
        .with::<&Player>()
    {
        prev_pos.0 = pos.0;

        pos.0 += vel.0 * delta_time;
    }
}

// Movement system
fn mouse_movement(world: &mut World) {
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
    let mut walls: Vec<Rect> = Vec::new();

    let extra = 0.0001;

    // Collect walls
    for (_, (pos, collider)) in world
        .query::<(&Position, &Collider)>()
        .without::<&Player>()
        .iter()
    {
        /* Revert origin to rect standard */
        let top_left_pos = pos.0 - DRAW_SIZE / 2.0;

        let rect = Rect::new(top_left_pos.x, top_left_pos.y, collider.0.x, collider.0.y);

        walls.push(rect);
    }

    // Check which one of the entities is movable
    for (_, (pos, collider, prev_pos)) in world
        .query_mut::<(&mut Position, &Collider, &mut PreviousPosition)>()
        .with::<&Player>()
    {
        let top_left_pos = pos.0 - DRAW_SIZE / 2.0;

        let player_rect = Rect::new(top_left_pos.x, top_left_pos.y, collider.0.x, collider.0.y);

        // Check if any entity overlaps one another
        for wall_rect in walls.iter() {
            if player_rect.overlaps(wall_rect) {
                // Check which axis overlapped in prev_pos
                if (prev_pos.0.x - wall_rect.center().x).abs() < DRAW_SIZE {
                    if prev_pos.0.y < wall_rect.top() {
                        // Top
                        pos.0.y = wall_rect.top() - DRAW_SIZE / 2.0 - extra;
                    } else {
                        // Bottom
                        pos.0.y = wall_rect.bottom() + DRAW_SIZE / 2.0 + extra;
                    }
                } else {
                    if prev_pos.0.x < wall_rect.left() {
                        // Left
                        pos.0.x = wall_rect.left() - DRAW_SIZE / 2.0 - extra;
                    } else {
                        // Right
                        pos.0.x = wall_rect.right() + DRAW_SIZE / 2.0 + extra;
                    }
                }
            }
        }
    }
}
