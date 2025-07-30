use hecs::{Entity, World};
use macroquad::prelude::*;

// Constants
const DRAW_SIZE: f32 = 64.0;

const PLAYER_SPEED: f32 = 300.0;

// Components

#[derive(Clone)]
struct Position(Vec2);

struct Velocity(Vec2);

struct Player;

struct Speed(f32);

#[derive(Clone)]
struct Collider(Vec2);

#[derive(Debug, Clone)]
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

    // Player
    world.spawn((
        Position(screen_center),
        Player,
        Collider(vec2(DRAW_SIZE, DRAW_SIZE)),
        PreviousPosition(screen_center),
        Velocity(Vec2::ZERO),
        Speed(PLAYER_SPEED),
        WHITE,
    ));

    world.spawn((
        Position(screen_center - 100.0),
        Collider(vec2(DRAW_SIZE, DRAW_SIZE)),
        PreviousPosition(screen_center - 100.0),
        Velocity(Vec2::ZERO),
        Speed(PLAYER_SPEED),
        RED,
    ));

    // Wall
    let wall_start_pos = Vec2::new(screen_center.x + 128.0, screen_center.y);

    for i in -2..4 {
        let wall_pos = Vec2::new(wall_start_pos.x, wall_start_pos.y + DRAW_SIZE * i as f32);

        world.spawn((
            Position(wall_pos),
            Collider(vec2(DRAW_SIZE, DRAW_SIZE)),
            GRAY,
        ));
    }

    // Game loop
    loop {
        let delta_time = get_frame_time();

        clear_background(BLACK);

        input(&mut world);

        movement(&mut world, delta_time);

        mouse_movement(&mut world);

        for _ in 0..8 {
            collision(&mut world);
        }

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
    for (_, (pos, vel, prev_pos)) in
        world.query_mut::<(&mut Position, &Velocity, &mut PreviousPosition)>()
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
struct Static {
    pos: Position,
    collider: Collider,
}

struct Dynamic {
    entity: Entity,
    collider: Collider,
    prev_pos: PreviousPosition,
}

pub fn collision(world: &mut World) {
    let mut static_entities: Vec<Static> = Vec::new();
    let mut dynamic_entities: Vec<Dynamic> = Vec::new();

    // Collect static entities
    for (_, (pos, collider)) in world
        .query::<(&Position, &Collider)>()
        .without::<&Velocity>()
        .iter()
    {
        static_entities.push(Static {
            pos: pos.clone(),
            collider: collider.clone(),
        });
    }

    // Collect dynamic entities
    for (entity, (collider, prev_pos)) in world
        .query::<(&Collider, &PreviousPosition)>()
        .with::<&Velocity>()
        .iter()
    {
        dynamic_entities.push(Dynamic {
            entity,
            collider: collider.clone(),
            prev_pos: prev_pos.clone(),
        });
    }

    // Loop over all dynamic entities
    for i in 0..dynamic_entities.len() {
        let mut mut_pos = world
            .get::<&mut Position>(dynamic_entities[i].entity)
            .unwrap();

        let prev_pos = world
            .get::<&mut PreviousPosition>(dynamic_entities[i].entity)
            .unwrap();

        // Set correct origin and init rect
        let top_left_pos = mut_pos.0 - DRAW_SIZE / 2.0;

        let dynamic_rect = Rect::new(
            top_left_pos.x,
            top_left_pos.y,
            dynamic_entities[i].collider.0.x,
            dynamic_entities[i].collider.0.y,
        );

        // Dynamic vs static
        for static_entity in &static_entities {
            // Set correct origin and init rect
            let top_left_pos = static_entity.pos.0 - DRAW_SIZE / 2.0;

            let static_rect = Rect::new(
                top_left_pos.x,
                top_left_pos.y,
                static_entity.collider.0.x,
                static_entity.collider.0.y,
            );

            let some_intersection = dynamic_rect.intersect(static_rect);

            let mut intersection = Rect::default();

            if some_intersection.is_some() {
                intersection = some_intersection.unwrap();
            }

            if intersection.w != 0.0 && intersection.h != 0.0 {
                // Check which axis overlapped in prev_pos
                if (dynamic_entities[i].prev_pos.0.x - static_rect.center().x).abs() < DRAW_SIZE {
                    println!("static vertical collision");

                    if dynamic_entities[i].prev_pos.0.y < static_rect.top() {
                        // Top
                        mut_pos.0.y = static_rect.top() - DRAW_SIZE / 2.0;
                    } else {
                        // Bottom
                        mut_pos.0.y = static_rect.bottom() + DRAW_SIZE / 2.0;
                    }
                } else {
                    println!("static horizontal collision");

                    if dynamic_entities[i].prev_pos.0.x < static_rect.left() {
                        // Left
                        mut_pos.0.x = static_rect.left() - DRAW_SIZE / 2.0;
                    } else {
                        // Right
                        mut_pos.0.x = static_rect.right() + DRAW_SIZE / 2.0;
                    }
                }
            }
        }

        // Dynamic vs dynamic
        for j in i + 1..dynamic_entities.len() {
            let mut other_mut_pos = world
                .get::<&mut Position>(dynamic_entities[j].entity)
                .unwrap();

            let other_prev_pos = world
                .get::<&mut PreviousPosition>(dynamic_entities[j].entity)
                .unwrap();

            let other_top_left_pos = other_mut_pos.0 - DRAW_SIZE / 2.0;

            let other_dynamic_rect = Rect::new(
                other_top_left_pos.x,
                other_top_left_pos.y,
                dynamic_entities[j].collider.0.x,
                dynamic_entities[j].collider.0.y,
            );

            let some_intersection = dynamic_rect.intersect(other_dynamic_rect);

            let mut intersection = Rect::default();

            if some_intersection.is_some() {
                intersection = some_intersection.unwrap();
            }

            if intersection.w != 0.0 && intersection.h != 0.0 {
                println!("y1: {}, y2: {},", prev_pos.0.y, other_prev_pos.0.y);

                // Check which axis overlaps
                if (prev_pos.0.x - other_prev_pos.0.x).abs().round() < DRAW_SIZE {
                    println!("dynamic vertical collision");

                    // Vertical collision
                    if prev_pos.0.y < other_prev_pos.0.y {
                        // Top
                        mut_pos.0.y -= intersection.h / 2.0;
                        other_mut_pos.0.y += intersection.h / 2.0;
                    } else {
                        // Bottom
                        mut_pos.0.y += intersection.h / 2.0;
                        other_mut_pos.0.y -= intersection.h / 2.0;
                    }
                } else {
                    println!("dynamic horizontal collision");

                    // Horizontal collision
                    if prev_pos.0.x < other_prev_pos.0.x {
                        // Left
                        mut_pos.0.x -= intersection.w / 2.0;
                        other_mut_pos.0.x += intersection.w / 2.0;
                    } else {
                        // Right
                        mut_pos.0.x += intersection.w / 2.0;
                        other_mut_pos.0.x -= intersection.w / 2.0;
                    }
                }
            }
        }
    }
}
