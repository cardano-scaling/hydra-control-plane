use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{body::Incoming, Request, Response};

pub type Body = BoxBody<Bytes, hyper::Error>;
pub type ProxyResponse = Response<Body>;

pub fn full<T: Into<Bytes>>(chunk: T) -> Body {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub fn get_header(req: &Request<Incoming>, key: &str) -> Option<String> {
    req.headers()
        .get(key)
        .and_then(|h| h.to_str().ok().map(|v| v.to_string()))
}
