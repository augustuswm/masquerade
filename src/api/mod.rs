use bodyparser;
use iron::Chain;
use persistent::Read;

use error::BannerError;
use flag::Flag;
use store::ThreadedStore;

mod api;
mod backend;
pub mod error;
mod flag;
mod status;

const MAX_BODY_LENGTH: usize = 1024 * 1024 * 10;

pub fn v1<T: ThreadedStore<Item = Flag, Error = BannerError> + 'static>(store: T) -> Chain {
    let mut chain = Chain::new(api::v1());
    chain.link_before(backend::BackendMiddleware::new(store));
    chain.link_before(Read::<bodyparser::MaxBodyLength>::one(MAX_BODY_LENGTH));
    chain
}
