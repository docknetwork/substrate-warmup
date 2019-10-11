extern crate alloc;

use crate::method::Method;
use core::fmt::{Debug, Display};
use futures::{future::Future, sink::Sink, stream::Stream};
use jrpc::{Id, Request, Response, V2_0};
use serde::{de::DeserializeOwned, Serialize};
use websocket::{ClientBuilder, OwnedMessage};

pub fn call<M: Method>(url: &str, arg: M::Args) -> Result<M::Return, String> {
    get_sumpm(url, M::NAME, arg).map_err(context(M::NAME))
}

fn get_sumpm<Return: DeserializeOwned>(
    url: &str,
    method: impl Serialize,
    params: impl Serialize,
) -> Result<Return, String> {
    let req = Request {
        jsonrpc: V2_0,
        method: method,
        params: Some(params),
        id: Id::Int(rand::random::<i64>().abs()).into(),
    };

    let request = serde_json::to_string(&req).map_err(context("serializing rpc call"))?;

    let (duplex, _headers) = ClientBuilder::new(url)
        .unwrap()
        .async_connect_insecure()
        .wait()
        .map_err(context("building ws rpc client"))?;

    let duplex = duplex
        .send(OwnedMessage::Text(request))
        .wait()
        .map_err(context("sending rpc call"))?;

    let (next_message, _duplex) = Stream::into_future(duplex)
        .wait()
        .map_err(|_| "failed to receive rpc response")
        .map_err(context("receiving rpc response"))?;

    let text: String = match next_message {
        Some(OwnedMessage::Text(s)) => s,
        _ => Err("received non-text packet").map_err(context("reading rpc response"))?,
    };

    let respres = serde_json::from_str(&text).map_err(context("deserializing rpc response"));
    let resp: Response<Return> = respres?;

    let respid = match &resp {
        Response::Ok(o) => o.id.clone(),
        Response::Err(e) => e.id.clone(),
    };
    if req.id != respid.into() {
        Err("rpc request id mismatch")?;
    }

    let success = match resp {
        Response::Ok(success) => success,
        Response::Err(e) => Err(e).map_err(context("checking rpc response"))?,
    };

    Ok(success.result)
}

fn context<D: Debug>(context: impl Display) -> impl (Fn(D) -> String) {
    move |a| format!("while {}, got {:?}", context, a)
}
