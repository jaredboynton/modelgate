use unified_model_proxy_v2::upstream::google::rewrite_google_path;

#[test]
fn rewrites_amp_vertex_path_to_gemini_path() {
    let rewritten = rewrite_google_path(
        "/api/provider/google/v1beta1/publishers/google/models/gemini-3-flash-preview:generateContent",
    )
    .unwrap();
    assert_eq!(
        rewritten,
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-3-flash-preview:generateContent"
    );
}
