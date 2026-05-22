use unified_model_proxy_v2::sse::{
    filter::filter_codex_events,
    splice::{splice_completed_event, splice_completed_event_filtered},
};

#[test]
fn unit_sse_drops_codex_events_and_keeps_response_events() {
    let input =
        "id: 1\nevent: codex.debug\ndata: nope\n\nevent: response.output_text.delta\ndata: ok\n\n";
    assert_eq!(
        filter_codex_events(input),
        "event: response.output_text.delta\ndata: ok\n\n"
    );
}

#[test]
fn unit_sse_splices_output_item_done_items_into_response_completed() {
    let input = concat!(
        "event: response.output_text.delta\n",
        "data: {\"delta\":\"hello\"}\n",
        "\n",
        "event: response.output_item.done\n",
        "data: {\"item\":{\"id\":\"call_1\",\"type\":\"function_call\",\"name\":\"lookup\",\"arguments\":\"{}\"}}\n",
        "\n",
        "event: response.completed\n",
        "data: {\"response\":{\"id\":\"resp_1\",\"status\":\"completed\"}}\n",
        "\n",
    );

    let spliced = splice_completed_event(input);
    assert!(spliced.contains("event: response.output_item.done"));
    assert!(spliced.contains("event: response.completed"));

    let completed_data = spliced
        .split("\n\n")
        .find(|block| block.starts_with("event: response.completed"))
        .and_then(|block| {
            block
                .lines()
                .find_map(|line| line.strip_prefix("data: "))
                .map(ToOwned::to_owned)
        })
        .unwrap();
    let completed: serde_json::Value = serde_json::from_str(&completed_data).unwrap();
    assert_eq!(completed["response"]["output"][0]["id"], "call_1");
    assert_eq!(completed["response"]["output"][0]["type"], "function_call");
}

#[test]
fn unit_sse_filtered_splice_drops_codex_events_in_same_pass() {
    let input = concat!(
        "event: codex.debug\n",
        "data: nope\n",
        "\n",
        "event: response.output_item.done\n",
        "data: {\"item\":{\"id\":\"call_1\",\"type\":\"function_call\"}}\n",
        "\n",
        "event: response.completed\n",
        "data: {\"response\":{\"id\":\"resp_1\"}}\n",
        "\n",
    );

    let spliced = splice_completed_event_filtered(input);

    assert!(!spliced.contains("codex.debug"));
    assert!(spliced.contains("response.output_item.done"));
    assert!(spliced.contains(r#""output":[{"id":"call_1""#));
}

#[test]
fn unit_sse_leaves_response_completed_unchanged_without_collected_items() {
    let input = "event: response.completed\ndata: {\"response\":{\"id\":\"resp_1\"}}\n\n";
    assert_eq!(splice_completed_event(input), input);
}
