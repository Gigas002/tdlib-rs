// Copyright 2020 - developers of the `grammers` project.
// Copyright 2021 - developers of the `tdlib-rs` project.
// Copyright 2024 - developers of the `tgt` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use futures_channel::oneshot;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

pub(super) struct Observer {
    requests: RwLock<HashMap<u32, oneshot::Sender<Value>>>,
}

impl Observer {
    pub fn new() -> Self {
        Observer {
            requests: RwLock::default(),
        }
    }

    pub fn subscribe(&self, extra: u32) -> oneshot::Receiver<Value> {
        let (sender, receiver) = oneshot::channel();
        self.requests.write().unwrap().insert(extra, sender);
        receiver
    }

    pub fn notify(&self, response: Value) {
        let extra = response["@extra"].as_u64().unwrap() as u32;
        match self.requests.write().unwrap().remove(&extra) {
            Some(sender) => {
                if sender.send(response).is_err() {
                    log::warn!("Got a response of an inaccessible request");
                }
            }
            None => {
                log::warn!("Got a response of an unknown request");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn notify_subscriber() {
        let observer = Observer::new();
        let mut receiver = observer.subscribe(1);
        assert_eq!(receiver.try_recv(), Ok(None));

        let response = json!({"@type": "ok", "@extra": 1});
        observer.notify(response.clone());
        assert_eq!(receiver.try_recv(), Ok(Some(response)));
        assert!(observer.requests.read().unwrap().is_empty());
    }

    #[test]
    fn notify_only_matching_subscriber() {
        let observer = Observer::new();
        let mut first = observer.subscribe(1);
        let mut second = observer.subscribe(2);

        observer.notify(json!({"@extra": 2}));
        assert_eq!(first.try_recv(), Ok(None));
        assert_eq!(second.try_recv(), Ok(Some(json!({"@extra": 2}))));
    }

    #[test]
    fn notify_unknown_request() {
        let observer = Observer::new();
        let mut receiver = observer.subscribe(1);

        observer.notify(json!({"@extra": 2}));
        assert_eq!(receiver.try_recv(), Ok(None));
        assert_eq!(observer.requests.read().unwrap().len(), 1);
    }

    #[test]
    fn notify_dropped_subscriber() {
        let observer = Observer::new();
        drop(observer.subscribe(1));

        observer.notify(json!({"@extra": 1}));
        assert!(observer.requests.read().unwrap().is_empty());
    }
}
