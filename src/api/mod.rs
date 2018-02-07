use iron::Chain;

use error::BannerError;
use flag::Flag;
use store::ThreadedStore;

mod api;
mod app;
mod backend;
mod env;
mod flag;
mod status;

pub fn v1<T: ThreadedStore<Item = Flag, Error = BannerError> + 'static>(store: T) -> Chain {
    let mut chain = Chain::new(api::v1());
    chain.link_before(backend::BackendMiddleware::new(store));
    chain
}
