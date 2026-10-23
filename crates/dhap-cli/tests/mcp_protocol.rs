use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn test_mcp_protocol_lifecycle() {
    let bin_path = env!("CARGO_BIN_EXE_dhap");
    let mut child = Command::new(bin_path)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn dhap mcp server");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // 1. initialize
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "1.0" }
        }
    });
    writeln!(stdin, "{}", init_req).expect("Failed to write init request");
    stdin.flush().expect("Failed to flush stdin");

    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");
    let init_resp: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse init response JSON");
    assert_eq!(init_resp["id"], 1);
    assert_eq!(init_resp["result"]["serverInfo"]["name"], "dhap");

    // 2. ping
    let ping_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "ping"
    });
    writeln!(stdin, "{}", ping_req).expect("Failed to write ping request");
    stdin.flush().expect("Failed to flush stdin");

    response_line.clear();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read ping response");
    let ping_resp: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse ping response JSON");
    assert_eq!(ping_resp["id"], 2);

    // 3. tools/list
    let list_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/list"
    });
    writeln!(stdin, "{}", list_req).expect("Failed to write tools/list request");
    stdin.flush().expect("Failed to flush stdin");

    response_line.clear();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read tools/list response");
    let list_resp: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse tools/list response JSON");
    assert_eq!(list_resp["id"], 3);
    let tools = list_resp["result"]["tools"]
        .as_array()
        .expect("tools should be an array");
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(tool_names.contains(&"dhap_validate_project"));
    assert!(tool_names.contains(&"dhap_inspect_project"));
    assert!(tool_names.contains(&"dhap_scan_secrets"));
    assert!(tool_names.contains(&"dhap_export_guide"));
    assert!(tool_names.contains(&"dhap_project_stats"));
    assert!(tool_names.contains(&"dhap_diff_projects"));
    assert!(tool_names.contains(&"dhap_summarize_project"));
    assert!(tool_names.contains(&"dhap_generate_gif"));

    // 4. tools/call: dhap_scan_secrets with clean text
    let clean_call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "dhap_scan_secrets",
            "arguments": {
                "text": "Open your terminal and type `dhap --version`."
            }
        }
    });
    writeln!(stdin, "{}", clean_call).expect("Failed to write scan call");
    stdin.flush().expect("Failed to flush stdin");

    response_line.clear();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read scan call response");
    let scan_resp: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse scan response JSON");
    assert_eq!(scan_resp["id"], 4);
    let content_text = scan_resp["result"]["content"][0]["text"]
        .as_str()
        .expect("text content expected");
    assert!(content_text.contains("No unredacted secrets or credentials detected"));

    // 5. tools/call: dhap_scan_secrets with GitHub Personal Access Token
    let dirty_call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "dhap_scan_secrets",
            "arguments": {
                "text": "Use token ghp_111122223333444455556666777788889999 to authenticate."
            }
        }
    });
    writeln!(stdin, "{}", dirty_call).expect("Failed to write dirty scan call");
    stdin.flush().expect("Failed to flush stdin");

    response_line.clear();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read dirty scan call response");
    let dirty_resp: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse dirty scan response JSON");
    assert_eq!(dirty_resp["id"], 5);
    let dirty_content_text = dirty_resp["result"]["content"][0]["text"]
        .as_str()
        .expect("text content expected");
    assert!(dirty_content_text.contains("Found 1 potential secret"));
    assert!(dirty_content_text.contains("GitHub Access Token"));

    // Drop stdin to signal EOF and terminate the server gracefully
    drop(stdin);
    let status = child.wait().expect("Failed to wait on child process");
    assert!(status.success());
}
