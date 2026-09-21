#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tasks_file="$repository_root/benchmark/tasks.json"
model="${CODEX_MODEL:-gpt-5.6-luna}"
reasoning="${CODEX_REASONING:-low}"
repetitions="${BENCHMARK_REPETITIONS:-1}"
output="${BENCHMARK_OUTPUT:-$repository_root/benchmark/results/latest.json}"
log_dir="${BENCHMARK_LOG_DIR:-}"
raw_root="$repository_root/benchmark/.runs/$(date -u +%Y%m%dT%H%M%SZ)"
snapshot="$(mktemp -d /tmp/jgrep-benchmark.XXXXXXXX)"
snapshot_archive="$snapshot/source.tar"

cleanup() {
    case "$snapshot" in
        /tmp/jgrep-benchmark.*) rm -rf -- "$snapshot" ;;
        *) printf 'refusing to remove unexpected path: %s\n' "$snapshot" >&2 ;;
    esac
}
trap cleanup EXIT

for command in codex git jq jgrep rg tar; do
    command -v "$command" >/dev/null 2>&1 || {
        printf 'required command not found: %s\n' "$command" >&2
        exit 2
    }
done

case "$repetitions" in
    ''|*[!0-9]*|0) printf 'BENCHMARK_REPETITIONS must be a positive integer\n' >&2; exit 2 ;;
esac

commit="$(git -C "$repository_root" rev-parse HEAD)"
mkdir -p "$raw_root" "$(dirname "$output")"
if [[ -n "$log_dir" ]]; then
    mkdir -p "$log_dir"
fi
git -C "$repository_root" archive --format=tar --output="$snapshot_archive" "$commit"
tar -xf "$snapshot_archive" --exclude=AGENTS.md -C "$snapshot"
rm -f -- "$snapshot_archive"
records="$raw_root/records.jsonl"

baseline_instructions="Run exactly one shell command: rg --hidden --sort path -n -F -g '!benchmark/**' -- NEEDLE . (replace NEEDLE with the shell-quoted supplied needle). Never run jgrep, grep, find, sed, ls, or any other command. Do not truncate or pipe the search output."
jgrep_instructions="Run exactly one shell command: jgrep --ai --ai-max-results 25 -r --exclude 'benchmark/**' -F -e NEEDLE . (replace NEEDLE with the shell-quoted supplied needle). Never run rg, grep, find, sed, ls, or any other command. Do not raise the result cap."

run_one() {
    local mode="$1"
    local repetition="$2"
    local task_id="$3"
    local needle="$4"
    local request="$5"
    local instructions log stderr_log prompt checked_log checked_stderr_log log_reference stderr_log_reference

    if [[ "$mode" == "rg" ]]; then
        instructions="$baseline_instructions"
    else
        instructions="$jgrep_instructions"
    fi

    log="$raw_root/${task_id}.${mode}.${repetition}.jsonl"
    stderr_log="$raw_root/${task_id}.${mode}.${repetition}.stderr"
    prompt="Read-only benchmark: do not modify files. ${instructions} The exact needle is: ${needle}. Task: ${request}"

    codex exec \
        --ephemeral \
        --ignore-user-config \
        --ignore-rules \
        --skip-git-repo-check \
        --json \
        --color never \
        --sandbox read-only \
        --model "$model" \
        -c "model_reasoning_effort=\"$reasoning\"" \
        -C "$snapshot" \
        "$prompt" </dev/null >"$log" 2>"$stderr_log"

    log_reference=""
    stderr_log_reference=""
    if [[ -n "$log_dir" ]]; then
        checked_log="$log_dir/${task_id}.${mode}.${repetition}.log"
        checked_stderr_log="$log_dir/${task_id}.${mode}.${repetition}.stderr.log"
        cp "$log" "$checked_log"
        cp "$stderr_log" "$checked_stderr_log"
        case "$checked_log" in
            "$repository_root"/*) log_reference="${checked_log#"$repository_root/"}" ;;
            *) log_reference="$checked_log" ;;
        esac
        case "$checked_stderr_log" in
            "$repository_root"/*) stderr_log_reference="${checked_stderr_log#"$repository_root/"}" ;;
            *) stderr_log_reference="$checked_stderr_log" ;;
        esac
    fi

    jq -s \
        --arg mode "$mode" \
        --arg task "$task_id" \
        --arg prompt "$prompt" \
        --arg log_file "$log_reference" \
        --arg stderr_log_file "$stderr_log_reference" \
        --argjson repetition "$repetition" '
        ([.[] | select(.type == "turn.completed") | .usage] | last) as $usage
        | {
            task: $task,
            mode: $mode,
            repetition: $repetition,
            prompt: $prompt,
            log_file: (if $log_file == "" then null else $log_file end),
            stderr_log_file: (if $stderr_log_file == "" then null else $stderr_log_file end),
            usage: $usage,
            total_tokens: ($usage.input_tokens + $usage.output_tokens),
            non_cached_input_tokens: ($usage.input_tokens - $usage.cached_input_tokens),
            tool_output_chars: ([.[] | select(.type == "item.completed" and .item.type == "command_execution") | (.item.aggregated_output // "") | length] | add // 0),
            commands: [.[] | select(.type == "item.completed" and .item.type == "command_execution") | .item.command],
            answer: ([.[] | select(.type == "item.completed" and .item.type == "agent_message") | .item.text] | last)
        }' "$log" >>"$records"
}

for repetition in $(seq 1 "$repetitions"); do
    while IFS=$'\t' read -r task_id needle request; do
        run_one rg "$repetition" "$task_id" "$needle" "$request"
        run_one jgrep "$repetition" "$task_id" "$needle" "$request"
    done < <(jq -r '.[] | [.id, .needle, .request] | @tsv' "$tasks_file")
done

jq -s \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg commit "$commit" \
    --arg codex_version "$(codex --version)" \
    --arg jgrep_version "$(jgrep --version)" \
    --arg model "$model" \
    --arg reasoning "$reasoning" \
    --argjson repetitions "$repetitions" '
    def total_for($mode; $field): map(select(.mode == $mode) | .[$field]) | add;
    (total_for("rg"; "total_tokens")) as $rg_total
    | (total_for("jgrep"; "total_tokens")) as $jgrep_total
    | (total_for("rg"; "non_cached_input_tokens")) as $rg_non_cached
    | (total_for("jgrep"; "non_cached_input_tokens")) as $jgrep_non_cached
    | {
        generated_at: $generated_at,
        source_commit: $commit,
        codex_version: $codex_version,
        jgrep_version: $jgrep_version,
        model: $model,
        reasoning_effort: $reasoning,
        repetitions: $repetitions,
        runs: .,
        aggregate: {
            rg: {
                total_tokens: $rg_total,
                input_tokens: total_for("rg"; "usage")
            },
            jgrep: {
                total_tokens: $jgrep_total,
                input_tokens: total_for("jgrep"; "usage")
            },
            total_token_reduction_percent: (100 * ($rg_total - $jgrep_total) / $rg_total),
            non_cached_input_token_reduction_percent: (100 * ($rg_non_cached - $jgrep_non_cached) / $rg_non_cached),
            tool_output_char_reduction_percent: (100 * (total_for("rg"; "tool_output_chars") - total_for("jgrep"; "tool_output_chars")) / total_for("rg"; "tool_output_chars"))
        }
    }
    | .aggregate.rg.input_tokens = ([.runs[] | select(.mode == "rg") | .usage.input_tokens] | add)
    | .aggregate.rg.cached_input_tokens = ([.runs[] | select(.mode == "rg") | .usage.cached_input_tokens] | add)
    | .aggregate.rg.output_tokens = ([.runs[] | select(.mode == "rg") | .usage.output_tokens] | add)
    | .aggregate.rg.non_cached_input_tokens = ([.runs[] | select(.mode == "rg") | .non_cached_input_tokens] | add)
    | .aggregate.rg.tool_output_chars = ([.runs[] | select(.mode == "rg") | .tool_output_chars] | add)
    | .aggregate.jgrep.input_tokens = ([.runs[] | select(.mode == "jgrep") | .usage.input_tokens] | add)
    | .aggregate.jgrep.cached_input_tokens = ([.runs[] | select(.mode == "jgrep") | .usage.cached_input_tokens] | add)
    | .aggregate.jgrep.output_tokens = ([.runs[] | select(.mode == "jgrep") | .usage.output_tokens] | add)
    | .aggregate.jgrep.non_cached_input_tokens = ([.runs[] | select(.mode == "jgrep") | .non_cached_input_tokens] | add)
    | .aggregate.jgrep.tool_output_chars = ([.runs[] | select(.mode == "jgrep") | .tool_output_chars] | add)
    ' "$records" >"$output"

printf 'wrote %s\n' "$output"
jq '.aggregate' "$output"
