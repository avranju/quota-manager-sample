use std::collections::HashMap;

use hub::Hub;
use ntex::{rt::System, Service, ServiceFactory};
use quota::{QuotaManager, QuotaService};
use telemetry::telemetry;

mod error;
mod hub;
mod quota;
mod telemetry;

fn main() {
    System::new("ntex-transform").block_on(async {
        println!("System running.");

        let mut count_quota = HashMap::new();
        count_quota.insert("hub1".to_string(), 3u64);
        count_quota.insert("hub2".to_string(), 5u64);
        count_quota.insert("hub3".to_string(), 50u64);
        let quota_manager = QuotaManager::new(count_quota);

        let hub1 = Hub::new("hub1".to_string());
        let factory = telemetry();
        let tel = factory.new_service(hub1.clone()).await.unwrap();

        let quota_service = QuotaService::new(tel, hub1.clone(), quota_manager);

        // send 3 messages - which should be allowed
        for i in 0..3 {
            quota_service.call(format!("m{}", i)).await.unwrap();
        }

        // the 4th message should exceed quota
        let res = quota_service.call("m3".to_string()).await;
        println!("{:?}", res);

        hub1.print();
    });
}
