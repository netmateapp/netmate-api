use std::sync::Arc;

use scylla::Session;

use crate::{helper::redis::Pool, middlewares::start_session::dsl::start_session::StartSession};

pub struct StartSessionImpl {
    db: Arc<Session>,
    cache: Arc<Pool>,
}

impl StartSession for StartSessionImpl {}