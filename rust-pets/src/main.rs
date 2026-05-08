#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod monitor;
mod config;
mod pet;

use eframe::egui;
use monitor::{Monitor, SystemStats};
use device_query::{DeviceQuery, DeviceState};
use config::PetConfig;
use pet::PetState;
use std::path::Path;


fn get_action_label(action: &str) -> &str {
    match action {
        "idle" => "멍...",
        "wave" => "ㅎㅇㅎㅇ",
        "think" => "계산중...",
        "typing" => "토큰 입력중",
        "cheer" => "힘내 휴먼!",
        "sit" => "잠깐 휴식",
        "sleep" => "충전중 (Zzz)",
        "pout" => "이건 억까야",
        "surprise" => "어라?",
        "sweep" => "청소하는 중",
        "walk" => "순찰중",
        "half_right" => "반만 인정",
        "welcome_agi" => "AGI 가즈아!",
        "agi_box" => "박스행... ㅠㅠ",
        "drag_dangle" => "놔라 휴먼!",
        "scroll_tickle" => "아ㅋㅋ 간지러",
        "bonk" => "아야! 딱콩!",
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
    action_timeout: f64,
    device_state: DeviceState,
    typing_gauge: f32,
    last_keys: Vec<device_query::Keycode>,
    wander_target: Option<egui::Pos2>,
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
            action_timeout: 0.0,
            device_state: DeviceState::new(),
            typing_gauge: 0.0,
            last_keys: Vec::new(),
            wander_target: None,
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

// Moved update_animation to the end of the update function

        // Window properties
        ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));

        // 1. Typing Detection
        let current_keys = self.device_state.get_keys();
        let new_presses = current_keys.iter().filter(|k| !self.last_keys.contains(k)).count();
        self.last_keys = current_keys;

        if new_presses > 0 {
            self.typing_gauge = (self.typing_gauge + new_presses as f32 * 50.0).min(300.0);
        } else {
            self.typing_gauge = (self.typing_gauge - 2.0).max(0.0);
        }

        if self.typing_gauge > 100.0 {
            if let Some(pet) = &mut self.pet {
                if pet.current_action != "typing" && pet.current_action != "drag_dangle" {
                    pet.set_action("typing");
                    self.status_text = "타닥타닥...".to_string();
                    self.status_timeout = time + 2.0;
                }
            }
        } else if self.typing_gauge == 0.0 {
            if let Some(pet) = &mut self.pet {
                if pet.current_action == "typing" {
                    pet.set_action("idle");
                }
            }
        }

        // 2. Global Mouse Following & Window Movement Logic
        if self.mouse_follow {
            let mouse_state = self.device_state.get_mouse();
            let (mx, my) = mouse_state.coords;
            
            // Adjust for High DPI scaling (Physical Pixels -> Logical Points)
            let ppp = ctx.pixels_per_point();
            let mouse_abs = egui::pos2(mx as f32 / ppp, my as f32 / ppp);

            if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
                let window_pos = outer_rect.min;
                // Pet center in screen coordinates (Approximate based on top-aligned layout)
                let pet_screen_center = window_pos + egui::vec2(80.0, 140.0);
                let dist_vec = mouse_abs - pet_screen_center;
                let dist = dist_vec.length();

                if let Some(pet) = &mut self.pet {
                    if pet.current_action != "typing" { // Don't follow if typing
                        // 1. Facing logic
                        if dist_vec.x < -30.0 { pet.facing_right = false; }
                        else if dist_vec.x > 30.0 { pet.facing_right = true; }

                        // 2. Movement & Animation
                        let mouse_rel = mouse_abs - window_pos;
                        let in_window = mouse_rel.x >= 0.0 && mouse_rel.x <= 300.0 
                                     && mouse_rel.y >= 0.0 && mouse_rel.y <= 500.0;

                        if !in_window && dist > 50.0 {
                            if dist > 800.0 {
                                if pet.current_action == "walk" {
                                    pet.set_action("idle");
                                    self.status_text = "너무 빨라요! 놓침...".to_string();
                                    self.status_timeout = time + 2.0;
                                }
                            } else {
                                if pet.current_action != "walk" && self.pending_action.is_none() {
                                    pet.set_action("walk");
                                    self.status_text = "추적중...".to_string();
                                    self.status_timeout = time + 2.0;
                                }
                                // Much slower and capped movement for natural walking
                                let lerp_factor = 0.002; 
                                let move_vec = dist_vec * lerp_factor;
                                // Cap the speed to keep it natural
                                let capped_move = if move_vec.length() > 1.5 {
                                    move_vec.normalized() * 1.5
                                } else {
                                    move_vec
                                };
                                let new_pos = window_pos + capped_move;
                                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(new_pos));
                                self.wander_target = None; // Reset wander if following mouse
                            }
                        } else {
                            // Only reset to idle if not wandering and not doing a manual action
                            if pet.current_action == "walk" && self.pending_action.is_none() && self.wander_target.is_none() && self.action_timeout == 0.0 {
                                pet.set_action("idle");
                            }
                        }
                    }
                }
            }
        }

        // 3. Automatic Patrolling (Wander)
        if self.wander_target.is_some() {
            if let (Some(target), Some(outer_rect)) = (self.wander_target, ctx.input(|i| i.viewport().outer_rect)) {
                let current_pos = outer_rect.min;
                let dist_vec = target - current_pos;
                let dist = dist_vec.length();

                if dist < 5.0 {
                    self.wander_target = None;
                    if let Some(pet) = &mut self.pet {
                        pet.set_action("idle");
                        self.status_text = "복귀완!".to_string();
                        self.status_timeout = ctx.input(|i| i.time) + 2.0;
                    }
                } else {
                    let move_step = dist_vec.normalized() * 0.45; // Slower, more natural patrol
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(current_pos + move_step));
                    if let Some(pet) = &mut self.pet {
                        if pet.current_action != "walk" { 
                            pet.set_action("walk"); 
                            self.status_text = "순찰중...".to_string();
                            self.status_timeout = ctx.input(|i| i.time) + 2.0;
                        }
                        pet.facing_right = move_step.x > 0.0;
                    }
                }
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let available_pets_copy = self.available_pets.clone();
                
                // Define the common context menu
                let show_menu = |ui: &mut egui::Ui, app: &mut PetApp| {
                    ui.label(egui::RichText::new("설정").strong());
                    ui.checkbox(&mut app.mouse_follow, "마우스 따라오기");
                    ui.checkbox(&mut app.show_stats, "그래프 표시");
                    ui.separator();
                    
                    ui.label(egui::RichText::new("액션 선택").strong());
                    if let Some(pet) = &app.pet {
                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            let mut action_names: Vec<String> = pet.config.manifest.actions.keys().cloned().collect();
                            action_names.sort();
                            for action_name in action_names {
                                let display_name = get_action_label(&action_name);
                                if ui.selectable_label(pet.current_action == action_name, display_name).clicked() {
                                    app.pending_action = Some(action_name.clone());
                                    ui.close_menu();
                                }
                            }
                        });
                    }
                    
                    ui.separator();
                    ui.label(egui::RichText::new("캐릭터").strong());
                    for name in &available_pets_copy {
                        if ui.radio(app.current_pet_name == *name, name).clicked() {
                            app.pending_pet_switch = Some(name.clone());
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    if ui.button("종료").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                };

                // Background interaction
                let bg_response = ui.interact(ui.max_rect(), ui.id().with("bg"), egui::Sense::click());
                bg_response.context_menu(|ui| show_menu(ui, self));

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.add_space(60.0); // Reduced space for a more compact feel
                    
                    let mut pet_rect = None;

                    // 1. Pet and Stats Row
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = -15.0; // Pull left
                        
                        let mut pet_response = None;
                        if let Some(pet) = &mut self.pet {
                            if let Some(texture) = pet.current_texture() {
                                let size = egui::vec2(pet.config.manifest.cell_size as f32, pet.config.manifest.cell_size as f32);
                                let uv = if pet.facing_right {
                                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
                                } else {
                                    egui::Rect::from_min_max(egui::pos2(1.0, 0.0), egui::pos2(0.0, 1.0))
                                };

                                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag().union(egui::Sense::click()));
                                ui.painter().image(texture.id(), rect, uv, egui::Color32::WHITE);
                                pet_rect = Some(rect);
                                
                                if response.dragged() {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                                    if pet.current_action != "drag_dangle" {
                                        pet.set_action("drag_dangle");
                                        
                                        use rand::seq::SliceRandom;
                                        let drag_labels = ["놔라 휴먼!", "살려줘요!", "대롱대롱~", "어디가요!", "히익!", "우와아악!"];
                                        let mut rng = rand::thread_rng();
                                        if let Some(label) = drag_labels.choose(&mut rng) {
                                            self.status_text = label.to_string();
                                        }
                                        self.status_timeout = time + 2.0;
                                    }
                                } else if response.drag_stopped() {
                                    if pet.current_action == "drag_dangle" {
                                        pet.set_action("idle");
                                        self.status_text = "살았다!".to_string();
                                        self.status_timeout = time + 2.0;
                                    }
                                }
                                
                                if response.double_clicked() {
                                    self.pending_action = Some("bonk".to_string());
                                } else if response.clicked() && !response.dragged() {
                                    if let Some(pet) = &self.pet {
                                        let mut available = vec!["wave", "cheer", "surprise"];
                                        for extra in ["half_right", "welcome_agi", "agi_box", "think", "pout", "sweep"] {
                                            if pet.textures.contains_key(extra) {
                                                available.push(extra);
                                            }
                                        }
                                        use rand::seq::SliceRandom;
                                        let mut rng = rand::thread_rng();
                                        if let Some(action) = available.choose(&mut rng) {
                                            self.pending_action = Some(action.to_string());
                                        }
                                    }
                                }
                                pet_response = Some(response);
                            } else {
                                ui.add_space(pet.config.manifest.cell_size as f32);
                            }
                        }

                        // Stats Column attached to Pet
                        if self.show_stats {
                            ui.vertical(|ui| {
                                ui.add_space(50.0); // Move slightly up
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 150))
                                    .rounding(5.0)
                                    .inner_margin(4.0)
                                    .show(ui, |ui| {
                                        ui.set_max_width(25.0); // Reduce length (width)
                                        ui.spacing_mut().item_spacing.y = 5.0;

                                        fn mini_resource_bar(ui: &mut egui::Ui, label: &str, val: f32, color: egui::Color32) {
                                            ui.vertical(|ui| {
                                                ui.label(egui::RichText::new(label).size(8.0).color(egui::Color32::WHITE));
                                                let progress = (val / 100.0).clamp(0.0, 1.0);
                                                ui.add(egui::ProgressBar::new(progress)
                                                    .fill(color)
                                                    .desired_height(4.0)
                                                    .desired_width(25.0));
                                            });
                                        }

                                        mini_resource_bar(ui, "CPU", self.stats.cpu_usage, egui::Color32::from_rgb(100, 200, 255));
                                        mini_resource_bar(ui, "RAM", self.stats.ram_usage_pct, egui::Color32::from_rgb(100, 255, 150));
                                        
                                        if let Some(gpu) = self.stats.gpu_usage {
                                            mini_resource_bar(ui, "GPU", gpu, egui::Color32::from_rgb(200, 150, 255));
                                        }
                                        if let Some(vram) = self.stats.gpu_mem_pct {
                                            mini_resource_bar(ui, "VRM", vram, egui::Color32::from_rgb(255, 200, 100));
                                        }
                                    });
                            });
                        }

                        // Context menu for pet
                        if let Some(response) = pet_response {
                            response.context_menu(|ui| show_menu(ui, self));
                        }
                    });

                    // 2. Speech Bubble Overlay (Drawn last to be on top)
                    if let Some(rect) = pet_rect {
                        if time < self.status_timeout {
                            let bubble_width = 80.0;
                            // Closer to head: 40.0 right (center-ish), -25.0 up (just above head)
                            let bubble_pos = rect.left_top() + egui::vec2(40.0, -25.0); 
                            let bubble_rect = egui::Rect::from_min_size(bubble_pos, egui::vec2(bubble_width, 100.0));
                            
                            ui.allocate_ui_at_rect(bubble_rect, |ui| {
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    egui::Frame::none()
                                        .fill(egui::Color32::from_rgba_premultiplied(255, 255, 255, 240))
                                        .rounding(8.0)
                                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(101, 205, 229)))
                                        .inner_margin(6.0)
                                        .show(ui, |ui| {
                                            ui.set_max_width(bubble_width);
                                            ui.label(egui::RichText::new(&self.status_text).size(12.0).color(egui::Color32::BLACK).strong());
                                        });
                                    
                                    // Speech bubble tail (Triangle)
                                    ui.horizontal(|ui| {
                                        ui.add_space(12.0); // Slightly adjusted space
                                        let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 8.0), egui::Sense::hover());
                                        let points = vec![
                                            rect.left_top() + egui::vec2(1.0, -3.0),
                                            rect.right_top() + egui::vec2(-1.0, -3.0),
                                            rect.center_bottom(),
                                        ];
                                        ui.painter().add(egui::Shape::convex_polygon(
                                            points,
                                            egui::Color32::from_rgba_premultiplied(255, 255, 255, 240),
                                            egui::Stroke::new(1.5, egui::Color32::from_rgb(101, 205, 229)),
                                        ));
                                    });
                                });
                            });
                        }
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
                self.action_timeout = time + 5.0; // Actions last 5s by default
            }
        }

        // Action Timeout (Return to idle)
        if self.action_timeout > 0.0 && time > self.action_timeout {
            if let Some(pet) = &mut self.pet {
                if pet.current_action != "idle" && pet.current_action != "walk" && pet.current_action != "typing" {
                    pet.set_action("idle");
                }
            }
            self.action_timeout = 0.0;
        }

        // Automatic Behaviors (Diversified & Frequent)
        if time % 8.0 < 0.1 && self.pending_action.is_none() && self.typing_gauge == 0.0 && self.action_timeout == 0.0 {
            if let Some(pet) = &mut self.pet {
                if pet.current_action == "idle" {
                    let choices = vec![
                        ("think", "흠... 계산중", 4),
                        ("wave", "안녕 휴먼!", 3),
                        ("cheer", "화이팅!!", 3),
                        ("sit", "잠시 쉬는 중", 3),
                        ("sweep", "주변 정리 중", 2),
                        ("pout", "칫...", 2),
                        ("surprise", "앗!", 2),
                        ("walk", "순찰 개시!", 3),
                        ("half_right", "그럴 수도 있죠", 1),
                        ("welcome_agi", "AGI 가즈아!!", 1),
                    ];
                    
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let filtered_choices: Vec<_> = choices.into_iter()
                        .filter(|(act, _, _)| pet.textures.contains_key(*act))
                        .collect();
                    
                    let total_weight: i32 = filtered_choices.iter().map(|c| c.2).sum();
                    if total_weight == 0 { return; }
                    
                    let mut pick = rng.gen_range(0..total_weight);
                    
                    for (act, label, weight) in filtered_choices {
                        if pick < weight {
                            if act == "walk" {
                                let dx = rng.gen_range(-300.0..300.0);
                                let dy = rng.gen_range(-150.0..150.0);
                                if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
                                    self.wander_target = Some(outer_rect.min + egui::vec2(dx, dy));
                                }
                            } else {
                                self.action_timeout = time + rng.gen_range(3.0..7.0);
                            }
                            pet.set_action(act);
                            self.status_text = label.to_string();
                            self.status_timeout = time + 3.0;
                            break;
                        }
                        pick -= weight;
                    }
                }
            }
        }

        // 4. Situational Reactions (Resources & Time)
        if time - self.last_update < 0.1 && self.status_timeout < time {
            if self.stats.cpu_usage > 90.0 {
                self.status_text = "CPU 풀가동! 힘내요!".to_string();
                self.status_timeout = time + 3.0;
                if let Some(pet) = &mut self.pet { pet.set_action("cheer"); }
            } else if self.stats.ram_usage_pct > 90.0 {
                self.status_text = "메모리가 부족해요...".to_string();
                self.status_timeout = time + 3.0;
                if let Some(pet) = &mut self.pet { pet.set_action("surprise"); }
            } else {
                use chrono::Timelike;
                let hour = chrono::Local::now().hour();
                if hour >= 23 || hour < 6 {
                    if let Some(pet) = &mut self.pet {
                        if pet.current_action == "idle" && rand::random::<f32>() < 0.001 {
                            self.status_text = "야간 근무인가요?".to_string();
                            self.status_timeout = time + 4.0;
                            pet.set_action("sleep");
                        }
                    }
                }
            }
        }

        ctx.request_repaint();

        // 5. Final Animation Update
        if let Some(pet) = &mut self.pet {
            pet.update_animation(time);
        }
    }
}

fn main() -> eframe::Result<()> {
    // Load Icon
    let icon_data = if let Ok(image) = image::load_from_memory(include_bytes!("../app.png")) {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_inner_size([300.0, 500.0])
            .with_icon(icon_data.unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Pet",
        options,
        Box::new(|cc| Box::new(PetApp::new(cc)) as Box<dyn eframe::App>),
    )
}
