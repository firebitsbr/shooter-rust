use crate::resources::GameTask;
use crate::resources::GameTaskResource;
use crate::resources::Message;
use crate::resources::MessageReceiver;
use crate::resources::MessageResource;
use crate::resources::NetworkTask;
use crate::resources::NetworkTaskResource;
use crate::resources::MESSAGE_SIZE_MAX;
use crate::systems::net::connection::Connection;
use crate::systems::net::connection::ConnectionStatus;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::System;
use amethyst::ecs::prelude::SystemData;
use amethyst::ecs::prelude::Write;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::time::Duration;
use std::time::Instant;

const MESSAGE_RESEND_INTERVAL: Duration = Duration::from_millis(400); // TODO: Tweak

#[derive(SystemDesc)]
pub struct NetworkSystem {
    socket: UdpSocket,
    is_server: bool,
    connections: HashMap<SocketAddr, Connection>,
    last_message_resend: Instant,
}

impl NetworkSystem {
    pub fn new_as_server(port: u16) -> Result<Self, String> {
        return Self::new(&format!("0.0.0.0:{}", port), true);
    }

    pub fn new_as_client(server_address: SocketAddr) -> Result<Self, String> {
        let mut network = Self::new("0.0.0.0:0", false)?;
        network
            .connections
            .insert(server_address, Connection::new());

        network.send(&MessageReceiver::Every, Message::Greeting { id: 0 });
        return Ok(network);
    }

    fn new(address: &str, is_server: bool) -> Result<Self, String> {
        let socket = UdpSocket::bind(address).map_err(|e| format!("{}", e))?;
        socket.set_nonblocking(true).map_err(|e| format!("{}", e))?;

        return Ok(Self {
            socket,
            is_server,
            connections: HashMap::new(),
            last_message_resend: Instant::now(),
        });
    }

    fn send_outgoing_messages(&mut self, messages: &mut MessageResource) {
        for (receiver, message) in messages.drain(..) {
            self.send(&receiver, message);
        }
    }

    fn update_connections(&mut self) {
        let mut disconnected = Vec::new();
        let do_resend_messaged;

        if self.last_message_resend.elapsed() >= MESSAGE_RESEND_INTERVAL {
            self.last_message_resend = Instant::now();
            do_resend_messaged = true;
        } else {
            do_resend_messaged = false;
        }

        for (address, connection) in self.connections.iter_mut() {
            if do_resend_messaged {
                connection.send_unacknowledged_messages(&self.socket, &address);
            }

            if let ConnectionStatus::Disconnected(ref reason) = *connection.get_status() {
                disconnected.push(*address);
                log::warn!("{} disconnected. {}", address, reason);
            }
        }

        for address in disconnected.iter() {
            self.connections.remove(address);
        }
    }

    fn process_incoming_tasks(&mut self, tasks: &mut NetworkTaskResource) {
        for task in tasks.drain(..) {
            match task {
                NetworkTask::AttachPublicId(address, public_id) => {
                    if let Some(connection) = self.connections.get_mut(&address) {
                        connection.attached_public_id.replace(public_id);
                    }
                }
            }
        }
    }

    fn read_incoming_messages(&mut self, tasks: &mut GameTaskResource) {
        loop {
            let mut buffer = [0; MESSAGE_SIZE_MAX];

            match self.socket.recv_from(&mut buffer) {
                Ok((message_length, address)) => {
                    if !self.connections.contains_key(&address) {
                        log::info!("{} connected", address);
                    }

                    let message = buffer
                        .get(..message_length)
                        .ok_or_else(|| "Wrong message length".to_string())
                        .and_then(|m| Message::decode(m).map_err(|e| format!("{}", e)));

                    match message {
                        Ok(message) => {
                            let connection = self
                                .connections
                                .entry(address)
                                .or_insert_with(Connection::new);

                            if let Message::Response { message_id } = message {
                                connection.acknowledge_message(message_id);
                            } else {
                                if let Some(message_id) = message.get_id() {
                                    connection.send(
                                        &self.socket,
                                        &address,
                                        &mut Message::Response { message_id },
                                    );
                                }

                                if let Some(message) = connection.filter_message(message) {
                                    let public_id = connection.attached_public_id;
                                    let next_messages = connection.take_next_held_messages();

                                    self.on_message(&address, &message, public_id, tasks);

                                    for message in next_messages.iter() {
                                        self.on_message(&address, &message, public_id, tasks);
                                    }
                                }
                            }
                        }
                        Err(error) => {
                            self.connections.remove(&address);
                            log::warn!(
                                "{} disconnected. A corrupted message received. {}",
                                address,
                                error,
                            );
                            // TODO: Notify address
                        }
                    }
                }
                Err(error) => {
                    if error.kind() == ErrorKind::WouldBlock {
                        break;
                    } else {
                        log::error!("Failed to receive new messages. {}", error);
                        // TODO: Close connection
                    }
                }
            }
        }
    }

    fn on_message(
        &mut self,
        address: &SocketAddr,
        message: &Message,
        public_id: Option<u16>,
        tasks: &mut GameTaskResource,
    ) {
        if self.is_server {
            self.on_message_as_server(&address, &message, public_id, tasks);
        } else {
            self.on_message_as_client(&message, tasks);
        }
    }

    fn on_message_as_server(
        &mut self,
        address: &SocketAddr,
        message: &Message,
        public_id: Option<u16>,
        tasks: &mut GameTaskResource,
    ) {
        match *message {
            Message::Greeting { .. } => {
                tasks.push(GameTask::PlayerConnect(*address));
            }
            Message::ActorAction {
                move_x,
                move_y,
                angle,
                ..
            } => {
                if let Some(public_id) = public_id {
                    tasks.push(GameTask::ActorAction {
                        public_id,
                        move_x,
                        move_y,
                        angle,
                    });

                    self.send(
                        &MessageReceiver::Every,
                        Message::ActorAction {
                            id: 0,
                            public_id,
                            move_x,
                            move_y,
                            angle,
                        },
                    );
                } else {
                    // TODO: Kick
                }
            }
            _ => {
                // TODO: Kick
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn on_message_as_client(&self, message: &Message, tasks: &mut GameTaskResource) {
        match *message {
            Message::ActorSpawn {
                public_id,
                x,
                y,
                angle,
                ..
            } => {
                tasks.push(GameTask::ActorSpawn {
                    public_id,
                    x,
                    y,
                    angle,
                });
            }
            Message::ActorGrant { public_id, .. } => {
                tasks.push(GameTask::ActorGrant(public_id));
            }
            Message::ActorAction {
                public_id,
                move_x,
                move_y,
                angle,
                ..
            } => {
                tasks.push(GameTask::ActorAction {
                    public_id,
                    move_x,
                    move_y,
                    angle,
                });
            }
            Message::TransformSync {
                public_id, x, y, ..
            } => {
                tasks.push(GameTask::TransformSync { public_id, x, y });
            }
            _ => {}
        }
    }

    fn send(&mut self, receiver: &MessageReceiver, mut message: Message) {
        match *receiver {
            MessageReceiver::Only(ref address) => {
                if let Some(connection) = self.connections.get_mut(address) {
                    connection.send(&self.socket, address, &mut message);
                }
            }
            MessageReceiver::Every | MessageReceiver::Except(..) => {
                let excepted;

                if let MessageReceiver::Except(address) = *receiver {
                    excepted = Some(address);
                } else {
                    excepted = None;
                }

                for (address, connection) in self.connections.iter_mut() {
                    if Some(*address) != excepted {
                        connection.send(&self.socket, &address, &mut message);
                    }
                }
            }
        }
    }
}

impl<'a> System<'a> for NetworkSystem {
    type SystemData = (
        Write<'a, GameTaskResource>,
        Write<'a, MessageResource>,
        Write<'a, NetworkTaskResource>,
    );

    fn run(&mut self, (mut game_tasks, mut messages, mut network_tasks): Self::SystemData) {
        self.send_outgoing_messages(&mut messages);
        self.update_connections();
        self.process_incoming_tasks(&mut network_tasks);
        self.read_incoming_messages(&mut game_tasks);
    }
}
