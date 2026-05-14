pub fn splice_completed_event(input: &str) -> String {
    let mut output = String::new();
    let mut current = String::new();
    let mut output_items = Vec::new();

    for line in input.lines() {
        current.push_str(line);
        current.push('\n');
        if line.is_empty() {
            output.push_str(&process_event_block(&current, &mut output_items));
            current.clear();
        }
    }

    if !current.is_empty() {
        output.push_str(&process_event_block(&current, &mut output_items));
    }

    output
}

fn process_event_block(block: &str, output_items: &mut Vec<serde_json::Value>) -> String {
    let Some(event) = event_name(block) else {
        return block.to_string();
    };

    match event.as_str() {
        "response.output_item.done" => {
            if let Some(item) = event_data_json(block).and_then(extract_output_item) {
                output_items.push(item);
            }
            block.to_string()
        }
        "response.completed" if !output_items.is_empty() => {
            let Some(mut data) = event_data_json(block) else {
                return block.to_string();
            };
            splice_output_items(&mut data, output_items);
            rewrite_data(block, &data)
        }
        _ => block.to_string(),
    }
}

fn event_name(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        line.strip_prefix("event:")
            .map(str::trim_start)
            .map(ToOwned::to_owned)
    })
}

fn event_data_json(block: &str) -> Option<serde_json::Value> {
    let data = block
        .lines()
        .filter_map(|line| line.strip_prefix("data:").map(str::trim_start))
        .collect::<Vec<_>>()
        .join("\n");
    serde_json::from_str(&data).ok()
}

fn extract_output_item(mut data: serde_json::Value) -> Option<serde_json::Value> {
    data.get_mut("item").map(std::mem::take).or_else(|| {
        if data.get("type").is_some() || data.get("id").is_some() {
            Some(data)
        } else {
            None
        }
    })
}

fn splice_output_items(data: &mut serde_json::Value, output_items: &[serde_json::Value]) {
    let items = serde_json::Value::Array(output_items.to_vec());
    if let Some(response) = data
        .get_mut("response")
        .and_then(|value| value.as_object_mut())
    {
        response.insert("output".into(), items);
    } else if let Some(response) = data.as_object_mut() {
        response.insert("output".into(), items);
    }
}

fn rewrite_data(block: &str, data: &serde_json::Value) -> String {
    let mut rewritten = String::new();
    let mut wrote_data = false;
    for line in block.lines() {
        if line.strip_prefix("data:").is_some() {
            if !wrote_data {
                rewritten.push_str("data: ");
                rewritten.push_str(&data.to_string());
                rewritten.push('\n');
                wrote_data = true;
            }
        } else {
            rewritten.push_str(line);
            rewritten.push('\n');
        }
    }
    rewritten
}
