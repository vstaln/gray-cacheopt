//! `gray-cacheopt` sidecar: JSON-lines on stdio. No args (or `sidecar`)
//! runs the wire loop; `stats`/`reset` are the standalone CLI.

use gray_cacheopt as g;
use serde_json::{json, Value};
use std::io::{BufRead, Write};

fn session_id(req: &Value) -> &str {
    req["params"]["session"]["id"].as_str().unwrap_or("")
}

fn run_command(req: &Value) -> Result<Value, String> {
    let argv: Vec<&str> = req["params"]["argv"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    match argv.first().copied() {
        Some("reset") => {
            g::save_stats(&g::Stats::default());
            Ok(json!({"text": "cacheopt: stats cleared"}))
        }
        _ => Ok(json!({"text": g::format_stats(&g::load_stats())})),
    }
}

fn on_notify(req: &Value) {
    if req["method"] != "event/notify" || req["params"]["type"] != "turn_end" {
        return;
    }
    let mut s = g::load_stats();
    g::record_turn(&mut s, &req["params"]["usage"]);
    g::save_stats(&s);
}

fn sidecar() -> std::io::Result<()> {
    let watch = g::Watch::default();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let Ok(req) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if req["method"] == "plugin/shutdown" {
            break;
        }
        if req.get("id").is_none() {
            on_notify(&req); // event/notify: fire-and-forget
            continue;
        }
        let id = req["id"].clone();
        let result = match req["method"].as_str().unwrap_or("") {
            "plugin/manifest" => Ok(g::manifest()),
            "prompt/context" => Ok(json!({
                "text": watch.take_warning(session_id(&req)).unwrap_or_default()
            })),
            "context/build" => {
                watch.observe(session_id(&req), &req["params"]["messages"]);
                Ok(json!({})) // observe-only: keep the outbound list
            }
            "command/run" => run_command(&req),
            _ => Err(format!("unsupported method {}", req["method"])),
        };
        let reply = match result {
            Ok(r) => json!({"id": id, "result": r}),
            Err(e) => json!({"id": id, "error": {"code": -32601, "message": e}}),
        };
        let mut o = stdout.lock();
        let _ = writeln!(o, "{reply}");
        let _ = o.flush();
    }
    Ok(())
}

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "manifest" => println!("{}", g::manifest()),
        "stats" | "cachestats" => print!("{}", g::format_stats(&g::load_stats())),
        "reset" => {
            g::save_stats(&g::Stats::default());
            println!("cacheopt: stats cleared");
        }
        _ => {
            if let Err(e) = sidecar() {
                eprintln!("gray-cacheopt: {e}");
            }
        }
    }
}
