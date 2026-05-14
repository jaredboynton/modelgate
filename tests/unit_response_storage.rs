mod common;

use serde_json::json;
use unified_model_proxy_v2::state::NewResponseStateRecord;

#[test]
fn store_false_continuation_state_is_not_publicly_retrievable() {
    let homes = common::TestHomes::new();

    homes
        .state
        .remember_response_for_continuation(response_record(
            "resp_ump_codex_store_false",
            "resp_upstream_store_false",
            json!({
                "id": "resp_ump_codex_store_false",
                "store": false,
                "output": [{ "id": "item_1", "type": "message" }]
            }),
        ));

    let continuation = homes
        .state
        .continuation_response("resp_ump_codex_store_false")
        .expect("store:false response remains available for internal continuation");

    assert!(!continuation.public_retrievable);
    assert_eq!(
        continuation.upstream_response_id,
        "resp_upstream_store_false"
    );
    assert!(
        homes
            .state
            .public_response("resp_ump_codex_store_false")
            .is_none(),
        "retrieve route should map this None to a 404"
    );
    assert!(
        homes
            .state
            .public_input_items("resp_ump_codex_store_false")
            .is_none(),
        "input_items route should map this None to a 404"
    );
}

#[test]
fn store_true_records_are_adapter_owned_public_storage() {
    let homes = common::TestHomes::new();

    homes.state.store_public_response(response_record(
        "resp_ump_codex_store_true",
        "resp_upstream_store_true",
        json!({
            "id": "resp_ump_codex_store_true",
            "store": true,
            "output": [{ "id": "item_2", "type": "message" }]
        }),
    ));

    let continuation = homes
        .state
        .continuation_response("resp_ump_codex_store_true")
        .expect("store:true response remains available for continuation");
    let retrieved = homes
        .state
        .public_response("resp_ump_codex_store_true")
        .expect("store:true response is public retrievable");
    let input_items = homes
        .state
        .public_input_items("resp_ump_codex_store_true")
        .expect("store:true input items are public retrievable");

    assert!(continuation.public_retrievable);
    assert_eq!(retrieved["id"], "resp_ump_codex_store_true");
    assert_eq!(input_items["data"][0]["id"], "item_2");
}

#[test]
fn unknown_response_ids_are_not_found_in_either_store() {
    let homes = common::TestHomes::new();

    assert!(homes.state.continuation_response("resp_missing").is_none());
    assert!(homes.state.public_response("resp_missing").is_none());
    assert!(homes.state.public_input_items("resp_missing").is_none());
}

fn response_record(
    adapter_response_id: &str,
    upstream_response_id: &str,
    raw_response: serde_json::Value,
) -> NewResponseStateRecord {
    NewResponseStateRecord {
        route: "responses".to_string(),
        provider: "codex".to_string(),
        upstream_model: "gpt-5.5".to_string(),
        upstream_response_id: upstream_response_id.to_string(),
        adapter_response_id: adapter_response_id.to_string(),
        conversation_id: None,
        raw_response,
        raw_input_items: json!({
            "object": "list",
            "data": [{ "id": "item_2", "type": "message" }]
        }),
        upstream_codex_minted: true,
    }
}
