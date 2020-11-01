use amethyst::ecs::Component;
use amethyst::ecs::VecStorage;
use amethyst::core::math::Point2;
use crate::utils;

pub enum Collision {
    Circle(f32),
}

impl Collision {
    pub fn resolve(
        c1: &Self,
        c2: &Self,
        p1: &mut Point2<f32>,
        p2: &mut Point2<f32>,
    ) {
        match *c1 {
            Self::Circle(r1) => {
                match *c2 {
                    Self::Circle(r2) => {
                        Self::resolve_circle_circle(p1, p2, r1, r2);
                    }
                }
            }
        }
    }

    pub fn resolve_circle_circle(
        p1: &mut Point2<f32>,
        p2: &mut Point2<f32>,
        r1: f32,
        r2: f32,
    ) {
        if utils::math::are_closer_than(
            p1.x,
            p1.y,
            p2.x,
            p2.y,
            r1 + r2,
        ) {
            todo!();
        }
    }
}

impl Component for Collision {
    type Storage = VecStorage<Self>;
}
