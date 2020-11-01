use crate::components::Collision;
use crate::components::Interpolation;
use amethyst::core::transform::Transform;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::Join;
use amethyst::ecs::prelude::Write;
use amethyst::ecs::prelude::System;
use amethyst::ecs::prelude::SystemData;
use amethyst::ecs::prelude::ReadStorage;
use amethyst::ecs::prelude::WriteStorage;
use amethyst::renderer::debug_drawing::DebugLines;
use amethyst::renderer::palette::Srgba;
use amethyst::core::math::Point2;
use amethyst::core::math::Point3;
use crate::data::LAYER_DEBUG_LINES;
use amethyst::ecs::Entities;
use crate::utils;

#[derive(SystemDesc)]
pub struct CollisionSystem;

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Collision>,
        ReadStorage<'a, Interpolation>,
        Write<'a, DebugLines>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (entities, collisions, interpolations, mut debug, mut transforms): Self::SystemData) {
        let query = (&entities, &transforms, &(interpolations).maybe(), &collisions);

        for (entity, transform, interpolation, collision) in query.join() {
            let point = to_point(transform, interpolation);

            for (other_entity, other_transform, other_interpolation, other_collision) in query.join() {
                if entity.id() == other_entity.id() {
                    continue;
                }

                let other_point = to_point(other_transform, other_interpolation);

                if utils::math::are_closer_than(
                    point.x,
                    point.y,
                    other_point.x,
                    other_point.y,
                    collisions.
                ) {}
            }

            let center = Point3::from([
                transform.translation().x + interpolation.offset_x,
                transform.translation().y + interpolation.offset_y,
                LAYER_DEBUG_LINES,
            ]);

            match *collision {
                Collision::Circle(radius) => {
                    debug.draw_circle(
                        center,
                        radius,
                        14, // TODO: To constant
                        Srgba::new(0.6, 0.6, 0.6, 0.4), // TODO: To constant
                    );
                }
            }
        }
    }
}

fn to_point(transform: &Transform, interpolation: Option<&Interpolation>) -> Point2 {
    let mut x = transform.translation().x;
    let mut y = transform.translation().y;

    if let Some(interpolation) = interpolation {
        x += interpolation.offset_x;
        y += interpolation.offset_y;
    }

    return Point2::from([x, y]);
}
