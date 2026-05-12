use include_dir::{Dir, include_dir};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Action {
    #[serde(rename = "display_name")]
    pub _display_name: String,
    pub frames: Vec<String>,
    pub frame_duration_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Manifest {
    pub cell_size: u32,
    pub actions: HashMap<String, Action>,
}

pub struct PetConfig {
    pub _name: String,
    pub manifest: Manifest,
    pub dir: &'static Dir<'static>,
}

static GP_CHAN_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../GP-Chan");
static GEMMI_CHAN_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../GEMMI-Chan");

impl PetConfig {
    pub fn load_embedded(name: &str) -> Option<Self> {
        let dir = match name {
            "GP-Chan" => &GP_CHAN_DIR,
            "GEMMI-Chan" => &GEMMI_CHAN_DIR,
            _ => return None,
        };

        let manifest_file = dir.get_file("assets/generated/manifest.json")?;
        let content = manifest_file.contents_utf8()?;
        let manifest: Manifest = serde_json::from_str(content).ok()?;

        Some(Self {
            _name: name.to_string(),
            manifest,
            dir,
        })
    }
}
