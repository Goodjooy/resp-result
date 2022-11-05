mod actix;
pub mod axum;

#[allow(unused_imports)]
use std::str::FromStr;

use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode};
#[cfg(feature = "tracing")]
use trace::{event, span, Level};

use super::{serde::SerializeWrap, RespResult};
use crate::{
    extra_flag::effect::{BodyEffect, Effects},
    get_config,
    resp_body::RespBody,
    resp_error::RespError,
};


#[allow(dead_code)]
static JSON_TYPE: &mime::Mime = &mime::APPLICATION_JSON;

#[derive(Debug)]
struct PrepareRespond {
    pub(crate) body: Vec<u8>,
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
}

impl PrepareRespond {
    #[allow(dead_code)]
    #[inline]
    pub fn from_resp_result<T, E>(resp: &RespResult<T, E>) -> Self
    where
        T: RespBody,
        E: RespError,
    {#[cfg(feature = "tracing")]
        let span = span!(Level::DEBUG, "preparation for Response");
        #[cfg(feature = "tracing")]
        let _enter = span.enter();

        let mut this = Self {
            body: Vec::new(),
            status: StatusCode::OK,
            headers: HeaderMap::new(),
        };

        #[allow(unused_variables)]
        let cfg = &get_config().resp;
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, prepare.state = "Set Payload");
        this.serde_body(resp);
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, prepare.state = "Set Status");
        this.set_status(resp);
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, prepare.state = "Set Headers");
        this.set_header(
            resp,
            #[cfg(feature = "extra-error")]
            cfg.extra_code.as_ref(),
        );
        #[cfg(feature = "tracing")]
        event!(
            Level::INFO,
            prepare.state = "Ready",
            response.status = %this.status,
            response.payload.length = %this.body.len()
        );

        this
    }

    #[allow(clippy::map_identity)]
    fn serde_body<T, E>(&mut self, resp: &RespResult<T, E>)
    where
        T: RespBody,
        E: RespError,
    {
        if let BodyEffect::Continue = resp.body_effect(&mut self.body) {
            #[cfg(feature = "tracing")]
            event!(Level::DEBUG, body.body_effect = "Continue");
            serde_json::to_writer(&mut self.body, &SerializeWrap(resp))
                .map_err(|err| {
                    #[cfg(feature = "tracing")]
                    event!(Level::ERROR, info = "Serialize Error", error = %err);
                    err
                })
                .expect("Json 响应时序列化异常");
        } else {
            #[cfg(feature = "tracing")]
            event!(Level::DEBUG, body.body_effect = "Empty");
        }
    }

    fn set_header<T, E>(
        &mut self,
        resp: &RespResult<T, E>,
        #[cfg(feature = "extra-error")] extra_header: Option<&http::header::HeaderName>,
    ) where
        T: RespBody,
        E: RespError,
    {
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, headers.content_type = %JSON_TYPE);
        self.headers.append(
            CONTENT_TYPE,
            HeaderValue::try_from(JSON_TYPE.as_ref()).expect("Bad HeaderValue"),
        );
        // extra header

        #[cfg(feature = "extra-error")]
        {
            #[cfg(feature = "tracing")]
            event!(Level::DEBUG, headers.extra_header = ?extra_header);
            match (resp, extra_header) {
                (RespResult::Success(_), _) | (_, None) => (),
                (RespResult::Err(err), Some(key)) => {
                    self.headers.append(
                        key,
                        HeaderValue::from_str(&err.extra_message().to_string())
                            .expect("Bad HeaderValue"),
                    );
                }
            }
        }
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, "Apply Header Effect");
        resp.headers_effect(&mut self.headers);
    }

    fn set_status<T, E>(&mut self, resp: &RespResult<T, E>)
    where
        T: RespBody,
        E: RespError,
    {
        // status code
        let status = match resp {
            RespResult::Success(_) => {
                #[cfg(feature = "tracing")]
                event!(
                    Level::INFO,
                    resp_result = "RespResult::Success",
                    status = %StatusCode::OK
                );
                StatusCode::OK
            }
            RespResult::Err(ref e) => {
                #[cfg(feature = "tracing")]
                event!(
                    Level::DEBUG,
                    result = "RespResult::Err",
                    status = %e.http_code(),
                    error = %e.log_message()
                );

                e.http_code()
            }
        };
        #[cfg(feature = "tracing")]
        event!(Level::DEBUG, "Apply Status Effect");
        self.status = resp.status_effect().unwrap_or(status)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct Nil;

impl From<()> for Nil {
    fn from(_: ()) -> Self {
        Self
    }
}

impl From<std::convert::Infallible> for Nil {
    fn from(_: std::convert::Infallible) -> Self {
        Self
    }
}

impl std::fmt::Display for Nil {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Nil")
    }
}

#[cfg(test)]
mod test {
    use http::StatusCode;

    use crate::{ExtraFlag, RespError, RespResult};

    use super::PrepareRespond;
    struct MockErr;

    impl RespError for MockErr {
        fn log_message(&self) -> std::borrow::Cow<'_, str> {
            "Mock Error".into()
        }
        #[cfg(feature = "extra-error")]
        type ExtraMessage = String;
        #[cfg(feature = "extra-error")]
        fn extra_message(&self) -> Self::ExtraMessage {
            "Mock".into()
        }
    }
    #[test]
    fn test_prepare_resp() {
        let a = RespResult::<_, MockErr>::Success(12i32).with_flags(
            ExtraFlag::EmptyBody
                + ExtraFlag::status(StatusCode::NOT_MODIFIED)
                + ExtraFlag::insert_header(http::header::ETAG, "1234567890")
                + ExtraFlag::remove_header(http::header::CONTENT_TYPE),
        );

        let p = PrepareRespond::from_resp_result(&a);

        assert_eq!(p.body.len(), 0);
        assert_eq!(p.status, StatusCode::NOT_MODIFIED);
        assert_eq!(p.headers.len(), 1);
        println!("{p:#?}")
    }
}
