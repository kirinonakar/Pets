use crate::config::PetConfig;
use std::collections::HashMap;
use egui::{TextureHandle, Context};

pub struct PetState {
    pub config: PetConfig,
    pub current_action: String,
    pub frame_index: usize,
    pub last_frame_time: f64,
    pub textures: HashMap<String, Vec<TextureHandle>>,
    pub facing_right: bool,
    pub _position: egui::Pos2,
}

impl PetState {
    pub fn new(config: PetConfig, ctx: &Context) -> Self {
        let mut textures = HashMap::new();
        
        for (action_name, action) in &config.manifest.actions {
            let mut action_textures = Vec::new();
            for frame_path in &action.frames {
                // Load from embedded dir
                if let Some(file) = config.dir.get_file(frame_path) {
                    if let Ok(image_data) = image::load_from_memory(file.contents()) {
                        let rgba = image_data.to_rgba8();
                        let pixels = rgba.as_flat_samples();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [rgba.width() as usize, rgba.height() as usize],
                            pixels.as_slice(),
                        );
                        let handle = ctx.load_texture(
                            format!("{}_{}_{}", config._name, action_name, frame_path),
                            color_image,
                            egui::TextureOptions::default(),
                        );
                        action_textures.push(handle);
                    } else {
                        eprintln!("Failed to decode image: {}", frame_path);
                    }
                } else {
                    eprintln!("File not found in embedded dir: {}", frame_path);
                }
            }
            if !action_textures.is_empty() {
                textures.insert(action_name.clone(), action_textures);
            }
        }

        Self {
            config,
            current_action: "idle".to_string(),
            frame_index: 0,
            last_frame_time: 0.0,
            textures,
            facing_right: true,
            _position: egui::pos2(100.0, 100.0),
        }
    }

    pub fn update_animation(&mut self, time: f64) {
        if let Some(action) = self.config.manifest.actions.get(&self.current_action) {
            if let Some(textures) = self.textures.get(&self.current_action) {
                let len = textures.len();
                if len > 1 {
                    let duration = action.frame_duration_ms as f64 / 1000.0;
                    if duration > 0.0 {
                        // If last_frame_time is 0, initialize it to current time to start fresh
                        if self.last_frame_time == 0.0 {
                            self.last_frame_time = time;
                        }
                        
                        let elapsed = time - self.last_frame_time;
                        if elapsed >= duration {
                            let advance = (elapsed / duration).floor() as usize;
                            self.frame_index = (self.frame_index + advance) % len;
                            self.last_frame_time += advance as f64 * duration;
                        }
                    }
                } else {
                    self.frame_index = 0;
                }
            }
        }
    }

    pub fn current_texture(&self) -> Option<&TextureHandle> {
        if let Some(frames) = self.textures.get(&self.current_action) {
            if frames.is_empty() { return None; }
            let idx = self.frame_index % frames.len();
            frames.get(idx)
        } else {
            None
        }
    }

    pub fn set_action(&mut self, action: &str, time: f64) {
        if self.current_action != action {
            if self.textures.contains_key(action) {
                self.current_action = action.to_string();
            } else {
                // Fallback to idle if the pet doesn't support this action
                self.current_action = "idle".to_string();
            }
            self.frame_index = 0;
            self.last_frame_time = time;
        }
    }
}
