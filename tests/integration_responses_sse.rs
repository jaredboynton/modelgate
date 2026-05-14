use axum::body::Bytes;
use unified_model_proxy_v2::adapter::responses_sse::ResponsesSseParser;

#[test]
fn responses_sse_parser_yields_json_frames_across_chunk_boundaries() {
    let mut parser = ResponsesSseParser::new();

    let first = parser
        .push_bytes(Bytes::from_static(
            b": heartbeat\n\nevent: response.created\ndata: {\"type\":\"response.created\",",
        ))
        .unwrap();
    assert!(first.is_empty());

    let second = parser
        .push_bytes(Bytes::from_static(
            b"\"response\":{\"id\":\"resp_1\"}}\n\nevent: response.completed\ndata: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_1\"}}\n\n",
        ))
        .unwrap();

    assert_eq!(second.len(), 2);
    assert_eq!(second[0].event.as_deref(), Some("response.created"));
    assert_eq!(second[0].data["type"], "response.created");
    assert_eq!(second[1].data["type"], "response.completed");
    parser.finish().unwrap();
}

#[test]
fn responses_sse_parser_joins_multiline_data_and_ignores_done_after_completed() {
    let mut parser = ResponsesSseParser::new();
    let frames = parser
        .push_bytes(Bytes::from_static(
            b"event: response.completed\ndata: {\"type\":\"response.completed\",\ndata: \"response\":{\"id\":\"resp_1\"}}\n\ndata: [DONE]\n\n",
        ))
        .unwrap();

    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].data["response"]["id"], "resp_1");
    parser.finish().unwrap();
}

#[test]
fn responses_sse_parser_rejects_done_before_completed() {
    let mut parser = ResponsesSseParser::new();
    let error = parser
        .push_bytes(Bytes::from_static(b"data: [DONE]\n\n"))
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("[DONE] before response.completed"));
}

#[test]
fn responses_sse_parser_rejects_invalid_utf8_and_json() {
    let mut parser = ResponsesSseParser::new();
    let error = parser
        .push_bytes(Bytes::from_static(b"data: \xFF\n\n"))
        .unwrap_err();
    assert!(error.to_string().contains("invalid Responses SSE UTF-8"));

    let mut parser = ResponsesSseParser::new();
    let error = parser
        .push_bytes(Bytes::from_static(b"data: {not-json}\n\n"))
        .unwrap_err();
    assert!(error.to_string().contains("json error"));
}

#[test]
fn responses_sse_parser_rejects_eof_before_completed() {
    let mut parser = ResponsesSseParser::new();
    parser
        .push_bytes(Bytes::from_static(
            b"data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\"}}\n\n",
        ))
        .unwrap();
    let error = parser.finish().unwrap_err();
    assert!(error
        .to_string()
        .contains("ended before terminal response event"));
}

#[test]
fn responses_sse_parser_yields_final_frame_without_trailing_separator() {
    let mut parser = ResponsesSseParser::new();
    parser
        .push_bytes(Bytes::from_static(
            b"data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\"}}\n\n",
        ))
        .unwrap();
    parser
        .push_bytes(Bytes::from_static(
            b"data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_1\"}}",
        ))
        .unwrap();
    let frames = parser.finish().unwrap();

    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].data["type"], "response.completed");
}

#[test]
fn responses_sse_parser_accepts_failed_and_incomplete_terminals() {
    for terminal in ["response.failed", "response.incomplete"] {
        let mut parser = ResponsesSseParser::new();
        let frames = parser
            .push_bytes(Bytes::from(format!(
                "data: {{\"type\":\"{terminal}\",\"response\":{{\"id\":\"resp_1\"}}}}\n\n"
            )))
            .unwrap();

        assert_eq!(frames[0].data["type"], terminal);
        parser.finish().unwrap();
    }
}
