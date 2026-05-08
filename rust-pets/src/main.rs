mod monitor;
mod config;
mod pet;

use eframe::egui;
use monitor::{Monitor, SystemStats};
use config::PetConfig;
use pet::PetState;
use std::path::Path;


fn get_action_label(action: &str) -> &str {
    match action {
        "idle" => "멍...",
        "wave" => "하이",
        "think" => "흠...",
        "typing" => "타닥타닥",
        "cheer" => "힘내!",
        "sit" => "쉬는중",
        "sleep" => "Zzz..",
        "pout" => "흥...",
        "surprise" => "헉!",
        "sweep" => "청소중",
        "walk" => "순찰중",
        "half_right" => "반만인정",
        "welcome_agi" => "AGI 가즈아",
        "agi_box" => "박스행",
        "drag_dangle" => "대롱대롱",
        "scroll_tickle" => "아ㅋㅋ",
        "bonk" => "아야!",
        _ => action,
    }
}

struct PetApp {
    monitor: Monitor,
    stats: SystemStats,
    pet: Option<PetState>,
    available_pets: Vec<String>,
    current_pet_name: String,
    last_update: f64,
    pending_pet_switch: Option<String>,
    status_text: String,
    status_timeout: f64,
    mouse_follow: bool,
    show_stats: bool,
    pending_action: Option<String>,
}

impl PetApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Setup Korean Font
        let mut fonts = egui::FontDefinitions::default();
        let font_path = "C:\\Windows\\Fonts\\malgun.ttf";
        if Path::new(font_path).exists() {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "Malgun".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                fonts.families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "Malgun".to_owned());
                fonts.families
                    .get_mut(&egui::FontFamily::Monospace)
                    .unwrap()
                    .push("Malgun".to_owned());
            }
        }
        cc.egui_ctx.set_fonts(fonts);

        let mut monitor = Monitor::new();
        let stats = monitor.update();
        
        let available_pets = vec!["GP-Chan".to_string(), "GEMMI-Chan".to_string()];
        let current_pet_name = "GP-Chan".to_string();
        
        let pet = PetConfig::load_embedded(&current_pet_name)
            .map(|config| PetState::new(config, &cc.egui_ctx));

        let status_text = if pet.is_none() { "로드 실패!".to_string() } else { "부팅 완료!".to_string() };

        Self {
            monitor,
            stats,
            pet,
            available_pets,
            current_pet_name,
            last_update: 0.0,
            pending_pet_switch: None,
            status_text,
            status_timeout: 5.0,
            mouse_follow: true,
            show_stats: true,
            pending_action: None,
        }
    }

    fn switch_pet(&mut self, ctx: &egui::Context, name: &str) {
        if self.current_pet_name == name { return; }
        if let Some(config) = PetConfig::load_embedded(name) {
            self.pet = Some(PetState::new(config, ctx));
            self.current_pet_name = name.to_string();
        }
    }
}

impl eframe::App for PetApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // Fully transparent
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let time = ctx.input(|i| i.time);
        
        // Update stats every 2 seconds
        if time - self.last_update > 2.0 {
            self.stats = self.monitor.update();
            self.last_update = time;
        }

        if let Some(pet) = &mut self.pet {
            pet.update_animation(time);
        }

        // Window properties
        ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));

        // Mouse Following & Window Movement Logic
        if self.mouse_follow {
            if let Some(mouse_pos) = ctx.input(|i| i.pointer.latest_pos()) {
                // Adjust center based on left-column placement (pet is ~160x160 at the bottom-left)
                let pet_center = egui::vec2(80.0, 300.0); 
                let dist_vec = mouse_pos.to_vec2() - pet_center;
                let dist = dist_vec.length();

                if let Some(pet) = &mut self.pet {
                    // 1. Facing logic
                    if dist_vec.x < -20.0 { pet.facing_right = false; }
                    else if dist_vec.x > 20.0 { pet.facing_right = true; }

                    // 2. Movement & Animation logic
                    let in_window = mouse_pos.x >= 0.0 && mouse_pos.x <= 300.0 
                                 && mouse_pos.y >= 0.0 && mouse_pos.y <= 500.0;

                    if !in_window && dist > 20.0 { 
                        if dist > 500.0 {
                            // Too far! Give up
                            if pet.current_action == "walk" {
                                pet.set_action("idle");
                                self.status_text = "놓침...".to_string();
                                self.status_timeout = time + 2.0;
                            }
                        } else {
                            // Switch to walk animation
                            if pet.current_action != "walk" && self.pending_action.is_none() {
                                pet.set_action("walk");
                            }

                            if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
                                let current_pos = outer_rect.min;
                                let target_pos = current_pos + dist_vec;
                                let lerp_factor = 0.05; // Slightly faster for responsiveness
                                let new_pos = current_pos + (target_pos - current_pos) * lerp_factor;
                                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(new_pos));
                            }
                        }
                    } else {
                        // Stop walking when inside or very close
                        if pet.current_action == "walk" && self.pending_action.is_none() {
                            pet.set_action("idle");
                        }
                    }
                }
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let available_pets_copy = self.available_pets.clone();
                
                // Interact with the whole panel for context menu
                let bg_response = ui.interact(ui.max_rect(), ui.id().with("bg"), egui::Sense::click());
                bg_response.context_menu(|ui| {
                    ui.label(egui::RichText::new("설정").strong());
                    ui.checkbox(&mut self.mouse_follow, "마우스 따라오기");
                    ui.checkbox(&mut self.show_stats, "그래프 표시");
                    ui.separator();
                    
                    ui.label(egui::RichText::new("액션 선택").strong());
                    if let Some(pet) = &self.pet {
                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            let mut action_names: Vec<String> = pet.config.manifest.actions.keys().cloned().collect();
                            action_names.sort();
                            for action_name in action_names {
                                let display_name = get_action_label(&action_name);
                                if ui.selectable_label(pet.current_action == action_name, display_name).clicked() {
                                    self.pending_action = Some(action_name.clone());
                                    ui.close_menu();
                                }
                            }
                        });
                    }
                    
                    ui.separator();
                    ui.label(egui::RichText::new("캐릭터").strong());
                    for name in &available_pets_copy {
                        if ui.radio(self.current_pet_name == *name, name).clicked() {
                            self.pending_pet_switch = Some(name.clone());
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    if ui.button("종료").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    
                    // Left Column: Speech Bubble + Pet
                    ui.vertical(|ui| {
                        ui.set_min_width(160.0);
                        
                        // Speech Bubble (Improved Aesthetics)
                        if time < self.status_timeout {
                            ui.add_space(20.0);
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgba_premultiplied(255, 255, 255, 240))
                                .rounding(15.0)
                                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 255)))
                                .inner_margin(10.0)
                                .show(ui, |ui| {
                                    ui.set_max_width(120.0);
                                    ui.label(egui::RichText::new(&self.status_text).size(15.0).color(egui::Color32::BLACK).strong());
                                });
                        } else {
                            ui.add_space(60.0);
                        }

                        // Pet Rendering
                        if let Some(pet) = &self.pet {
                            if let Some(texture) = pet.current_texture() {
                                let size = egui::vec2(pet.config.manifest.cell_size as f32, pet.config.manifest.cell_size as f32);
                                let uv = if pet.facing_right {
                                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
                                } else {
                                    egui::Rect::from_min_max(egui::pos2(1.0, 0.0), egui::pos2(0.0, 1.0))
                                };

                                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag().union(egui::Sense::click()));
                                
                                // Draw image using painter to avoid hover visuals
                                ui.painter().image(texture.id(), rect, uv, egui::Color32::WHITE);
                                
                                if response.dragged() {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                                }
                                
                                // Double Click -> Bonk!
                                if response.double_clicked() {
                                    self.pending_action = Some("bonk".to_string());
                                }
                            } else {
                                ui.add_space(pet.config.manifest.cell_size as f32);
                            }
                        }
                    });

                    // Right Column: Resource Graphs
                    if self.show_stats {
                        ui.vertical(|ui| {
                            ui.add_space(60.0);
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 150))
                                .rounding(5.0)
                                .inner_margin(5.0)
                                .show(ui, |ui| {
                                    ui.set_max_width(70.0);
                                    ui.spacing_mut().item_spacing.y = 8.0;

                                    fn small_resource_bar(ui: &mut egui::Ui, label: &str, val: f32, color: egui::Color32) {
                                        ui.vertical(|ui| {
                                            ui.label(egui::RichText::new(label).size(9.0).color(egui::Color32::WHITE));
                                            let progress = (val / 100.0).clamp(0.0, 1.0);
                                            ui.add(egui::ProgressBar::new(progress)
                                                .fill(color)
                                                .desired_height(6.0));
                                        });
                                    }

                                    small_resource_bar(ui, "CPU", self.stats.cpu_usage, egui::Color32::from_rgb(100, 200, 255));
                                    small_resource_bar(ui, "RAM", self.stats.ram_usage_pct, egui::Color32::from_rgb(100, 255, 150));
                                    
                                    if let Some(gpu) = self.stats.gpu_usage {
                                        small_resource_bar(ui, "GPU", gpu, egui::Color32::from_rgb(200, 150, 255));
                                    }
                                    if let Some(vram) = self.stats.gpu_mem_pct {
                                        small_resource_bar(ui, "VRM", vram, egui::Color32::from_rgb(255, 200, 100));
                                    }
                                });
                        });
                    }
                });
            });

        if let Some(new_pet) = self.pending_pet_switch.take() {
            self.switch_pet(ctx, &new_pet);
            self.status_text = format!("{} 로 전환!", new_pet);
            self.status_timeout = time + 4.0;
        }

        if let Some(new_action) = self.pending_action.take() {
            if let Some(pet) = &mut self.pet {
                pet.set_action(&new_action);
                self.status_text = get_action_label(&new_action).to_string();
                self.status_timeout = time + 3.0;
            }
        }

        // Automatic Behaviors
        if time % 15.0 < 0.1 && self.pending_action.is_none() {
            if let Some(pet) = &mut self.pet {
                if pet.current_action == "idle" {
                    let actions = vec!["think", "wave", "cheer", "sit"];
                    use rand::seq::SliceRandom;
                    if let Some(act) = actions.choose(&mut rand::thread_rng()) {
                        pet.set_action(act);
                    }
                }
            }
        }

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_inner_size([300.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Pet",
        options,
        Box::new(|cc| Box::new(PetApp::new(cc)) as Box<dyn eframe::App>),
    )
}
