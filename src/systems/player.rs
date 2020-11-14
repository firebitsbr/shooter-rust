use crate::components::Actor;
use crate::components::ActorActions;
use crate::components::Player;
use crate::input;
use crate::input::AxisBinding;
use crate::input::CustomBindingTypes;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::Join;
use amethyst::ecs::prelude::Read;
use amethyst::ecs::prelude::System;
use amethyst::ecs::prelude::SystemData;
use amethyst::ecs::prelude::WriteStorage;
use amethyst::ecs::prelude::ReadStorage;
use amethyst::input::InputHandler;
use std::f32::consts::TAU;

const ROTATION_SENSITIVITY: f32 = 0.01;

#[derive(SystemDesc)]
pub struct PlayerSystem;

impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        Read<'a, InputHandler<CustomBindingTypes>>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Actor>,
    );

    fn run(&mut self, (input, players, mut actors): Self::SystemData) {
        let rotation = input::take_mouse_delta() as f32 * ROTATION_SENSITIVITY;

        for (_, actor) in (&players, &mut actors).join() {
            actor.turning_to = (actor.turning_to - rotation) % TAU;

            temp(
                &mut actor.actions,
                ActorActions::MOVEMENT_FORWARD,
                ActorActions::MOVEMENT_BACKWARDS,
                input.axis_value(&AxisBinding::MoveForward).unwrap_or(0.0),
            );

            temp(
                &mut actor.actions,
                ActorActions::MOVEMENT_LEFT,
                ActorActions::MOVEMENT_RIGHT,
                input.axis_value(&AxisBinding::MoveAside).unwrap_or(0.0),
            );
        }
    }
}

// TODO: Rename
fn temp(actions: &mut ActorActions, a: ActorActions, b: ActorActions, ratio: f32) {
    if ratio > 0.0 {
        *actions |= a;
        *actions -= b;
    } else if ratio < 0.0 {
        *actions -= a;
        *actions |= b;
    } else {
        *actions -= a;
        *actions -= b;
    }
}
