use futures::Future;
use warp::path::FullPath;
use warp::reject::not_found;
use warp::reply::Response;
use warp::{Filter, Rejection};

use crate::frontend::IndexHtml;
use crate::html;
use crate::request::WarpRequest;

/// A filter that extracts the warp request.
pub(crate) fn warp_request(
    index_html: IndexHtml,
    auto_refresh: bool,
) -> impl Clone
       + Send
       + Filter<
    Extract = (WarpRequest<()>,),
    Error = Rejection,
    Future = impl Future<Output = Result<(WarpRequest<()>,), Rejection>>,
> {
    warp::path::full()
        .and(warp::query::raw().or_else(|_| async move { Ok::<_, Rejection>((String::new(),)) }))
        .and(warp::header::headers_cloned())
        .then(move |path: FullPath, raw_queries: String, headers| {
            let index_html = index_html.clone();
            async move {
                let mut template = index_html.read_content().await;

                if auto_refresh {
                    template = html::add_refresh_script(&template).into();
                }

                WarpRequest {
                    path: path.into(),
                    raw_queries: raw_queries.into(),
                    template,
                    context: ().into(),
                    headers,
                }
            }
        })
}

pub(crate) fn reject() -> impl Clone
       + Send
       + Filter<
    Extract = (Response,),
    Error = Rejection,
    Future = impl Future<Output = Result<(Response,), Rejection>>,
> {
    warp::path::end().and_then(|| async move { Err::<Response, Rejection>(not_found()) })
}
