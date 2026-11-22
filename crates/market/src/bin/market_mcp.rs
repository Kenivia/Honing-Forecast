// MCP server over stdio for driving and looking at the game while developing the scanner. Coordinates are 1080p
// client pixels. Holding F10 aborts a local script. `--remote <machine>` drives that machine's market_agent
// through the relay instead (ignored/market/relay_admin.json: {"url", "token"}).
use base64::Engine;
use hf_market::{assets, ctl, game::Game};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router,
};
use serde_json::{Value, json};
use std::sync::OnceLock;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct Shot {
    #[schemars(description = "crop left, client pixels")]
    x: Option<u32>,
    y: Option<u32>,
    w: Option<u32>,
    h: Option<u32>,
    #[schemars(description = "shrink by this whole factor, e.g. 2 for half size")]
    scale: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct Script {
    #[schemars(
        description = "commands separated by ';': click x y | rclick x y | dclick x y | move x y | drag x0 y0 x1 y1 | scroll x y notches(+down) | key chord (esc, enter, alt+y, ctrl+a) | type text | wait ms | ocr x0 x1 y0 y1 | num x0 x1 y0 y1 | looks refname [dy] | ref name x y w h (saves a reference crop to the machine's profile)"
    )]
    script: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct Control {
    #[schemars(
        description = "status | start | pause | launch (the game through Steam) | log | setup (screenshot after every unit of work; with on) | sync (upload the profile's templates)"
    )]
    op: String,
    #[schemars(description = "log: how many lines (default 30)")]
    lines: Option<u32>,
    #[schemars(description = "setup: on or off")]
    on: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct Files {
    #[schemars(
        description = "key prefix, e.g. '<machine>/shots/' (error screenshots), '<machine>/steps/' (setup mode), '<machine>/profile/'"
    )]
    prefix: String,
    #[schemars(
        description = "download every listed file to ignored/market/relay_files/<key> instead of listing"
    )]
    download: Option<bool>,
}

struct Remote {
    url: String,
    token: String,
    machine: String,
}

static REMOTE: OnceLock<Option<Remote>> = OnceLock::new();

fn remote() -> Option<&'static Remote> {
    REMOTE.get().unwrap().as_ref()
}

fn auth(r: &Remote) -> String {
    format!("Bearer {}", r.token)
}

fn cmd(r: &Remote, body: Value) -> Value {
    let reply = ureq::post(&format!("{}/cmd/{}", r.url, r.machine))
        .header("Authorization", &auth(r))
        .content_type("application/json")
        .send(&body.to_string());
    match reply {
        Ok(mut res) => serde_json::from_str(&res.body_mut().read_to_string().unwrap()).unwrap(),
        Err(e) => json!({"ok": false, "text": format!("relay: {e}")}),
    }
}

fn get(r: &Remote, path: &str) -> Result<Vec<u8>, String> {
    let mut res = ureq::get(&format!("{}{path}", r.url))
        .header("Authorization", &auth(r))
        .call()
        .map_err(|e| e.to_string())?;
    Ok(res
        .body_mut()
        .with_config()
        .limit(64 << 20)
        .read_to_vec()
        .unwrap())
}

#[derive(Debug, Clone)]
struct Market {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Market {
    #[tool(
        description = "Screenshot of the game (1920x1080 client), optionally cropped and shrunk; PNG"
    )]
    fn screenshot(&self, Parameters(s): Parameters<Shot>) -> CallToolResult {
        let data = if let Some(r) = remote() {
            let reply = cmd(
                r,
                json!({"op": "shot", "x": s.x, "y": s.y, "w": s.w, "h": s.h, "scale": s.scale}),
            );
            match reply["png"].as_str() {
                Some(p) => p.to_string(),
                None => {
                    return CallToolResult::error(vec![ContentBlock::text(
                        reply["text"].as_str().unwrap_or("").to_string(),
                    )]);
                }
            }
        } else {
            let mut g = Game::find();
            let mut v = vec![];
            if let (Some(x), Some(y), Some(w), Some(h)) = (s.x, s.y, s.w, s.h) {
                v.extend([x, y, w, h]);
            }
            v.extend(s.scale);
            let img = ctl::shot(&mut g, &v);
            let mut png = std::io::Cursor::new(vec![]);
            img.write_to(&mut png, image::ImageFormat::Png).unwrap();
            base64::engine::general_purpose::STANDARD.encode(png.into_inner())
        };
        CallToolResult::success(vec![ContentBlock::image(data, "image/png")])
    }

    #[tool(
        description = "Run input and reading commands on the game; returns what the reading commands read. Remote: only while the agent is paused"
    )]
    fn run(&self, Parameters(s): Parameters<Script>) -> String {
        if let Some(r) = remote() {
            let reply = cmd(r, json!({"op": "run", "script": s.script}));
            return reply["text"].as_str().unwrap_or("").to_string();
        }
        let mut g = Game::find();
        let out = ctl::run(&mut g, &s.script);
        if out.is_empty() { "ok".into() } else { out }
    }

    #[tool(description = "Remote only: control the machine's market_agent")]
    fn control(&self, Parameters(c): Parameters<Control>) -> String {
        let Some(r) = remote() else {
            return "remote only (market_mcp --remote <machine>)".into();
        };
        let reply = cmd(r, json!({"op": c.op, "lines": c.lines, "on": c.on}));
        reply["text"].as_str().unwrap_or("").to_string()
    }

    #[tool(description = "Remote only: list or download files in the relay's store")]
    fn files(&self, Parameters(f): Parameters<Files>) -> String {
        let Some(r) = remote() else {
            return "remote only (market_mcp --remote <machine>)".into();
        };
        let list: Value = match get(r, &format!("/files?prefix={}", f.prefix)) {
            Ok(b) => serde_json::from_slice(&b).unwrap(),
            Err(e) => return e,
        };
        let keys: Vec<&str> = list["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["key"].as_str().unwrap())
            .collect();
        if !f.download.unwrap_or(false) {
            return keys.join("\n");
        }
        let root = assets::data_dir("relay_files");
        for k in &keys {
            let p = root.join(k);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            let enc: String = k
                .bytes()
                .map(|b| match b {
                    b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'/' | b'@' => {
                        (b as char).to_string()
                    }
                    _ => format!("%{b:02X}"),
                })
                .collect();
            std::fs::write(&p, get(r, &format!("/files/{enc}")).unwrap()).unwrap();
        }
        format!("{} files in {}", keys.len(), root.display())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Market {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Drives Lost Ark (1080p, client pixels). Alt+Y toggles the Market. Never click Buy/Bid/Confirm buttons.")
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let machine = args
        .iter()
        .position(|a| a == "--remote")
        .map(|i| args[i + 1].clone());
    REMOTE
        .set(machine.map(|machine| {
            let cfg: Value = serde_json::from_str(
                &std::fs::read_to_string(assets::data().join("relay_admin.json")).unwrap(),
            )
            .unwrap();
            Remote {
                url: cfg["url"].as_str().unwrap().to_string(),
                token: cfg["token"].as_str().unwrap().to_string(),
                machine,
            }
        }))
        .ok();
    let server = Market {
        tool_router: Market::tool_router(),
    }
    .serve(rmcp::transport::stdio())
    .await
    .unwrap();
    server.waiting().await.unwrap();
}
