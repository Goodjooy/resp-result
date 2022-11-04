use std::future::Future;

use crate::{RespError, RespResult};

/// convert into [`RespResult`](crate::RespResult)
pub trait IntoRespResult<T, E: RespError> {
    fn into_rresult(self) -> RespResult<T, E>;
}

/// convert into [`RespResult`](crate::RespResult) with provide error
pub trait IntoRespResultWithErr<T, E: RespError> {
    fn into_with_err<Et: Into<E>>(self, err: Et) -> RespResult<T, E>;
}

impl<E, T> IntoRespResult<T, E> for Result<T, E>
where
    E: RespError,
{
    #[inline]
    fn into_rresult(self) -> RespResult<T, E> {
        RespResult::from(self.map_err(|e| e.into()))
    }
}

impl< E, T> IntoRespResult<T, E> for RespResult<T, E>
where
    E: RespError,
{
    fn into_rresult(self) -> RespResult<T, E> {
        self.map_err(Into::into)
    }
}

impl<T, E> IntoRespResultWithErr<T, E> for Option<T>
where
    E: RespError,
{
    #[inline]
    fn into_with_err<Et: Into<E>>(self, err: Et) -> RespResult<T, E> {
        self.ok_or(err).map_err(Into::into).into_rresult()
    }
}

pub async fn resp_try<Fut, T, E>(future: Fut) -> RespResult<T, E>
where
    Fut: Future,
    Fut::Output: IntoRespResult<T, E>,
    E: RespError,
{
    future.await.into_rresult()
}
