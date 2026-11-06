//! Drive the packed component through `act run --mcp` with a real MCP client.
//!
//! This replaces the python fastmcp/pytest suite that lived in this
//! directory: the tests observe exactly what an agent observes, over the same
//! client stack (`rmcp`) the host bridge itself is built on.
//!
//! Env: WASM — path to the packed component (default: the component's
//!      release build output);
//!      ACT  — the act invocation (default `act`; `npx @actcore/act`, the
//!             component justfile's default, also works — whitespace-split,
//!             like the shlex.split the python conftest did).

use std::path::PathBuf;

use rmcp::{
    ServiceExt,
    model::CallToolRequestParams,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::{Value, json};

/// `().serve(transport)` hands back the client-role service running over the
/// child process: role first, the unit client handler second.
type Client = rmcp::service::RunningService<rmcp::service::RoleClient, ()>;

fn wasm_path() -> PathBuf {
    PathBuf::from(std::env::var("WASM").unwrap_or_else(|_| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../target/wasm32-wasip3/release/component_time.wasm"
        )
        .into()
    }))
}

/// The ACT invocation, honouring the same override the component justfile
/// uses. Its default there is `npx @actcore/act` — two words — which cannot
/// be `argv[0]` for a non-shell spawn, so the value is whitespace-split into
/// program + leading args.
fn act_argv() -> Vec<String> {
    std::env::var("ACT")
        .unwrap_or_else(|_| "act".into())
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// Spawn `act run <wasm> --mcp` with no grant flags.
///
/// time declares no capabilities (`act.toml` is a bare `[std] name`), so
/// there is nothing to grant — the same command line the python conftest
/// used. A headless default policy of `ask` degrades to deny only what a
/// guest reaches for, and this guest reaches for nothing.
fn act_command() -> tokio::process::Command {
    let argv = act_argv();
    let mut cmd = tokio::process::Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd.arg("run").arg(wasm_path()).arg("--mcp");
    cmd
}

async fn connect() -> Client {
    let transport = TokioChildProcess::new(act_command()).expect("spawn act run --mcp");
    ().serve(transport)
        .await
        .expect("rmcp handshake with act run --mcp")
}

fn first_text_block(result: &rmcp::model::CallToolResult) -> &rmcp::model::TextContent {
    match result.content.first() {
        Some(rmcp::model::ContentBlock::Text(t)) => t,
        other => panic!("expected the first content block to be Text, got: {other:?}"),
    }
}

async fn call_get_current_time(client: &Client, args: Value) -> rmcp::model::CallToolResult {
    let params = CallToolRequestParams::new("get_current_time")
        .with_arguments(args.as_object().cloned().unwrap_or_default());
    let result = client.call_tool(params).await.expect("call_tool");
    assert_ne!(result.is_error, Some(true), "call failed: {result:?}");
    result
}

/// The kind of a failed call may arrive on either path: as a JSON-RPC error
/// response (`ErrorData.data`) or as an isError result (`_meta`). The python
/// conftest's `expect_error` fixture handled both; so does this. `call-tool`
/// has no `result<>` wrapper, so a guest reporting a failed call can only do
/// it through `tool-event::error` — the isError path; the JSON-RPC path stays
/// handled for the non-guest failure modes (list-tools, a wasmtime trap).
async fn error_kind_of(client: &Client, params: CallToolRequestParams) -> String {
    match client.call_tool(params).await {
        Err(rmcp::ServiceError::McpError(e)) => e
            .data
            .as_ref()
            .and_then(|d| d.get("dev.actcore/error-kind"))
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| {
                format!("JSON-RPC error without dev.actcore/error-kind: {:?}", e.data)
            }),
        Ok(result) => {
            assert_eq!(result.is_error, Some(true), "call must fail: {result:?}");
            result
                .meta
                .as_ref()
                .and_then(|m| m.0.get("dev.actcore/error-kind"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| {
                    format!("isError result without dev.actcore/error-kind: {result:?}")
                })
        }
        Err(other) => panic!("unexpected transport failure: {other:?}"),
    }
}

/// The manifest probe from python test_info.py — and the fast-fail the
/// python `wasm_path` fixture provided: an unpacked wasm (raw `cargo build`
/// output, no `act:component` section) declares no ceiling, so failures
/// would point anywhere but at the missing metadata. The justfile's
/// `test: build` ordering exists so this test finds a packed artifact.
#[test]
fn manifest_reports_name_and_version() {
    let output = {
        let argv = act_argv();
        let mut cmd = std::process::Command::new(&argv[0]);
        cmd.args(&argv[1..]);
        cmd.args(["inspect", "component-manifest"])
            .arg(wasm_path())
            .output()
            .expect("run act inspect component-manifest")
    };
    assert!(
        output.status.success(),
        "inspect failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest: Value = serde_json::from_slice(&output.stdout).expect("manifest is JSON");
    assert_eq!(
        manifest["std"]["name"], "time",
        "packed manifest must carry the component name"
    );
    assert!(
        manifest["std"]["version"].is_string(),
        "packed manifest must carry a version, got: {}",
        manifest["std"]["version"]
    );
}

/// python test_tools.py::test_component_exposes_its_tools — hurl tools.hurl:
/// `$.tools` count >= 1.
#[tokio::test]
async fn component_exposes_its_tools() {
    let client = connect().await;
    let tools = client.list_all_tools().await.expect("list_all_tools");
    assert!(
        !tools.is_empty(),
        "component must expose at least one tool"
    );
    client.cancel().await.ok();
}

/// python test_tools.py::test_lists_exactly_get_current_time — hurl
/// list_tools.hurl: exact count and name — time exposes exactly one tool.
#[tokio::test]
async fn lists_exactly_get_current_time() {
    let client = connect().await;
    let tools = client.list_all_tools().await.expect("list_all_tools");
    assert_eq!(tools.len(), 1, "time exposes exactly one tool");
    assert_eq!(tools[0].name, "get_current_time");
    client.cancel().await.ok();
}

/// python test_get_current_time.py::test_default_is_utc — hurl 'UTC time'
/// block: no args -> an RFC 3339 timestamp.
#[tokio::test]
async fn default_is_utc() {
    let client = connect().await;
    let result = call_get_current_time(&client, json!({})).await;
    assert!(
        first_text_block(&result).text.contains('T'),
        "expected an RFC 3339 timestamp, got: {:?}",
        first_text_block(&result).text
    );
    client.cancel().await.ok();
}

/// python test_get_current_time.py::test_with_timezone — hurl 'With
/// timezone' block: Europe/Moscow.
///
/// Moscow sits at UTC+03:00 today but carried +04:00 under permanent DST
/// from 2011-2014, so the original regex tolerates either offset — kept
/// rather than pinned to the current value. hurl's `matches` predicate is an
/// unanchored substring search, and `re.search(r"\+0[34]:00", text)` matches
/// exactly the two literals below, so the substring disjunction is the same
/// assertion without pulling in the regex crate.
#[tokio::test]
async fn with_timezone() {
    let client = connect().await;
    let result = call_get_current_time(&client, json!({"timezone": "Europe/Moscow"})).await;
    let text = &first_text_block(&result).text;
    assert!(
        text.contains("+03:00") || text.contains("+04:00"),
        "expected a +03:00/+04:00 offset fragment in: {text}"
    );
    client.cancel().await.ok();
}

/// python test_get_current_time.py::test_rejects_an_invalid_timezone —
/// hurl 'Invalid timezone' block: HTTP 422 + `$.error.kind` ==
/// std:invalid-args was ACT-HTTP's projection of the same
/// tool-event::error the MCP transport surfaces as an isError result;
/// error_kind_of covers both paths.
#[tokio::test]
async fn rejects_an_invalid_timezone() {
    let client = connect().await;
    let args = json!({"timezone": "Not/A/Zone"});
    let params = CallToolRequestParams::new("get_current_time")
        .with_arguments(args.as_object().unwrap().clone());
    let kind = error_kind_of(&client, params).await;
    assert_eq!(
        kind, "std:invalid-args",
        "an unknown IANA zone must surface as std:invalid-args"
    );
    client.cancel().await.ok();
}

/// python test_get_current_time.py::test_safe_to_call_repeatedly — hurl
/// 'QUERY method (safe, cacheable)' block: ACT-HTTP re-issued the same call
/// over the QUERY verb to prove this read_only tool is reachable safely that
/// way too. MCP's `tools/call` has no verb to vary — there is exactly one
/// way to invoke a tool — so the faithful counterpart is confirming a
/// second call is just as well-formed as the first. The python body issued
/// one call per fresh process; the repetition the name claims is made
/// literal here, within one host process.
#[tokio::test]
async fn safe_to_call_repeatedly() {
    let client = connect().await;
    for call in 1..=2 {
        let result = call_get_current_time(&client, json!({})).await;
        assert!(
            first_text_block(&result).text.contains('T'),
            "call #{call} must return an RFC 3339 timestamp, got: {:?}",
            first_text_block(&result).text
        );
    }
    client.cancel().await.ok();
}
