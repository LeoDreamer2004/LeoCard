use super::socket;
use crate::network::handshake::{Handshake, validate_message};
use crate::network::state::{
    CONNECTION_HEARTBEAT_TIMEOUT, EVENT_QUEUE, EventQueue, NETWORK_ROOM_ID, NetworkEvent,
    NetworkLaunch, NetworkStartError, PING_INTERVAL, RECONNECT_ATTEMPTS, RECONNECT_CONNECT_TIMEOUT,
    push_event,
};
use js_sys::Uint8Array;
use leocard_protocol::{
    ClientCommand, ClientMessage, MAX_FRAME_PAYLOAD, RequestId, ServerEvent, ServerMessage,
    decode_frame, encode_frame,
};
use serde_json::json;
use wasm_bindgen::prelude::*;
use web_sys::Url;

pub(in crate::network) struct CommandSender(u32);

impl CommandSender {
    pub(in crate::network) fn try_send(&self, message: ClientMessage) -> Result<(), ()> {
        let frame = encode_frame(&message).map_err(|_| ())?;
        socket::send(self.0, &frame).then_some(()).ok_or(())
    }
}

pub(in crate::network) struct WorkerHandle(u32);

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        socket::stop(self.0);
    }
}

pub(in crate::network) fn normalize_address(address: &str) -> Result<String, NetworkStartError> {
    let address = address.trim();
    let invalid = || NetworkStartError::InvalidAddress(address.to_owned());
    let url = Url::new(address).map_err(|_| invalid())?;
    if !matches!(url.protocol().as_str(), "ws:" | "wss:")
        || url.hostname().is_empty()
        || !url.username().is_empty()
        || !url.password().is_empty()
        || !url.hash().is_empty()
    {
        return Err(invalid());
    }
    if web_sys::window()
        .is_some_and(|window| window.location().protocol().ok().as_deref() == Some("https:"))
        && url.protocol() != "wss:"
    {
        return Err(NetworkStartError::Unsupported(
            "HTTPS 网页需要使用 wss:// 加密地址",
        ));
    }
    Ok(url.href())
}

pub(in crate::network) fn spawn_worker(
    launch: NetworkLaunch,
    events: EventQueue,
    join: ClientMessage,
) -> Result<(CommandSender, WorkerHandle), NetworkStartError> {
    let NetworkLaunch::Join { address } = launch;
    let mut handshake = Handshake::default();
    let mut ready = false;
    let connected_address = address.clone();
    let callback = Closure::wrap(Box::new(move |kind: u8, value: JsValue| -> u8 {
        if kind == 4 {
            push_event(&events, NetworkEvent::Resync);
            return 0;
        }
        if kind == 1 {
            ready = false;
            handshake = Handshake::default();
            push_event(
                &events,
                NetworkEvent::Reconnecting(value.as_string().unwrap_or_default()),
            );
            return 0;
        }
        if kind == 2 {
            push_event(
                &events,
                NetworkEvent::Failed(
                    value
                        .as_string()
                        .unwrap_or_else(|| "WSS 连接失败".to_owned()),
                ),
            );
            return 2;
        }
        {
            let mut queue = events.lock().expect("network event queue mutex poisoned");
            if queue.len() >= EVENT_QUEUE - 4 {
                queue.clear();
                queue.push_back(NetworkEvent::Reconnecting(
                    "正在重新同步房间状态".to_owned(),
                ));
                return 3;
            }
        }
        let bytes = Uint8Array::new(&value).to_vec();
        let result = (|| {
            let message: ServerMessage = decode_frame(&bytes).map_err(|error| error.to_string())?;
            validate_message(&message)?;
            let joined = !ready && handshake.accept(&message)?;
            let finished = matches!(
                message.event,
                ServerEvent::RoomClosed | ServerEvent::LeftRoom | ServerEvent::KickedFromRoom
            );
            if !matches!(message.event, ServerEvent::Heartbeat) {
                push_event(&events, NetworkEvent::Message(Box::new(message)));
            }
            if joined {
                ready = true;
                push_event(
                    &events,
                    NetworkEvent::Connected(format!("已连接 {connected_address}")),
                );
            }
            Ok::<_, String>(if finished {
                2
            } else if joined {
                1
            } else {
                0
            })
        })();
        match result {
            Ok(status) => status,
            Err(error) => {
                push_event(&events, NetworkEvent::Failed(error));
                2
            }
        }
    }) as Box<dyn FnMut(u8, JsValue) -> u8>);
    let encode = |message: &ClientMessage| {
        encode_frame(message).map_err(|_| NetworkStartError::Unsupported("无法编码网络指令"))
    };
    let ping = ClientMessage::new(NETWORK_ROOM_ID, RequestId(0), ClientCommand::Ping);
    let settings = json!({
        "attempts": RECONNECT_ATTEMPTS,
        "token": match &join.command { ClientCommand::Join(request) => request.reconnect_token.0.to_string(), _ => String::new() },
        "heartbeat": CONNECTION_HEARTBEAT_TIMEOUT.as_millis(),
        "connectTimeout": RECONNECT_CONNECT_TIMEOUT.as_millis(),
        "pingInterval": PING_INTERVAL.as_millis(),
        "maxFrame": MAX_FRAME_PAYLOAD + 4,
    })
    .to_string();
    let id = socket::start(
        &address,
        &encode(&join)?,
        &encode(&ping)?,
        callback.as_ref().unchecked_ref(),
        &settings,
    )
    .map_err(|_| NetworkStartError::Unsupported("无法创建 WebSocket，请检查地址格式"))?;
    let _ = callback.into_js_value();
    Ok((CommandSender(id), WorkerHandle(id)))
}

pub(in crate::network) fn remembered_reconnect_token(launch: &NetworkLaunch) -> Option<u64> {
    let NetworkLaunch::Join { address } = launch;
    socket::remembered_token(address)?
        .parse()
        .ok()
        .filter(|token| *token != 0)
}

pub(in crate::network) fn forget_session() {
    socket::forget_room();
}
