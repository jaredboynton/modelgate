pub fn filter_codex_events(input: &str) -> String {
    let mut output = String::new();
    let mut current = String::new();
    for line in input.lines() {
        current.push_str(line);
        current.push('\n');
        if line.is_empty() {
            if !is_codex_event(&current) {
                output.push_str(&current);
            }
            current.clear();
        }
    }
    if !current.is_empty() && !is_codex_event(&current) {
        output.push_str(&current);
    }
    output
}

fn is_codex_event(block: &str) -> bool {
    block.lines().any(|line| {
        line.strip_prefix("event:")
            .map(str::trim_start)
            .is_some_and(|event| event.starts_with("codex."))
    })
}
