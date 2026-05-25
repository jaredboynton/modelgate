#!/usr/bin/env python3
import argparse
import datetime as dt
import http.client
import json
import os
import statistics
import time
import urllib.parse
from pathlib import Path


PROXY_URL = os.environ.get(
    "UMP_BENCH_BASE_URL", "http://127.0.0.1:18743"
)
ARTIFACT_ROOT = Path(
    os.environ.get("UMP_BENCH_ARTIFACT_ROOT", ".live-harness/benchmarks")
)
SCENARIOS = ("text", "tool_call", "tool_continuation")
METRIC_KEYS = (
    "ttft_ms",
    "terminal_ms",
    "total_ms",
    "chars_per_sec",
    "events_per_sec",
)
OUTPUT_COUNT_KEYS = ("text_chars", "events")
LOWER_IS_BETTER = {"ttft_ms", "terminal_ms", "total_ms"}
LATENCY_METRICS = ("ttft_ms", "terminal_ms", "total_ms")
THROUGHPUT_OUTPUT_REQUIREMENTS = {
    "chars_per_sec": ("text_chars", 32),
    "events_per_sec": ("events", 16),
}
DISTRIBUTION_DOMINANCE_THRESHOLD = 0.8
TERMINAL_EVENTS = {
    "response.completed",
    "response.failed",
    "response.incomplete",
    "message_stop",
    "error",
}
FAILED_TERMINAL_EVENTS = {"response.failed", "response.incomplete", "error"}


PROVIDERS = {
    "bedrock_messages": {
        "route": "/v1/messages",
        "model": "claude-sonnet-4-6",
        "kind": "messages",
        "tool": False,
    },
    "cursor_responses": {
        "route": "/v1/responses",
        "model": "composer-2-fast",
        "kind": "responses",
        "tool": True,
        "tool_choice": "cursor_nested",
    },
    "windsurf_responses": {
        "route": "/v1/responses",
        "model": "swe-1.6",
        "kind": "responses",
        "tool": True,
        "tool_choice": "openai",
    },
    "codex_responses_ws": {
        "route": "/v1/responses",
        "model": "gpt-5.5",
        "kind": "responses",
        "tool": True,
        "tool_choice": "openai",
    },
}


def now_utc():
    return dt.datetime.now(dt.UTC).strftime("%Y-%m-%dT%H:%M:%SZ")


def perf_ms(start):
    return (time.perf_counter() - start) * 1000.0


def text_payload(config):
    if config["kind"] == "messages":
        return {
            "model": config["model"],
            "stream": True,
            "max_tokens": 32,
            "messages": [{"role": "user", "content": "Reply with exactly: pong"}],
        }
    return {
        "model": config["model"],
        "stream": True,
        "store": False,
        "input": "Reply with exactly: pong",
    }


def tool_payload(config):
    payload = {
        "model": config["model"],
        "stream": True,
        "store": False,
        "input": "Call the lookup tool with q set to ok. Do not answer directly.",
        "tools": [
            {
                "type": "function",
                "name": "lookup",
                "description": "lookup a value",
                "parameters": {
                    "type": "object",
                    "properties": {"q": {"type": "string"}},
                    "required": ["q"],
                },
            }
        ],
        "max_tool_calls": 1,
    }
    if config.get("tool_choice") == "cursor_nested":
        payload["tool_choice"] = {"type": "function", "function": {"name": "lookup"}}
    else:
        payload["tool_choice"] = {"type": "function", "name": "lookup"}
    return payload


def continuation_payload(config, response_id, call_id):
    return {
        "model": config["model"],
        "stream": True,
        "store": False,
        "previous_response_id": response_id,
        "input": [
            {
                "type": "function_call_output",
                "call_id": call_id,
                "output": "lookup result: ok",
            }
        ],
    }


def parse_sse_frames(raw_text):
    event = None
    data_lines = []
    frames = []
    for line in raw_text.splitlines():
        if line == "":
            if event or data_lines:
                frames.append((event, "\n".join(data_lines)))
            event = None
            data_lines = []
            continue
        if line.startswith("event:"):
            event = line[6:].strip()
        elif line.startswith("data:"):
            data_lines.append(line[5:].strip())
        elif line.startswith(("id:", "retry:", ":")):
            continue
        elif data_lines:
            data_lines.append(line)
    if event or data_lines:
        frames.append((event, "\n".join(data_lines)))
    return frames


def parse_json_data(data):
    if not data or data == "[DONE]":
        return None
    try:
        return json.loads(data)
    except json.JSONDecodeError:
        return None


def extract_text_delta(event, data):
    value = parse_json_data(data)
    if not isinstance(value, dict):
        return ""
    delta = value.get("delta")
    if isinstance(delta, str):
        return delta
    if isinstance(delta, dict):
        text = delta.get("text") or delta.get("thinking")
        if isinstance(text, str):
            return text
    content_block = value.get("content_block")
    if isinstance(content_block, dict) and isinstance(content_block.get("text"), str):
        return content_block["text"]
    item = value.get("item")
    if isinstance(item, dict):
        content = item.get("content")
        if isinstance(content, list):
            return "".join(
                part.get("text", "") for part in content if isinstance(part, dict)
            )
    if event and "delta" in event and isinstance(value.get("text"), str):
        return value["text"]
    return ""


def tool_call_detected(event, value):
    if not isinstance(value, dict):
        return False
    if event and ("function_call" in event or "tool_use" in event):
        return True
    output = value.get("output")
    if isinstance(output, list) and output_has_tool_call(output):
        return True
    response = value.get("response")
    if isinstance(response, dict):
        response_output = response.get("output")
        if isinstance(response_output, list) and output_has_tool_call(response_output):
            return True
    item = value.get("item")
    if isinstance(item, dict) and item.get("type") in {"function_call", "tool_use"}:
        return True
    content_block = value.get("content_block")
    if isinstance(content_block, dict) and content_block.get("type") == "tool_use":
        return True
    return value.get("type") in {"function_call", "tool_use"}


def output_has_tool_call(output):
    return any(
        isinstance(item, dict)
        and (
            item.get("type") in {"function_call", "tool_use"}
            or (
                item.get("call_id")
                and (item.get("arguments") is not None or item.get("name"))
            )
        )
        for item in output
    )


def parse_arguments(arguments):
    if isinstance(arguments, dict):
        return arguments
    if not isinstance(arguments, str):
        return None
    try:
        parsed = json.loads(arguments)
    except json.JSONDecodeError:
        return None
    return parsed if isinstance(parsed, dict) else None


def arguments_are_valid(arguments):
    parsed = parse_arguments(arguments)
    return isinstance(parsed, dict) and parsed.get("q") == "ok"


def choose_arguments(current, candidate):
    if candidate in (None, ""):
        return current
    if current in (None, ""):
        return candidate
    if arguments_are_valid(candidate) and not arguments_are_valid(current):
        return candidate
    if isinstance(candidate, dict) and not isinstance(current, dict):
        return candidate
    if (
        isinstance(candidate, str)
        and isinstance(current, str)
        and len(candidate) > len(current)
    ):
        return candidate
    return current


def extract_ids(frames):
    response_id = None
    call_id = None
    arguments = None
    terminal_event = None
    error_message = None
    argument_chunks = []
    detected_tool_call = False
    for event, data in frames:
        value = parse_json_data(data)
        event_name = frame_event_name(event, value)
        event_type = value.get("type") if isinstance(value, dict) else None
        if event_name in TERMINAL_EVENTS:
            terminal_event = event_name
        if not isinstance(value, dict):
            continue
        detected_tool_call = detected_tool_call or tool_call_detected(event, value)
        if event == "error" or event_type == "error":
            error = value.get("error")
            if isinstance(error, dict):
                error_message = error.get("message") or json.dumps(error, sort_keys=True)
            else:
                error_message = data
        response = value.get("response")
        if isinstance(response, dict):
            response_id = response_id or response.get("id")
            error = response.get("error")
            if isinstance(error, dict):
                error_message = error.get("message") or json.dumps(error, sort_keys=True)
            output = response.get("output")
            if isinstance(output, list):
                for item in output:
                    if not isinstance(item, dict):
                        continue
                    call_id = call_id or item.get("call_id")
                    arguments = choose_arguments(arguments, item.get("arguments"))
        item = value.get("item")
        if isinstance(item, dict):
            call_id = call_id or item.get("call_id")
            arguments = choose_arguments(arguments, item.get("arguments"))
        call_id = call_id or value.get("call_id")
        arguments = choose_arguments(arguments, value.get("arguments"))
        delta = value.get("delta")
        if event and "arguments" in event and isinstance(delta, str):
            argument_chunks.append(delta)
        if isinstance(delta, dict):
            arguments = choose_arguments(arguments, delta.get("arguments"))
            partial_json = delta.get("partial_json")
            if isinstance(partial_json, str):
                argument_chunks.append(partial_json)
        content_block = value.get("content_block")
        if isinstance(content_block, dict):
            arguments = choose_arguments(arguments, content_block.get("input"))
    if argument_chunks:
        arguments = choose_arguments(arguments, "".join(argument_chunks))
    return response_id, call_id, arguments, terminal_event, error_message, detected_tool_call


def frame_event_name(event, value):
    if event in TERMINAL_EVENTS:
        return event
    if isinstance(value, dict) and value.get("type") in TERMINAL_EVENTS:
        return value["type"]
    if event:
        return event
    return None


def frame_error_message(value):
    if not isinstance(value, dict):
        return None
    error = value.get("error")
    if isinstance(error, dict):
        return error.get("message") or json.dumps(error, sort_keys=True)
    response = value.get("response")
    if isinstance(response, dict):
        response_error = response.get("error")
        if isinstance(response_error, dict):
            return response_error.get("message") or json.dumps(
                response_error, sort_keys=True
            )
    return None


def frame_is_failed_terminal(event_name, value):
    if event_name in FAILED_TERMINAL_EVENTS:
        return True
    if isinstance(value, dict):
        if value.get("type") in FAILED_TERMINAL_EVENTS or value.get("type") == "error":
            return True
        response = value.get("response")
        if isinstance(response, dict):
            if response.get("status") in {"failed", "incomplete"}:
                return True
            if response.get("error") is not None:
                return True
    return False


def read_stream(base_url, route, payload, timeout):
    parsed = urllib.parse.urlparse(base_url)
    if parsed.scheme != "http":
        raise RuntimeError(
            f"benchmark harness only supports local http base URLs: {base_url}"
        )
    body = json.dumps(payload, separators=(",", ":")).encode()
    conn = http.client.HTTPConnection(parsed.hostname, parsed.port, timeout=timeout)
    started = time.perf_counter()
    conn.request(
        "POST",
        route,
        body=body,
        headers={"content-type": "application/json", "accept": "text/event-stream"},
    )
    response = conn.getresponse()
    headers_ms = perf_ms(started)
    raw = bytearray()
    first_byte_ms = None
    first_event_ms = None
    first_meaningful_ms = None
    terminal_ms = None
    saw_error = False
    error_message = None
    event_count = 0
    text_chars = 0
    buffer = ""
    read_chunk = getattr(response, "read1", response.read)
    stop_after_terminal = False

    while True:
        chunk = read_chunk(8192)
        if not chunk:
            break
        if first_byte_ms is None:
            first_byte_ms = perf_ms(started)
        raw.extend(chunk)
        buffer += chunk.decode("utf-8", errors="replace")
        while "\n\n" in buffer or "\r\n\r\n" in buffer:
            if "\n\n" in buffer:
                frame_text, buffer = buffer.split("\n\n", 1)
            else:
                frame_text, buffer = buffer.split("\r\n\r\n", 1)
            frames = parse_sse_frames(frame_text + "\n\n")
            for event, data in frames:
                event_count += 1
                if first_event_ms is None:
                    first_event_ms = perf_ms(started)
                delta = extract_text_delta(event, data)
                text_chars += len(delta)
                value = parse_json_data(data)
                detected_tool_call = (
                    isinstance(value, dict) and tool_call_detected(event, value)
                )
                if first_meaningful_ms is None and (delta or detected_tool_call):
                    first_meaningful_ms = perf_ms(started)
                event_name = frame_event_name(event, value)
                if event_name in TERMINAL_EVENTS:
                    terminal_ms = perf_ms(started)
                    stop_after_terminal = True
                if frame_is_failed_terminal(event_name, value):
                    saw_error = True
                    stop_after_terminal = True
                    error_message = error_message or frame_error_message(value)
            if stop_after_terminal:
                break
        if stop_after_terminal:
            break

    ended_ms = perf_ms(started)
    conn.close()
    raw_text = raw.decode("utf-8", errors="replace")
    frames = parse_sse_frames(raw_text)
    (
        response_id,
        call_id,
        arguments,
        terminal_event,
        parsed_error,
        detected_tool_call,
    ) = extract_ids(frames)
    error_message = error_message or parsed_error
    active_ms = max(
        (terminal_ms or ended_ms)
        - (first_meaningful_ms or first_byte_ms or headers_ms),
        1,
    )
    terminal_failed = terminal_event in FAILED_TERMINAL_EVENTS
    if not terminal_failed:
        for event, data in frames:
            value = parse_json_data(data)
            if frame_is_failed_terminal(frame_event_name(event, value), value):
                terminal_failed = True
                break
    return {
        "ok": 200 <= response.status < 300 and not saw_error and not terminal_failed,
        "status": response.status,
        "reason": response.reason,
        "headers_ms": headers_ms,
        "first_byte_ms": first_byte_ms,
        "first_event_ms": first_event_ms,
        "ttft_ms": first_meaningful_ms or first_event_ms or first_byte_ms,
        "terminal_ms": terminal_ms,
        "total_ms": ended_ms,
        "bytes": len(raw),
        "events": event_count,
        "text_chars": text_chars,
        "chars_per_sec": (text_chars / active_ms) * 1000.0,
        "events_per_sec": (event_count / active_ms) * 1000.0,
        "response_id": response_id,
        "call_id": call_id,
        "arguments": arguments,
        "tool_call_detected": detected_tool_call,
        "tool_arguments_valid": arguments_are_valid(arguments),
        "terminal_event": terminal_event,
        "error": error_message,
        "body_preview": raw_text[:500],
        "body": raw_text,
    }


def percentile(values, percentile_value):
    sorted_values = sorted(values)
    if not sorted_values:
        return None
    if len(sorted_values) == 1:
        return sorted_values[0]
    rank = (len(sorted_values) - 1) * (percentile_value / 100.0)
    lower_index = int(rank)
    upper_index = min(lower_index + 1, len(sorted_values) - 1)
    weight = rank - lower_index
    return sorted_values[lower_index] + (
        sorted_values[upper_index] - sorted_values[lower_index]
    ) * weight


def summarize_values(values):
    summary = {
        "median": statistics.median(values),
        "mean": statistics.fmean(values),
        "min": min(values),
        "max": max(values),
        "p05": percentile(values, 5),
        "p25": percentile(values, 25),
        "p75": percentile(values, 75),
        "p90": percentile(values, 90),
        "p95": percentile(values, 95),
    }
    if len(values) > 1:
        summary["stdev"] = statistics.stdev(values)
    return summary


def count_values(samples, key):
    counts = {}
    for sample in samples:
        value = sample.get(key)
        if value in (None, ""):
            continue
        counts[str(value)] = counts.get(str(value), 0) + 1
    return counts


def summarize(samples):
    successful = [sample for sample in samples if sample.get("ok")]
    summary = {
        "success_count": len(successful),
        "sample_count": len(samples),
        "failure_count": len(samples) - len(successful),
        "status_counts": count_values(samples, "status"),
        "terminal_event_counts": count_values(samples, "terminal_event"),
    }
    for key in METRIC_KEYS:
        values = [
            sample[key]
            for sample in successful
            if isinstance(sample.get(key), (int, float))
        ]
        if values:
            summary[key] = summarize_values(values)
    for key in OUTPUT_COUNT_KEYS:
        values = [
            sample[key]
            for sample in successful
            if isinstance(sample.get(key), (int, float))
        ]
        if values:
            summary[key] = summarize_values(values)
    if any(sample.get("tool_call_detected") or sample.get("call_id") for sample in samples):
        summary["tool_call_count"] = sum(
            1 for sample in successful if sample.get("tool_call_detected")
        )
        summary["tool_arguments_valid_count"] = sum(
            1 for sample in successful if sample.get("tool_arguments_valid")
        )
    return summary


def with_continuation_attempt(result, response_id, call_id):
    sample = dict(result)
    sample["continuation_of_response_id"] = response_id
    sample["continuation_of_call_id"] = call_id
    return sample


def reject_repeated_tool_continuation(result):
    sample = dict(result)
    if sample.get("ok") and sample.get("tool_call_detected"):
        sample["ok"] = False
        sample["error"] = "tool continuation emitted another tool call"
    return sample


def run_provider(provider_id, samples, timeout):
    config = PROVIDERS[provider_id]
    text_samples = []
    tool_samples = []
    continuation_samples = []
    for index in range(samples):
        try:
            text_samples.append(
                {
                    "sample": index + 1,
                    **without_body(
                        read_stream(
                            PROXY_URL, config["route"], text_payload(config), timeout
                        )
                    ),
                }
            )
        except Exception as error:
            text_samples.append({"sample": index + 1, "ok": False, "error": str(error)})
    if config["tool"]:
        for index in range(samples):
            sample_number = index + 1
            try:
                tool = read_stream(
                    PROXY_URL, config["route"], tool_payload(config), timeout
                )
            except Exception as error:
                tool_samples.append(
                    {"sample": sample_number, "ok": False, "error": str(error)}
                )
                continuation_samples.append(
                    {
                        "sample": sample_number,
                        "ok": False,
                        "error": "tool request failed before continuation",
                    }
                )
                continue

            tool_samples.append({"sample": sample_number, **without_body(tool)})
            response_id, call_id = tool.get("response_id"), tool.get("call_id")
            if response_id and call_id:
                try:
                    continuation = read_stream(
                        PROXY_URL,
                        config["route"],
                        continuation_payload(config, response_id, call_id),
                        timeout,
                    )
                    continuation_samples.append(
                        {
                            "sample": sample_number,
                            **without_body(
                                with_continuation_attempt(
                                    reject_repeated_tool_continuation(continuation),
                                    response_id,
                                    call_id,
                                )
                            ),
                        }
                    )
                except Exception as error:
                    continuation_samples.append(
                        {
                            "sample": sample_number,
                            "ok": False,
                            "error": str(error),
                            "continuation_of_response_id": response_id,
                            "continuation_of_call_id": call_id,
                        }
                    )
            else:
                continuation_samples.append(
                    {
                        "sample": sample_number,
                        "ok": False,
                        "error": "missing response_id or call_id",
                    }
                )
    return {
        "provider_id": provider_id,
        "provider": config,
        "text": {"samples": text_samples, "summary": summarize(text_samples)},
        "tool_call": {"samples": tool_samples, "summary": summarize(tool_samples)},
        "tool_continuation": {
            "samples": continuation_samples,
            "summary": summarize(continuation_samples),
        },
    }


def without_body(result):
    result = dict(result)
    result.pop("body", None)
    return result


def summary_metric(summary, metric):
    value = summary.get(metric)
    if isinstance(value, dict):
        return value.get("median")
    return None


def scenario_field_median(scenario_data, field):
    summary_value = summary_metric(scenario_data.get("summary", {}), field)
    if isinstance(summary_value, (int, float)):
        return summary_value
    values = [
        sample[field]
        for sample in scenario_data.get("samples", [])
        if sample.get("ok") and isinstance(sample.get(field), (int, float))
    ]
    if values:
        return statistics.median(values)
    return None


def improvement_pct(metric, baseline_value, current_value):
    if not isinstance(baseline_value, (int, float)) or baseline_value == 0:
        return None
    if not isinstance(current_value, (int, float)):
        return None
    if metric in LOWER_IS_BETTER:
        return ((baseline_value - current_value) / baseline_value) * 100.0
    return ((current_value - baseline_value) / baseline_value) * 100.0


def tool_correctness_ratio(summary):
    if "tool_arguments_valid_count" not in summary:
        return None
    success_count = summary.get("success_count", 0)
    if success_count == 0:
        return None
    return summary.get("tool_arguments_valid_count", 0) / success_count


def success_ratio(summary):
    sample_count = summary.get("sample_count", 0)
    if sample_count == 0:
        return None
    return summary.get("success_count", 0) / sample_count


def normalize_baseline_runs(baseline_runs):
    if not baseline_runs:
        return []
    if isinstance(baseline_runs, dict):
        return [baseline_runs]
    return list(baseline_runs)


def run_artifact(run):
    if not isinstance(run, dict):
        return None
    return run.get("artifact") or run.get("label")


def baseline_artifacts(baseline_runs):
    return [artifact for artifact in (run_artifact(run) for run in baseline_runs) if artifact]


def empty_scenario_result():
    return {
        "samples": [],
        "summary": {
            "success_count": 0,
            "sample_count": 0,
            "failure_count": 0,
            "status_counts": {},
            "terminal_event_counts": {},
        },
    }


def scenario_result(provider, scenario):
    result = provider.get(scenario, {}) if isinstance(provider, dict) else {}
    if isinstance(result, dict) and isinstance(result.get("summary"), dict):
        return result
    return empty_scenario_result()


def provider_baselines(baseline_runs, provider_id):
    matches = []
    for baseline_run in baseline_runs:
        for provider in baseline_run.get("providers", []):
            if provider.get("provider_id") == provider_id:
                matches.append((baseline_run, provider))
    return matches


def metric_is_better(metric, candidate_value, current_best):
    if current_best is None:
        return True
    if metric in LOWER_IS_BETTER:
        return candidate_value < current_best
    return candidate_value > current_best


def best_metric_baseline(baselines, scenario, metric):
    best = None
    for baseline_run, baseline_provider in baselines:
        baseline_scenario = scenario_result(baseline_provider, scenario)
        summary = baseline_scenario["summary"]
        value = summary_metric(summary, metric)
        if not isinstance(value, (int, float)):
            continue
        if best is None or metric_is_better(metric, value, best["value"]):
            best = {
                "run": baseline_run,
                "scenario": baseline_scenario,
                "summary": summary,
                "value": value,
            }
    return best


def latency_non_regressed(current_summary, baselines, scenario):
    checked_latency = False
    for metric in LATENCY_METRICS:
        baseline = best_metric_baseline(baselines, scenario, metric)
        if baseline is None:
            continue
        pct = improvement_pct(
            metric, baseline["value"], summary_metric(current_summary, metric)
        )
        if pct is None:
            continue
        checked_latency = True
        if pct < 0:
            return False
    return checked_latency


def throughput_gate(metric, current_scenario, metric_baseline, baselines, scenario):
    requirement = THROUGHPUT_OUTPUT_REQUIREMENTS.get(metric)
    if requirement is None:
        return None
    output_field, minimum_units = requirement
    baseline_scenario = metric_baseline["scenario"] if metric_baseline else {}
    current_units = scenario_field_median(current_scenario, output_field)
    baseline_units = scenario_field_median(baseline_scenario, output_field)
    enough_output = (
        isinstance(current_units, (int, float))
        and isinstance(baseline_units, (int, float))
        and current_units >= minimum_units
        and baseline_units >= minimum_units
    )
    latency_ok = latency_non_regressed(
        current_scenario["summary"], baselines, scenario
    )
    return {
        "passed": enough_output or latency_ok,
        "enough_output": enough_output,
        "latency_non_regressed": latency_ok,
        "output_field": output_field,
        "min_output_median": minimum_units,
        "baseline_output_median": baseline_units,
        "current_output_median": current_units,
    }


def metric_sample_values(scenario, metric):
    return [
        sample[metric]
        for sample in scenario.get("samples", [])
        if sample.get("ok") and isinstance(sample.get(metric), (int, float))
    ]


def distribution_gate(metric, current_scenario, baseline_scenario):
    current_values = metric_sample_values(current_scenario, metric)
    baseline_values = metric_sample_values(baseline_scenario, metric)
    if not current_values or not baseline_values:
        return {
            "passed": True,
            "dominance": None,
            "current_samples": len(current_values),
            "baseline_samples": len(baseline_values),
        }

    favorable = 0
    ties = 0
    total = len(current_values) * len(baseline_values)
    for current in current_values:
        for baseline in baseline_values:
            if current == baseline:
                ties += 1
            elif metric in LOWER_IS_BETTER:
                favorable += int(current < baseline)
            else:
                favorable += int(current > baseline)
    dominance = (favorable + (0.5 * ties)) / total
    return {
        "passed": dominance >= DISTRIBUTION_DOMINANCE_THRESHOLD,
        "dominance": dominance,
        "current_samples": len(current_values),
        "baseline_samples": len(baseline_values),
        "threshold": DISTRIBUTION_DOMINANCE_THRESHOLD,
    }


def best_ratio_baseline(baselines, scenario, ratio_fn):
    best = None
    for baseline_run, baseline_provider in baselines:
        summary = scenario_result(baseline_provider, scenario)["summary"]
        ratio = ratio_fn(summary)
        if not isinstance(ratio, (int, float)):
            continue
        if best is None or ratio > best["ratio"]:
            best = {
                "run": baseline_run,
                "summary": summary,
                "ratio": ratio,
            }
    return best


def has_enough_baseline_samples(baselines, scenario, min_successful_samples):
    return any(
        scenario_result(baseline_provider, scenario)["summary"]
        .get("success_count", 0)
        >= min_successful_samples
        for _, baseline_provider in baselines
    )


def normalize_provider_baselines(baselines):
    if isinstance(baselines, dict):
        return [(None, baselines)]
    return [
        (None, provider) if isinstance(provider, dict) else provider
        for provider in baselines
    ]


def tool_loop_regressions(current_provider, baseline_provider):
    baselines = normalize_provider_baselines(baseline_provider)
    regressions = []
    for scenario in ("tool_call", "tool_continuation"):
        current_summary = scenario_result(current_provider, scenario)["summary"]
        current_success_ratio = success_ratio(current_summary)
        baseline_success = best_ratio_baseline(baselines, scenario, success_ratio)
        baseline_success_ratio = (
            baseline_success["ratio"] if baseline_success is not None else None
        )
        if (
            current_success_ratio is not None
            and baseline_success_ratio is not None
            and current_success_ratio < baseline_success_ratio
        ):
            baseline_summary = baseline_success["summary"]
            regressions.append(
                "{} success {:.0f}/{} < {:.0f}/{}".format(
                    scenario,
                    current_summary.get("success_count", 0),
                    current_summary.get("sample_count", 0),
                    baseline_summary.get("success_count", 0),
                    baseline_summary.get("sample_count", 0),
                )
            )
        current_tool_ratio = tool_correctness_ratio(current_summary)
        baseline_tool = best_ratio_baseline(baselines, scenario, tool_correctness_ratio)
        baseline_tool_ratio = (
            baseline_tool["ratio"] if baseline_tool is not None else None
        )
        if (
            current_tool_ratio is not None
            and baseline_tool_ratio is not None
            and current_tool_ratio < baseline_tool_ratio
        ):
            regressions.append(
                f"{scenario} tool args {current_tool_ratio:.3f} < {baseline_tool_ratio:.3f}"
            )
    return regressions


def compare_runs(current_run, baseline_run, threshold_pct, min_successful_samples):
    baseline_runs = normalize_baseline_runs(baseline_run)
    comparisons = []
    any_significant_improvement = False
    for current_provider in current_run.get("providers", []):
        provider_id = current_provider["provider_id"]
        baselines = provider_baselines(baseline_runs, provider_id)
        if not baselines:
            continue
        provider_tool_regressions = tool_loop_regressions(
            current_provider, baselines
        )
        provider_tool_loop_regressed = bool(provider_tool_regressions)
        for scenario in SCENARIOS:
            current_scenario = scenario_result(current_provider, scenario)
            current_summary = current_scenario["summary"]
            enough_samples = (
                current_summary.get("success_count", 0) >= min_successful_samples
                and has_enough_baseline_samples(
                    baselines, scenario, min_successful_samples
                )
            )
            current_tool_ratio = tool_correctness_ratio(current_summary)
            baseline_tool = best_ratio_baseline(
                baselines, scenario, tool_correctness_ratio
            )
            baseline_tool_ratio = (
                baseline_tool["ratio"] if baseline_tool is not None else None
            )
            tool_correctness_regressed = (
                current_tool_ratio is not None
                and baseline_tool_ratio is not None
                and current_tool_ratio < baseline_tool_ratio
            )
            metrics = []
            for metric in METRIC_KEYS:
                metric_baseline = best_metric_baseline(baselines, scenario, metric)
                baseline_value = (
                    metric_baseline["value"] if metric_baseline is not None else None
                )
                current_value = summary_metric(current_summary, metric)
                pct = improvement_pct(metric, baseline_value, current_value)
                metric_enough_samples = (
                    current_summary.get("success_count", 0) >= min_successful_samples
                    and metric_baseline is not None
                    and metric_baseline["summary"].get("success_count", 0)
                    >= min_successful_samples
                )
                metric_throughput_gate = throughput_gate(
                    metric, current_scenario, metric_baseline, baselines, scenario
                )
                throughput_gate_passed = (
                    metric_throughput_gate is None
                    or metric_throughput_gate["passed"]
                )
                baseline_scenario = (
                    metric_baseline["scenario"] if metric_baseline is not None else {}
                )
                metric_distribution_gate = distribution_gate(
                    metric, current_scenario, baseline_scenario
                )
                significant = (
                    metric_enough_samples
                    and pct is not None
                    and pct >= threshold_pct
                    and throughput_gate_passed
                    and metric_distribution_gate["passed"]
                    and not tool_correctness_regressed
                    and not provider_tool_loop_regressed
                )
                any_significant_improvement = (
                    any_significant_improvement or significant
                )
                metric_comparison = {
                    "metric": metric,
                    "baseline_median": baseline_value,
                    "current_median": current_value,
                    "improvement_pct": pct,
                    "significant": significant,
                    "baseline_artifact": run_artifact(metric_baseline["run"])
                    if metric_baseline is not None
                    else None,
                    "enough_samples": metric_enough_samples,
                }
                if metric_throughput_gate is not None:
                    metric_comparison["throughput_gate"] = metric_throughput_gate
                metric_comparison["distribution_gate"] = metric_distribution_gate
                metrics.append(metric_comparison)
            comparisons.append(
                {
                    "provider_id": provider_id,
                    "scenario": scenario,
                    "enough_samples": enough_samples,
                    "tool_correctness_regressed": tool_correctness_regressed,
                    "tool_loop_regressed": provider_tool_loop_regressed,
                    "tool_loop_regressions": provider_tool_regressions,
                    "metrics": metrics,
                }
            )
    comparison = {
        "threshold_pct": threshold_pct,
        "min_successful_samples": min_successful_samples,
        "any_significant_improvement": any_significant_improvement,
        "comparisons": comparisons,
    }
    artifacts = baseline_artifacts(baseline_runs)
    if len(baseline_runs) == 1:
        comparison["baseline_artifact"] = artifacts[0] if artifacts else None
    else:
        comparison["baseline_artifacts"] = artifacts
        comparison["baseline_mode"] = "best_of_prior_median"
    return comparison


def format_number(value):
    return f"{value:.1f}" if isinstance(value, (int, float)) else ""


def format_tool_valid(summary):
    if "tool_arguments_valid_count" not in summary:
        return ""
    return "{}/{}".format(
        summary.get("tool_arguments_valid_count", 0),
        summary.get("success_count", 0),
    )


def markdown_summary(run):
    lines = []
    lines.append(f"### {run['label']}")
    lines.append("")
    lines.append(f"- Timestamp: `{run['created_at']}`")
    lines.append(f"- Samples per scenario: `{run['samples']}`")
    lines.append(f"- Artifact: `{run['artifact']}`")
    lines.append(
        f"- Improvement threshold: `{run['improvement_threshold_pct']}%` with "
        f"`{run['min_successful_samples']}` successful samples per side"
    )
    lines.append("")
    lines.append(
        "| Provider | Scenario | Success | Median TTFT ms | P95 TTFT ms | "
        "Median total ms | Median chars/sec | Tool args valid | Notes |"
    )
    lines.append("|---|---:|---:|---:|---:|---:|---:|---:|---|")
    for provider in run["providers"]:
        for scenario in SCENARIOS:
            scenario_data = scenario_result(provider, scenario)
            summary = scenario_data["summary"]
            ttft = summary.get("ttft_ms", {}).get("median")
            p95_ttft = summary.get("ttft_ms", {}).get("p95")
            total = summary.get("total_ms", {}).get("median")
            cps = summary.get("chars_per_sec", {}).get("median")
            success = f"{summary.get('success_count', 0)}/{summary.get('sample_count', 0)}"
            notes = ""
            if summary.get("success_count", 0) == 0 and scenario_data["samples"]:
                notes = scenario_data["samples"][0].get("error") or ""
            lines.append(
                (
                    "| {provider_id} | {scenario} | {success} | {ttft} | "
                    "{p95_ttft} | {total} | {cps} | {tool_valid} | {notes} |"
                ).format(
                    provider_id=provider["provider_id"],
                    scenario=scenario,
                    success=success,
                    ttft=format_number(ttft),
                    p95_ttft=format_number(p95_ttft),
                    total=format_number(total),
                    cps=format_number(cps),
                    tool_valid=format_tool_valid(summary),
                    notes=notes.replace("|", "/")[:80],
                )
            )
    comparison = run.get("comparison")
    if comparison:
        lines.append("")
        lines.append("#### Baseline comparison")
        lines.append("")
        if comparison.get("baseline_artifacts"):
            artifacts = "`, `".join(comparison["baseline_artifacts"])
            lines.append(f"- Baseline artifacts: `{artifacts}`")
            lines.append(
                "- Baseline selection: `best prior median per provider/scenario/metric; "
                "tool-loop guard uses best prior success/tool ratio`"
            )
        else:
            lines.append(f"- Baseline artifact: `{comparison.get('baseline_artifact')}`")
        lines.append(
            f"- Any significant improvement: `{comparison.get('any_significant_improvement')}`"
        )
        lines.append("")
        lines.append(
            "| Provider | Scenario | Metric | Baseline median | Current median | "
            "Improvement % | Significant | Notes |"
        )
        lines.append("|---|---|---|---:|---:|---:|---:|---|")
        for item in comparison["comparisons"]:
            notes = []
            if not item["enough_samples"]:
                notes.append("insufficient samples")
            if item["tool_correctness_regressed"]:
                notes.append("tool correctness regressed")
            if item.get("tool_loop_regressed"):
                notes.append(
                    "tool loop regressed: "
                    + "; ".join(item.get("tool_loop_regressions", []))
                )
            for metric in item["metrics"]:
                metric_notes = list(notes)
                if (
                    not metric.get("enough_samples", item["enough_samples"])
                    and "insufficient samples" not in metric_notes
                ):
                    metric_notes.append("insufficient samples")
                throughput_gate_result = metric.get("throughput_gate")
                if (
                    throughput_gate_result is not None
                    and not throughput_gate_result.get("passed")
                ):
                    metric_notes.append(
                        "throughput gate: insufficient output and latency regressed"
                    )
                distribution_gate_result = metric.get("distribution_gate")
                if (
                    distribution_gate_result is not None
                    and not distribution_gate_result.get("passed")
                ):
                    metric_notes.append(
                        "distribution gate: pairwise dominance below threshold"
                    )
                if comparison.get("baseline_artifacts") and metric.get(
                    "baseline_artifact"
                ):
                    metric_notes.append(f"baseline: {metric['baseline_artifact']}")
                notes_text = ", ".join(metric_notes)
                lines.append(
                    (
                        "| {provider_id} | {scenario} | {metric} | {baseline} | "
                        "{current} | {pct} | {significant} | {notes} |"
                    ).format(
                        provider_id=item["provider_id"],
                        scenario=item["scenario"],
                        metric=metric["metric"],
                        baseline=format_number(metric["baseline_median"]),
                        current=format_number(metric["current_median"]),
                        pct=format_number(metric["improvement_pct"]),
                        significant=metric["significant"],
                        notes=notes_text,
                    )
                )
    lines.append("")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--label", default="wave-0-baseline")
    parser.add_argument("--samples", type=int, default=3)
    parser.add_argument("--timeout", type=float, default=60.0)
    parser.add_argument(
        "--providers",
        default=",".join(PROVIDERS.keys()),
        help="comma-separated provider ids",
    )
    parser.add_argument(
        "--compare-to",
        action="append",
        default=[],
        help=(
            "prior summary.json artifact used for median improvement comparison; "
            "repeat to compare against the best prior median per metric"
        ),
    )
    parser.add_argument(
        "--improvement-threshold-pct",
        type=float,
        default=5.0,
        help="minimum median improvement percentage counted as significant",
    )
    parser.add_argument(
        "--min-successful-samples",
        type=int,
        default=3,
        help="minimum successful samples per side before significance is evaluated",
    )
    args = parser.parse_args()

    run_id = dt.datetime.now(dt.UTC).strftime("%Y%m%dT%H%M%SZ") + "-" + args.label
    run_dir = ARTIFACT_ROOT / run_id
    run_dir.mkdir(parents=True, exist_ok=True)
    provider_ids = [
        provider.strip() for provider in args.providers.split(",") if provider.strip()
    ]
    run = {
        "label": args.label,
        "created_at": now_utc(),
        "base_url": PROXY_URL,
        "samples": args.samples,
        "improvement_threshold_pct": args.improvement_threshold_pct,
        "min_successful_samples": args.min_successful_samples,
        "providers": [],
    }
    for provider_id in provider_ids:
        if provider_id not in PROVIDERS:
            raise SystemExit(f"unknown provider id: {provider_id}")
        provider_result = run_provider(provider_id, args.samples, args.timeout)
        run["providers"].append(provider_result)
    artifact = run_dir / "summary.json"
    run["artifact"] = str(artifact)
    if args.compare_to:
        baseline_runs = [
            json.loads(Path(compare_to).read_text()) for compare_to in args.compare_to
        ]
        run["comparison"] = compare_runs(
            run,
            baseline_runs,
            args.improvement_threshold_pct,
            args.min_successful_samples,
        )
    artifact.write_text(json.dumps(run, indent=2, sort_keys=True))
    (run_dir / "summary.md").write_text(markdown_summary(run))
    print(json.dumps({"artifact": str(artifact), "label": args.label}, indent=2))
    print(markdown_summary(run))


if __name__ == "__main__":
    main()
