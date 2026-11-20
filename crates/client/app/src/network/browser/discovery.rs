use super::connection::normalize_address;
use super::socket;
use crate::network::handshake::validate_message;
use crate::network::state::{NETWORK_ROOM_ID, RoomInfoResult};
use js_sys::Uint8Array;
use leocard_platform::time::Instant;
use leocard_protocol::{
    ClientCommand, ClientMessage, RequestId, RoomInfo, ServerEvent, ServerMessage, decode_frame,
    encode_frame,
};
use std::io;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver};
use wasm_bindgen::prelude::*;

pub(crate) struct RoomInfoQueries {
    receiver: Mutex<Receiver<RoomInfoResult>>,
    pending: Vec<u32>,
}

impl RoomInfoQueries {
    pub fn start(addresses: Vec<String>) -> io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let mut pending = Vec::new();
        let request = encode_frame(&ClientMessage::new(
            NETWORK_ROOM_ID,
            RequestId(0),
            ClientCommand::RequestRoomInfo,
        ))
        .map_err(io::Error::other)?;
        for address in addresses {
            let Ok(url) = normalize_address(&address) else {
                let _ = sender.send(RoomInfoResult {
                    address,
                    info: None,
                });
                continue;
            };
            let started = Instant::now();
            let replies = sender.clone();
            let queried = address.clone();
            let callback = Closure::wrap(Box::new(move |value: JsValue| {
                let info = if value.is_null() {
                    None
                } else {
                    let bytes = Uint8Array::new(&value).to_vec();
                    decode_frame::<ServerMessage>(&bytes)
                        .ok()
                        .filter(|message| validate_message(message).is_ok())
                        .and_then(|message| match message.event {
                            ServerEvent::RoomInfo(info) => info.map(|info| {
                                (
                                    info,
                                    started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32,
                                )
                            }),
                            _ => None,
                        })
                };
                let _ = replies.send(RoomInfoResult {
                    address: queried.clone(),
                    info,
                });
            }) as Box<dyn FnMut(JsValue)>);
            match socket::query(&url, &request, callback.as_ref().unchecked_ref()) {
                Ok(id) => {
                    pending.push(id);
                    let _ = callback.into_js_value();
                }
                Err(_) => {
                    let _ = sender.send(RoomInfoResult {
                        address,
                        info: None,
                    });
                }
            }
        }
        Ok(Self {
            receiver: Mutex::new(receiver),
            pending,
        })
    }

    pub fn poll(&self) -> Vec<RoomInfoResult> {
        self.receiver
            .lock()
            .expect("room query results")
            .try_iter()
            .collect()
    }
}

impl Drop for RoomInfoQueries {
    fn drop(&mut self) {
        for id in &self.pending {
            socket::stop(*id);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanRoom {
    pub server_id: u64,
    pub address: SocketAddr,
    pub info: RoomInfo,
}

/// The shared room picker never starts discovery when HAS_LAN is false.
pub struct LanRoomBrowser;

impl LanRoomBrowser {
    pub fn start() -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "网页端不支持局域网发现",
        ))
    }

    pub fn rooms(&self) -> &[LanRoom] {
        &[]
    }

    pub fn error(&self) -> Option<&str> {
        None
    }

    pub fn poll(&mut self) -> bool {
        false
    }
}
