use futures::future;
use ntex::{fn_factory_with_config, fn_service, ServiceFactory};

use crate::{error::Error, hub::Hub};

pub fn telemetry(
) -> impl ServiceFactory<Config = Hub, InitError = Error, Request = String, Response = (), Error = Error>
{
    fn_factory_with_config(|hub: Hub| {
        let hub = hub.clone();
        future::ok::<_, Error>(fn_service(move |msg: String| {
            println!("[{}] Message: {}", hub.message_count(), msg);
            hub.inc_message_count();

            future::ok::<_, Error>(())
        }))
    })
}
