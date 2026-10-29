// Requires the system-wide tdjson library, which is not linked with the `docs` feature
#![cfg(not(feature = "docs"))]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tdlib_rs::{enums, functions};

#[tokio::test]
async fn client() {
    let client_id = tdlib_rs::create_client();

    let running = Arc::new(AtomicBool::new(true));
    let receiver = std::thread::spawn({
        let running = running.clone();
        move || {
            let mut updates = Vec::new();
            while running.load(Ordering::Relaxed) {
                if let Some((update, update_client_id)) = tdlib_rs::receive() {
                    assert_eq!(update_client_id, client_id);
                    updates.push(update);
                }
            }
            updates
        }
    });

    // A function returning `Ok`
    functions::set_log_verbosity_level(0, client_id)
        .await
        .unwrap();

    // A function returning an object
    let version = functions::get_option("version".into(), client_id)
        .await
        .unwrap();
    let enums::OptionValue::String(version) = version else {
        panic!("unexpected version: {version:?}");
    };
    assert!(version.value.starts_with("1.8."), "{}", version.value);

    // A function returning an error, since the client isn't authorized
    let error = functions::get_me(client_id).await.unwrap_err();
    assert!(error.code > 0);
    assert!(!error.message.is_empty());

    running.store(false, Ordering::Relaxed);
    let updates = receiver.join().unwrap();
    assert!(updates.iter().any(|update| matches!(
        update,
        enums::Update::AuthorizationState(state)
            if state.authorization_state == enums::AuthorizationState::WaitTdlibParameters
    )));
}
