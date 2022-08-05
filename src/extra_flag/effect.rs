use http::{HeaderMap, StatusCode};
use serde::Serialize;

use crate::{resp_body::RespBody, ExtraFlag, ExtraFlags, RespError, RespResult};

use super::flags::HeaderType;

#[derive(Debug)]
pub enum BodyEffect {
    Empty,
    Continue,
}


pub trait Effects {
    /// change the body
    /// return true allow following set json serde respond
    fn body_effect(&self, _: &mut Vec<u8>) -> BodyEffect {
        BodyEffect::Continue
    }
    /// return `Some` for cover resp-result StatusCode
    /// or return `None`
    fn status_effect(&self) -> Option<StatusCode> {
        None
    }
    /// adding header map
    fn headers_effect(&self, _: &mut HeaderMap) {}
}

impl Effects for ExtraFlags {
    fn body_effect(&self, body: &mut Vec<u8>) -> BodyEffect {
        if self.flags.iter().any(|flag| flag == &ExtraFlag::EmptyBody) {
            body.clear();
            BodyEffect::Empty
        } else {
            BodyEffect::Continue
        }
    }

    fn status_effect(&self) -> Option<StatusCode> {
        self.flags
            .iter()
            .filter_map(|flag| {
                if let ExtraFlag::SetStatus(status) = flag {
                    Some(status)
                } else {
                    None
                }
            })
            .reduce(|_, r| r)
            .copied()
    }

    fn headers_effect(&self, header_map: &mut HeaderMap) {
        self.flags
            .iter()
            .filter_map(|flag| {
                if let ExtraFlag::RemoveHeader(k) = flag {
                    Some(k)
                } else {
                    None
                }
            })
            .for_each(|k| {
                header_map.remove(k);
            });

        self.flags
            .iter()
            .filter_map(|flag| {
                if let ExtraFlag::SetHeader(k, v, ty) = flag {
                    Some((k, v.clone(), ty))
                } else {
                    None
                }
            })
            .for_each(|(k, v, ty)| match ty {
                HeaderType::Insert => {
                    header_map.insert(k, v);
                }
                HeaderType::Append => {
                    header_map.append(k, v);
                }
            })
    }
}

impl<T: Serialize> Effects for T {}

impl<T, E> Effects for RespResult<T, E>
where
    T: RespBody,
    E: RespError,
{
    fn body_effect(&self, body: &mut Vec<u8>) -> BodyEffect {
        match self {
            RespResult::Success(b) => b.body_effect(body),
            RespResult::Err(_) => BodyEffect::Continue,
        }
    }

    fn status_effect(&self) -> Option<StatusCode> {
        match self {
            RespResult::Success(b) => b.status_effect(),
            RespResult::Err(_) => None,
        }
    }

    fn headers_effect(&self, header_map: &mut HeaderMap) {
        if let RespResult::Success(b) = self {
            b.headers_effect(header_map)
        }
    }
}
