// MCP server over stdio for driving and looking at the game while developing the scanner.
// Coordinates are 1080p client pixels. Holding F10 aborts (the server exits).
use base64::Engine;
use hf_market::{ctl, game::Game};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router,
};

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
        description = "commands separated by ';': click x y | rclick x y | dclick x y | move x y | drag x0 y0 x1 y1 | scroll x y notches(+down) | key chord (esc, enter, alt+y, ctrl+a) | type text | wait ms | ocr x0 x1 y0 y1 | num x0 x1 y0 y1 | looks refname [dy]"
    )]
    script: String,
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
        let mut g = Game::find();
        let mut v = vec![];
        if let (Some(x), Some(y), Some(w), Some(h)) = (s.x, s.y, s.w, s.h) {
            v.extend([x, y, w, h]);
        }
        v.extend(s.scale);
        let img = ctl::shot(&mut g, &v);
        let mut png = std::io::Cursor::new(vec![]);
        img.write_to(&mut png, image::ImageFormat::Png).unwrap();
        let data = base64::engine::general_purpose::STANDARD.encode(png.into_inner());
        CallToolResult::success(vec![ContentBlock::image(data, "image/png")])
    }

    #[tool(
        description = "Run input and reading commands on the game; returns what the reading commands read"
    )]
    fn run(&self, Parameters(s): Parameters<Script>) -> String {
        let mut g = Game::find();
        let out = ctl::run(&mut g, &s.script);
        if out.is_empty() { "ok".into() } else { out }
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
    let server = Market {
        tool_router: Market::tool_router(),
    }
    .serve(rmcp::transport::stdio())
    .await
    .unwrap();
    server.waiting().await.unwrap();
}
