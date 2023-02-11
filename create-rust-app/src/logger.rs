use poem::{async_trait, Endpoint, IntoResponse, Middleware, Request, Response, Result};

#[deprecated]
/// Logger middleware that provides similar functionality as [`actix_web::middleware::Logger`]
/// for the poem backend
pub struct Logger;

#[allow(deprecated)]
impl<E: Endpoint> Middleware<E> for Logger {
    type Output = LogImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        LogImpl(ep)
    }
}

#[deprecated]
pub struct LogImpl<E>(E);

#[allow(deprecated)]
#[async_trait]
impl<E: Endpoint> Endpoint for LogImpl<E> {
    type Output = Response;

    /// Logs requests recieved by the server, as well as the associated responses
    async fn call(&self, req: Request) -> Result<Self::Output> {
        println!(">  REQUEST: {}", req.uri().path());
        let res = self.0.call(req).await;

        match res {
            Ok(resp) => {
                let resp = resp.into_response();
                println!("< RESPONSE: {}", resp.status());
                Ok(resp)
            }
            Err(err) => {
                println!("<  ERROR: {}", err);
                Err(err)
            }
        }
    }
}
