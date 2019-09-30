pub mod method;
pub mod serde_as_scale;

#[cfg(test)]
mod testnode;

use futures::{future::Future, sink::Sink, stream::Stream};
use jrpc::{Id, Request, Response, V2_0};
use method::Method;
use serde::{de::DeserializeOwned, Serialize};
use serde_as_scale::SerdeAsScale;
use websocket::{ClientBuilder, OwnedMessage};

/// ```
/// # use client::remote_call;
/// # use client::method::StateGetMetadata;
/// # use srml_metadata::RuntimeMetadataPrefixed;
/// let _: RuntimeMetadataPrefixed = remote_call::<StateGetMetadata>("ws://127.0.0.1:9944", []).unwrap();
/// ```
pub fn remote_call<M: Method>(url: &str, arg: M::Args) -> Result<M::Return, &'static str> {
    let ret: SerdeAsScale<M::Return> = get_sumpm(url, M::NAME, arg)?;
    Ok(ret.0)
}

fn get_sumpm<Return: DeserializeOwned>(
    url: &str,
    method: impl Serialize,
    params: impl Serialize,
) -> Result<Return, &'static str> {
    let req = Request {
        jsonrpc: V2_0,
        method: method,
        params: Some(params),
        id: Id::Int(rand::random::<i64>().abs()).into(),
    };

    let (duplex, _headers) = ClientBuilder::new(url)
        .unwrap()
        .async_connect_insecure()
        .wait()
        .map_err(|e| {
            use websocket::result::WebSocketError as Wse;
            match e {
                Wse::DataFrameError(s)
                | Wse::ProtocolError(s)
                | Wse::RequestError(s)
                | Wse::ResponseError(s) => s,
                _ => "couldn't connect",
            }
        })?;

    let duplex = duplex
        .send(OwnedMessage::Text(
            serde_json::to_string(&req).map_err(|_| "serialization failure")?,
        ))
        .wait()
        .map_err(|_| "couldn't send")?;

    let (next_message, _duplex) = Stream::into_future(duplex)
        .wait()
        .map_err(|_| "couldn't receive")?;

    let text: String = match next_message {
        Some(OwnedMessage::Text(s)) => s,
        _ => return Err("received non-text packet"),
    };

    let resp: Response<Return> = serde_json::from_str(&text).map_err(|_| "got invalid json")?;
    let success = match resp {
        Response::Ok(success) => success,
        Response::Err(_) => Err("recieved rpc err response")?,
    };

    Ok(success.result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use method::StateGetMetadata;
    use testnode::RunningFullNode;

    #[test]
    fn g_metadata() {
        RunningFullNode::new()
            .remote_call::<StateGetMetadata>([])
            .unwrap();
    }

    #[test]
    fn metadata_same() {
        // Get metadata from server, assert it is equal to metadata exported by runtime.
        let remote_metadata = RunningFullNode::new()
            .remote_call::<StateGetMetadata>([])
            .unwrap();
        let expected_metadata = node_template_runtime::Runtime::metadata();
        assert_eq!(remote_metadata, expected_metadata);
    }
}
