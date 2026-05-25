#!/usr/bin/env python3
import importlib.util
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("stream_bench.py")
SPEC = importlib.util.spec_from_file_location("stream_bench", SCRIPT_PATH)
stream_bench = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(stream_bench)


def metric(median):
    return {
        "median": median,
        "mean": median,
        "min": median,
        "max": median,
        "p05": median,
        "p25": median,
        "p75": median,
        "p90": median,
        "p95": median,
    }


def scenario_summary(
    ttft=None,
    terminal=None,
    total=None,
    chars=None,
    events=None,
    text_chars=128,
    event_count=32,
    metric_samples=None,
):
    samples = []
    if ttft is not None:
        values = {
            "ttft_ms": ttft,
            "terminal_ms": terminal,
            "total_ms": total,
            "chars_per_sec": chars,
            "events_per_sec": events,
        }
        metric_samples = metric_samples or {}
        for index in range(3):
            sample = {"ok": True, "text_chars": text_chars, "events": event_count}
            for key, value in values.items():
                if value is None:
                    continue
                per_sample = metric_samples.get(key)
                sample[key] = per_sample[index] if per_sample else value
            samples.append(sample)
    summary = {
        "success_count": 3 if ttft is not None else 0,
        "sample_count": 3 if ttft is not None else 0,
        "failure_count": 0,
        "status_counts": {},
        "terminal_event_counts": {},
    }
    values = {
        "ttft_ms": ttft,
        "terminal_ms": terminal,
        "total_ms": total,
        "chars_per_sec": chars,
        "events_per_sec": events,
    }
    for key, value in values.items():
        if value is not None:
            summary[key] = metric(value)
    return {"samples": samples, "summary": summary}


def run(artifact, text):
    return {
        "artifact": artifact,
        "providers": [
            {
                "provider_id": "fake_provider",
                "provider": {},
                "text": scenario_summary(**text),
                "tool_call": scenario_summary(),
                "tool_continuation": scenario_summary(),
            }
        ],
    }


def set_counts(scenario, success_count, sample_count, tool_valid_count=None):
    scenario["samples"] = [{} for _ in range(sample_count)]
    scenario["summary"].update(
        {
            "success_count": success_count,
            "sample_count": sample_count,
            "failure_count": sample_count - success_count,
        }
    )
    if tool_valid_count is not None:
        scenario["summary"]["tool_arguments_valid_count"] = tool_valid_count


def markdown_run(run_payload, comparison=None):
    run_payload = dict(run_payload)
    run_payload.update(
        {
            "label": "test-run",
            "created_at": "2026-05-25T00:00:00Z",
            "samples": 3,
            "improvement_threshold_pct": 5.0,
            "min_successful_samples": 3,
        }
    )
    if comparison is not None:
        run_payload["comparison"] = comparison
    return run_payload


class CompareRunsTest(unittest.TestCase):
    def text_comparison(self, comparison):
        return next(
            item
            for item in comparison["comparisons"]
            if item["provider_id"] == "fake_provider" and item["scenario"] == "text"
        )

    def metric_comparison(self, comparison, metric_name):
        text = self.text_comparison(comparison)
        return next(
            metric for metric in text["metrics"] if metric["metric"] == metric_name
        )

    def test_single_baseline_comparison_remains_compatible(self):
        baseline = run(
            "baseline.json",
            {"ttft": 100, "terminal": 100, "total": 100, "chars": 50, "events": 50},
        )
        current = run(
            "current.json",
            {"ttft": 90, "terminal": 90, "total": 90, "chars": 55, "events": 55},
        )

        comparison = stream_bench.compare_runs(current, baseline, 5.0, 3)
        ttft = self.metric_comparison(comparison, "ttft_ms")

        self.assertEqual(comparison["baseline_artifact"], "baseline.json")
        self.assertTrue(comparison["any_significant_improvement"])
        self.assertEqual(ttft["baseline_median"], 100)
        self.assertEqual(ttft["current_median"], 90)
        self.assertAlmostEqual(ttft["improvement_pct"], 10.0)
        self.assertTrue(ttft["significant"])

    def test_multiple_baselines_compare_each_metric_to_best_prior_median(self):
        latency_slow_throughput_best = run(
            "latency-slow-throughput-best.json",
            {"ttft": 100, "terminal": 100, "total": 100, "chars": 70, "events": 70},
        )
        latency_best_throughput_slow = run(
            "latency-best-throughput-slow.json",
            {"ttft": 80, "terminal": 80, "total": 80, "chars": 60, "events": 60},
        )
        current = run(
            "current.json",
            {"ttft": 90, "terminal": 90, "total": 90, "chars": 65, "events": 65},
        )

        comparison = stream_bench.compare_runs(
            current,
            [latency_slow_throughput_best, latency_best_throughput_slow],
            5.0,
            3,
        )
        ttft = self.metric_comparison(comparison, "ttft_ms")
        throughput = self.metric_comparison(comparison, "chars_per_sec")

        self.assertEqual(
            comparison["baseline_artifacts"],
            [
                "latency-slow-throughput-best.json",
                "latency-best-throughput-slow.json",
            ],
        )
        self.assertFalse(comparison["any_significant_improvement"])
        self.assertEqual(ttft["baseline_median"], 80)
        self.assertEqual(ttft["baseline_artifact"], "latency-best-throughput-slow.json")
        self.assertAlmostEqual(ttft["improvement_pct"], -12.5)
        self.assertFalse(ttft["significant"])
        self.assertEqual(throughput["baseline_median"], 70)
        self.assertEqual(
            throughput["baseline_artifact"], "latency-slow-throughput-best.json"
        )
        self.assertAlmostEqual(throughput["improvement_pct"], -7.142857142857142)
        self.assertFalse(throughput["significant"])

    def test_tool_loop_regression_uses_best_success_and_tool_ratios(self):
        metric_baseline = run(
            "metric-baseline.json",
            {"ttft": 100, "terminal": 100, "total": 100, "chars": 50, "events": 50},
        )
        best_success = run(
            "best-success.json",
            {"ttft": 140, "terminal": 140, "total": 140, "chars": 40, "events": 40},
        )
        best_tool_ratio = run(
            "best-tool-ratio.json",
            {"ttft": 150, "terminal": 150, "total": 150, "chars": 30, "events": 30},
        )
        current = run(
            "current.json",
            {"ttft": 80, "terminal": 80, "total": 80, "chars": 60, "events": 60},
        )
        set_counts(best_success["providers"][0]["tool_call"], 3, 3, 1)
        set_counts(best_tool_ratio["providers"][0]["tool_call"], 2, 3, 2)
        set_counts(current["providers"][0]["tool_call"], 2, 3, 1)

        comparison = stream_bench.compare_runs(
            current,
            [metric_baseline, best_success, best_tool_ratio],
            5.0,
            3,
        )
        text = self.text_comparison(comparison)
        ttft = self.metric_comparison(comparison, "ttft_ms")

        self.assertFalse(comparison["any_significant_improvement"])
        self.assertFalse(ttft["significant"])
        self.assertTrue(text["tool_loop_regressed"])
        self.assertIn(
            "tool_call success 2/3 < 3/3",
            text["tool_loop_regressions"],
        )
        self.assertIn(
            "tool_call tool args 0.500 < 1.000",
            text["tool_loop_regressions"],
        )

    def test_zero_throughput_baseline_does_not_create_percentage(self):
        baseline = run(
            "zero-baseline.json",
            {"ttft": 100, "terminal": 100, "total": 100, "chars": 0, "events": 50},
        )
        current = run(
            "current.json",
            {"ttft": 90, "terminal": 90, "total": 90, "chars": 0, "events": 55},
        )

        comparison = stream_bench.compare_runs(current, baseline, 5.0, 3)
        throughput = self.metric_comparison(comparison, "chars_per_sec")
        markdown = stream_bench.markdown_summary(markdown_run(current, comparison))

        self.assertEqual(throughput["baseline_median"], 0)
        self.assertEqual(throughput["current_median"], 0)
        self.assertIsNone(throughput["improvement_pct"])
        self.assertFalse(throughput["significant"])
        self.assertIn(
            "| fake_provider | text | 3/3 | 90.0 | 90.0 | 90.0 | 0.0 |",
            markdown,
        )

    def test_tiny_output_throughput_does_not_count_when_latency_regresses(self):
        baseline = run(
            "baseline.json",
            {
                "ttft": 100,
                "terminal": 100,
                "total": 100,
                "chars": 100,
                "events": 100,
                "text_chars": 1,
                "event_count": 1,
            },
        )
        current = run(
            "current.json",
            {
                "ttft": 120,
                "terminal": 120,
                "total": 120,
                "chars": 150,
                "events": 150,
                "text_chars": 1,
                "event_count": 1,
            },
        )

        comparison = stream_bench.compare_runs(current, baseline, 5.0, 3)
        chars = self.metric_comparison(comparison, "chars_per_sec")
        events = self.metric_comparison(comparison, "events_per_sec")
        markdown = stream_bench.markdown_summary(markdown_run(current, comparison))

        self.assertFalse(comparison["any_significant_improvement"])
        self.assertAlmostEqual(chars["improvement_pct"], 50.0)
        self.assertFalse(chars["significant"])
        self.assertAlmostEqual(events["improvement_pct"], 50.0)
        self.assertFalse(events["significant"])
        self.assertIn(
            "throughput gate: insufficient output and latency regressed",
            markdown,
        )

    def test_overlapping_samples_block_noisy_median_improvement(self):
        baseline = run(
            "baseline.json",
            {
                "ttft": 105,
                "terminal": 105,
                "total": 105,
                "chars": 50,
                "events": 50,
                "metric_samples": {
                    "ttft_ms": [100, 105, 110],
                    "terminal_ms": [100, 105, 110],
                    "total_ms": [100, 105, 110],
                },
            },
        )
        current = run(
            "current.json",
            {
                "ttft": 95,
                "terminal": 95,
                "total": 95,
                "chars": 50,
                "events": 50,
                "metric_samples": {
                    "ttft_ms": [90, 95, 120],
                    "terminal_ms": [90, 95, 120],
                    "total_ms": [90, 95, 120],
                },
            },
        )

        comparison = stream_bench.compare_runs(current, baseline, 5.0, 3)
        ttft = self.metric_comparison(comparison, "ttft_ms")
        markdown = stream_bench.markdown_summary(markdown_run(current, comparison))

        self.assertFalse(comparison["any_significant_improvement"])
        self.assertGreater(ttft["improvement_pct"], 5.0)
        self.assertFalse(ttft["significant"])
        self.assertIn(
            "distribution gate: pairwise dominance below threshold",
            markdown,
        )

    def test_non_overlapping_samples_allow_stable_improvement(self):
        baseline = run(
            "baseline.json",
            {
                "ttft": 102,
                "terminal": 102,
                "total": 102,
                "chars": 50,
                "events": 50,
                "metric_samples": {
                    "ttft_ms": [100, 102, 104],
                    "terminal_ms": [100, 102, 104],
                    "total_ms": [100, 102, 104],
                },
            },
        )
        current = run(
            "current.json",
            {
                "ttft": 82,
                "terminal": 82,
                "total": 82,
                "chars": 55,
                "events": 55,
                "metric_samples": {
                    "ttft_ms": [80, 82, 84],
                    "terminal_ms": [80, 82, 84],
                    "total_ms": [80, 82, 84],
                },
            },
        )

        comparison = stream_bench.compare_runs(current, baseline, 5.0, 3)
        ttft = self.metric_comparison(comparison, "ttft_ms")

        self.assertTrue(comparison["any_significant_improvement"])
        self.assertTrue(ttft["significant"])
        self.assertEqual(ttft["distribution_gate"]["dominance"], 1.0)

    def test_compare_and_markdown_tolerate_absent_scenario(self):
        baseline = run(
            "baseline.json",
            {"ttft": 100, "terminal": 100, "total": 100, "chars": 50, "events": 50},
        )
        current = run(
            "current.json",
            {"ttft": 90, "terminal": 90, "total": 90, "chars": 55, "events": 55},
        )
        del current["providers"][0]["tool_continuation"]

        comparison = stream_bench.compare_runs(current, baseline, 5.0, 3)
        missing = next(
            item
            for item in comparison["comparisons"]
            if item["provider_id"] == "fake_provider"
            and item["scenario"] == "tool_continuation"
        )
        markdown = stream_bench.markdown_summary(markdown_run(current, comparison))

        self.assertFalse(missing["enough_samples"])
        self.assertTrue(
            all(metric["baseline_median"] is None for metric in missing["metrics"])
        )
        self.assertIn("| fake_provider | tool_continuation | 0/0 |", markdown)

    def test_markdown_describes_best_of_metric_and_tool_loop_selection(self):
        baseline = run(
            "baseline.json",
            {"ttft": 100, "terminal": 100, "total": 100, "chars": 50, "events": 50},
        )
        alternate_baseline = run(
            "alternate-baseline.json",
            {"ttft": 110, "terminal": 110, "total": 110, "chars": 45, "events": 45},
        )
        current = run(
            "current.json",
            {"ttft": 90, "terminal": 90, "total": 90, "chars": 55, "events": 55},
        )
        comparison = stream_bench.compare_runs(
            current, [baseline, alternate_baseline], 5.0, 3
        )

        markdown = stream_bench.markdown_summary(markdown_run(current, comparison))

        self.assertIn(
            "best prior median per provider/scenario/metric",
            markdown,
        )
        self.assertIn(
            "tool-loop guard uses best prior success/tool ratio",
            markdown,
        )
        self.assertIn("baseline: baseline.json", markdown)

    def test_extract_ids_detects_tool_call_inside_completed_response_output(self):
        frames = [
            (
                "response.completed",
                """{"type":"response.completed","response":{"id":"resp","output":[{"type":"function_call","call_id":"call_1","name":"lookup","arguments":"{\\"q\\":\\"ok\\"}"}]}}""",
            )
        ]

        response_id, call_id, arguments, terminal, error, detected = (
            stream_bench.extract_ids(frames)
        )

        self.assertEqual(response_id, "resp")
        self.assertEqual(call_id, "call_1")
        self.assertEqual(arguments, '{"q":"ok"}')
        self.assertEqual(terminal, "response.completed")
        self.assertIsNone(error)
        self.assertTrue(detected)

    def test_repeated_tool_call_in_continuation_marks_sample_failed(self):
        sample = stream_bench.reject_repeated_tool_continuation(
            {
                "ok": True,
                "tool_call_detected": True,
                "call_id": "call_2",
                "arguments": '{"q":"ok"}',
            }
        )

        self.assertFalse(sample["ok"])
        self.assertEqual(sample["error"], "tool continuation emitted another tool call")


if __name__ == "__main__":
    unittest.main()
