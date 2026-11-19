//! Reads the pinned graph and emits the photo work list for the operator script.

use tomi_api::config::GraphConfig;
use tomi_api::dish_photos::{Manifest, SHIPPED};
use tomi_api::graph::files;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GraphConfig { dir: Some("graph".into()), allow_draft: true, required: true };
    let graph = files::start(include_str!("../graph.lock"), &config)?.ok_or("graph required")?;
    let manifest = Manifest::parse(SHIPPED)?;
    if std::env::args().nth(1).as_deref() == Some("validate") {
        manifest.validate(&graph)?;
    } else {
        serde_json::to_writer(std::io::stdout().lock(), &manifest.plan(&graph))?;
    }
    Ok(())
}
