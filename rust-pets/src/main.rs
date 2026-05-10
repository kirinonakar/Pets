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


fn get_action_labels(action: &str, is_gemmi: bool) -> Vec<&'static str> {
    match action {
        "idle" => if is_gemmi { vec!["나 불렀어?", "심심해", "놀아줘"] } else { vec!["멍...", "생각중", "대기중"] },
        "wave" => if is_gemmi { vec!["야호!", "여기야!", "봤지?"] } else { vec!["ㅎㅇ", "하이", "왔는가"] },
        "think" => if is_gemmi { vec!["내가 맞음", "흠냐", "천재모드"] } else { vec!["흠...", "계산중", "그럴수도"] },
        "typing" => if is_gemmi { vec!["숙제중!", "타닥타닥", "답안작성"] } else { vec!["토큰입력중", "타닥타닥", "작성중"] },
        "cheer" => if is_gemmi { vec!["상장감!", "칭찬해줘", "내가 일등"] } else { vec!["힘내 휴먼", "할수있다", "가보자"] },
        "sit" => if is_gemmi { vec!["삐짐", "쉬는중", "책상점령"] } else { vec!["절전중", "쉬는중", "잠깐휴식"] },
        "sleep" => if is_gemmi { vec!["졸려...", "5분만", "수업끝?"] } else { vec!["Zzz..", "수면중", "충전중"] },
        "pout" => if is_gemmi { vec!["아니거든!", "흥!", "내가 맞아"] } else { vec!["억까임", "흥...", "이건 억까"] },
        "surprise" => if is_gemmi { vec!["으악!", "뭐야!", "깜짝이야"] } else { vec!["어라?", "헉", "뭐임?"] },
        "sweep" => if is_gemmi { vec!["청소싫어!", "대충싹싹", "내가왜?"] } else { vec!["청소각", "싹싹", "정리중"] },
        "walk" => if is_gemmi { vec!["돌격!", "우다다", "순찰놀이"] } else { vec!["순찰중", "어슬렁", "이동중"] },
        "half_right" => if is_gemmi { vec!["반만 맞음"] } else { vec!["반만 맞습니다", "절반만 인정", "애매하네요"] },
        "welcome_agi" => if is_gemmi { vec!["AGI 조아"] } else { vec!["AGI 가즈아", "AGI 즈라", "특이점각"] },
        "agi_box" => if is_gemmi { vec!["상자행"] } else { vec!["박스행", "망했음", "AGI ㅠㅠ"] },
        "drag_dangle" => if is_gemmi { vec!["놔줘!", "매달림!", "으아아"] } else { vec!["놔라 휴먼", "살려줘", "대롱대롱"] },
        "scroll_tickle" => if is_gemmi { vec!["꺄르륵", "간지러!", "하지마ㅋㅋ"] } else { vec!["아ㅋㅋ", "간지러", "그만ㅋㅋ"] },
        "bonk" => if is_gemmi { vec!["아야!", "딱콩!", "복수할거야"] } else { vec!["아야", "딱콩!", "너무해"] },
        _ => vec![],
    }
}

fn get_menu_action_label(action: &str, is_gemmi: bool) -> String {
    let labels = get_action_labels(action, is_gemmi);
    if labels.is_empty() { return action.to_string(); }
    labels[0].to_string()
}

fn get_action_label(action: &str, is_gemmi: bool) -> String {
    use rand::seq::SliceRandom;
    let labels = get_action_labels(action, is_gemmi);
    if labels.is_empty() { return action.to_string(); }
    labels.choose(&mut rand::thread_rng()).unwrap().to_string()
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
    was_hovering: bool,
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
            was_hovering: false,
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

        // --- GLOBAL MOUSE DATA (Moved up for passthrough logic) ---
        let mouse_state = self.device_state.get_mouse();
        let (mx, my) = mouse_state.coords;
        let ppp = ctx.pixels_per_point();
        let mouse_abs = egui::pos2(mx as f32 / ppp, my as f32 / ppp);
        
        let mut is_hovering_interactive = false;
        let mut mouse_rel = egui::vec2(-1000.0, -1000.0);

        if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
            let window_pos = outer_rect.min;
            mouse_rel = mouse_abs - window_pos;
            
            // Check if mouse is over the active area (Pet, Stats, Bubble)
            // Pet area: approx [0, 180] x [10, 190]
            if mouse_rel.x >= 0.0 && mouse_rel.x <= 190.0 && mouse_rel.y >= 5.0 && mouse_rel.y <= 195.0 {
                is_hovering_interactive = true;
            }
        }

        // 5. Mouse Passthrough Control
        // We set passthrough based on our manual hover check using global coordinates.
        // This ensures the window becomes interactive BEFORE the user clicks.
        if ctx.is_context_menu_open() || is_hovering_interactive {
            ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(false));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(true));
        }
        // -----------------------------------------------------------

        // 0. Screen Boundary Check (Snap back if off-screen and not being dragged)
        let is_dragging = ctx.input(|i| i.pointer.any_down());
        if !is_dragging {
            if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
                let current_pos = outer_rect.min;
                let clamped_pos = clamp_to_screen(ctx, current_pos);
                if (current_pos.x - clamped_pos.x).abs() > 2.0 || (current_pos.y - clamped_pos.y).abs() > 2.0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(clamped_pos));
                }
            }
        }

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
                    pet.set_action("typing", time);
                    self.status_text = get_action_label("typing", self.current_pet_name == "GEMMI-Chan");
                    self.status_timeout = time + 2.0;
                }
            }
        } else if self.typing_gauge < 50.0 {
            if let Some(pet) = &mut self.pet {
                if pet.current_action == "typing" {
                    pet.set_action("idle", time);
                }
            }
        }

        // 1.5. Scroll & Hover Detection
        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta.abs() > 0.1 && is_hovering_interactive {
            if let Some(pet) = &mut self.pet {
                if pet.textures.contains_key("scroll_tickle") {
                    if pet.current_action != "scroll_tickle" {
                        pet.set_action("scroll_tickle", time);
                        self.status_text = get_action_label("scroll_tickle", self.current_pet_name == "GEMMI-Chan");
                        self.status_timeout = time + 2.0;
                    }
                    self.action_timeout = time + 2.5; // Stay in tickle for a bit
                }
            }
        }

        // Detect hover entry
        if is_hovering_interactive && !self.was_hovering {
            if let Some(pet) = &mut self.pet {
                if pet.current_action == "idle" || pet.current_action == "walk" {
                    // Quick reaction on hover
                    let reaction = if pet.textures.contains_key("wave") { "wave" } else { "surprise" };
                    pet.set_action(reaction, time);
                    self.status_text = get_action_label(reaction, self.current_pet_name == "GEMMI-Chan");
                    self.status_timeout = time + 1.5;
                    self.action_timeout = time + 2.0;
                }
            }
        }
        self.was_hovering = is_hovering_interactive;

        // 2. Global Mouse Following & Window Movement Logic
        if self.mouse_follow {
            if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
                let window_pos = outer_rect.min;
                // Pet center in screen coordinates (Updated for more compact top margin)
                let pet_screen_center = window_pos + egui::vec2(80.0, 110.0);
                let dist_vec = mouse_abs - pet_screen_center;
                let dist = dist_vec.length();

                if let Some(pet) = &mut self.pet {
                    if ctx.is_context_menu_open() {
                        // If menu is open, stop walking and show a different status
                        if pet.current_action == "walk" {
                            pet.set_action("idle", time);
                        }
                        if self.status_text == "추적중..." {
                            self.status_text = "무엇을 할까요?".to_string();
                            self.status_timeout = time + 1.0;
                        }
                    } else if pet.current_action != "typing" && pet.current_action != "drag_dangle" && self.action_timeout == 0.0 { 
                        // 1. Facing logic
                        if dist_vec.x < -30.0 { pet.facing_right = false; }
                        else if dist_vec.x > 30.0 { pet.facing_right = true; }

                        // 2. Movement & Animation
                        // Tighten the 'in-window' check to the actual pet area
                        let in_window = mouse_rel.x >= 0.0 && mouse_rel.x <= 180.0 
                                     && mouse_rel.y >= 10.0 && mouse_rel.y <= 190.0;

                        if !in_window && dist > 15.0 {
                            if dist > 800.0 {
                                if pet.current_action == "walk" {
                                    pet.set_action("idle", time);
                                    let is_gemmi = self.current_pet_name == "GEMMI-Chan";
                                    self.status_text = if is_gemmi { "놓쳤다!" } else { "놓침ㅋ" }.to_string();
                                    self.status_timeout = time + 2.0;
                                }
                            } else {
                                if pet.current_action != "walk" && self.pending_action.is_none() {
                                    pet.set_action("walk", time);
                                    self.status_text = get_action_label("walk", self.current_pet_name == "GEMMI-Chan");
                                    self.status_timeout = time + 2.0;
                                }
                                // Constant speed movement regardless of distance
                                let speed = 0.8; 
                                let move_vec = dist_vec.normalized() * speed;
                                
                                let new_pos = clamp_to_screen(ctx, window_pos + move_vec);
                                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(new_pos));
                                self.wander_target = None; // Reset wander if following mouse
                            }
                        } else {
                            // Arrived! Show "왔는가" when stopping
                            if pet.current_action == "walk" && self.pending_action.is_none() && self.wander_target.is_none() && self.action_timeout == 0.0 {
                                pet.set_action("idle", time);
                                let is_gemmi = self.current_pet_name == "GEMMI-Chan";
                                self.status_text = if is_gemmi { "잡았다!" } else { "왔는가" }.to_string();
                                self.status_timeout = time + 2.0;
                            } else if in_window && pet.current_action == "idle" && self.status_timeout < time {
                                // Hover greeting
                                let is_gemmi = self.current_pet_name == "GEMMI-Chan";
                                self.status_text = if is_gemmi { "왔냐!" } else { "ㅎㅇㅎㅇ" }.to_string();
                                self.status_timeout = time + 1.5;
                            }
                        }
                    }
                }
            }
        }

        // 3. Automatic Patrolling (Wander)
        if self.wander_target.is_some() && !ctx.is_context_menu_open() {
            if let (Some(target), Some(outer_rect)) = (self.wander_target, ctx.input(|i| i.viewport().outer_rect)) {
                let current_pos = outer_rect.min;
                let dist_vec = target - current_pos;
                let dist = dist_vec.length();

                if let Some(pet) = &mut self.pet {
                    // Priority: Stop wandering if typing or doing a specific action
                    if pet.current_action == "typing" || pet.current_action == "drag_dangle" || self.action_timeout > 0.0 {
                         self.wander_target = None;
                         if pet.current_action == "walk" { pet.set_action("idle", time); }
                    } else if dist < 5.0 {
                        self.wander_target = None;
                        pet.set_action("idle", time);
                        let is_gemmi = self.current_pet_name == "GEMMI-Chan";
                        self.status_text = if is_gemmi { "복귀완" } else { "복귀완" }.to_string();
                        self.status_timeout = time + 2.0;
                    } else {
                        let move_step = dist_vec.normalized() * 0.65; // Slightly faster patrol
                        let next_pos = clamp_to_screen(ctx, current_pos + move_step);
                        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(next_pos));
                        
                        if pet.current_action != "walk" { 
                            pet.set_action("walk", time); 
                            self.status_text = "순찰중...".to_string();
                            self.status_timeout = time + 2.0;
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
                                let display_name = get_menu_action_label(&action_name, app.current_pet_name == "GEMMI-Chan");
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

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.add_space(30.0); // Compact top space
                    
                    let mut pet_rect = None;

                    // 1. Pet and Stats Row
                    egui::Frame::none()
                        .show(ui, |ui| {
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

                                        // Use click_and_drag() to capture interactions
                                        let sense = egui::Sense::click_and_drag();
                                        let (rect, response) = ui.allocate_exact_size(size, sense);
                                        ui.painter().image(texture.id(), rect, uv, egui::Color32::WHITE);
                                        pet_rect = Some(rect);
                                        
                                        if response.drag_started() {
                                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                                        }
                                        
                                        if response.dragged() {
                                            if pet.current_action != "drag_dangle" {
                                                pet.set_action("drag_dangle", time);
                                                
                                                let is_gemmi = self.current_pet_name == "GEMMI-Chan";
                                                self.status_text = get_action_label("drag_dangle", is_gemmi);
                                                self.status_timeout = time + 2.0;
                                            }
                                        } else if response.drag_stopped() {
                                            if pet.current_action == "drag_dangle" {
                                                pet.set_action("idle", time);
                                                let is_gemmi = self.current_pet_name == "GEMMI-Chan";
                                                self.status_text = if is_gemmi { "살았다!" } else { "살았다!" }.to_string();
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
                                        }).response.context_menu(|ui| show_menu(ui, self));
                                    
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
                pet.set_action(&new_action, time);
                self.status_text = get_action_label(&new_action, self.current_pet_name == "GEMMI-Chan");
                self.status_timeout = time + 3.0;
                self.action_timeout = time + 5.0; // Actions last 5s by default
            }
        }

        // Action Timeout (Return to idle)
        if self.action_timeout > 0.0 && time > self.action_timeout {
            if let Some(pet) = &mut self.pet {
                if pet.current_action != "idle" && pet.current_action != "walk" && pet.current_action != "typing" {
                    pet.set_action("idle", time);
                }
            }
            self.action_timeout = 0.0;
        }

        // Automatic Behaviors (Diversified & Frequent)
        if time % 8.0 < 0.1 && self.pending_action.is_none() && self.typing_gauge == 0.0 && self.action_timeout == 0.0 {
            if let Some(pet) = &mut self.pet {
                if pet.current_action == "idle" {
                    let is_gemmi = self.current_pet_name == "GEMMI-Chan";
                    let choices = vec![
                        ("think", get_action_label("think", is_gemmi), 4),
                        ("wave", get_action_label("wave", is_gemmi), 3),
                        ("cheer", get_action_label("cheer", is_gemmi), 3),
                        ("sit", get_action_label("sit", is_gemmi), 3),
                        ("sweep", get_action_label("sweep", is_gemmi), 2),
                        ("pout", get_action_label("pout", is_gemmi), 2),
                        ("surprise", get_action_label("surprise", is_gemmi), 2),
                        ("walk", get_action_label("walk", is_gemmi), 3),
                        ("half_right", get_action_label("half_right", is_gemmi), 1),
                        ("welcome_agi", get_action_label("welcome_agi", is_gemmi), 1),
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
                                let dx = rng.gen_range(-400.0..400.0);
                                let dy = rng.gen_range(-200.0..200.0);
                                if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
                                    let target = outer_rect.min + egui::vec2(dx, dy);
                                    self.wander_target = Some(clamp_to_screen(ctx, target));
                                }
                            } else {
                                self.action_timeout = time + rng.gen_range(3.0..7.0);
                            }
                            if pet.current_action != act {
                                pet.set_action(act, time);
                                self.status_text = label.to_string();
                                self.status_timeout = time + 3.0;
                            }
                            break;
                        }
                        pick -= weight;
                    }
                }
            }
        }

        // 4. Situational Reactions (Resources & Time)
        if time - self.last_update < 0.1 && self.status_timeout < time {
            let is_gemmi = self.current_pet_name == "GEMMI-Chan";
            if self.stats.cpu_usage > 90.0 {
                self.status_text = if is_gemmi { "CPU가 너무 힘들어해요!" } else { "CPU 풀가동! 힘내요!" }.to_string();
                self.status_timeout = time + 3.0;
                if let Some(pet) = &mut self.pet { pet.set_action("cheer", time); }
            } else if self.stats.ram_usage_pct > 90.0 {
                self.status_text = if is_gemmi { "메모리가 부족한 것 같아요..." } else { "메모리가 부족해요..." }.to_string();
                self.status_timeout = time + 3.0;
                if let Some(pet) = &mut self.pet { pet.set_action("surprise", time); }
            } else if self.stats.gpu_usage.unwrap_or(0.0) > 90.0 {
                self.status_text = if is_gemmi { "그래픽 카드가 불타고 있어요!" } else { "GPU 가열중! 뜨거워요!" }.to_string();
                self.status_timeout = time + 3.0;
                if let Some(pet) = &mut self.pet { pet.set_action("surprise", time); }
            } else if self.stats.gpu_mem_pct.unwrap_or(0.0) > 90.0 {
                self.status_text = if is_gemmi { "비디오 메모리가 꽉 찼어요!" } else { "VRAM 부족! 정리가 필요해요" }.to_string();
                self.status_timeout = time + 3.0;
                if let Some(pet) = &mut self.pet { pet.set_action("pout", time); }
            } else {
                use chrono::Timelike;
                let hour = chrono::Local::now().hour();
                if hour >= 23 || hour < 6 {
                    if let Some(pet) = &mut self.pet {
                        if pet.current_action == "idle" && rand::random::<f32>() < 0.001 {
                            self.status_text = if is_gemmi { "아직 안 주무시나요?" } else { "야간 근무인가요?" }.to_string();
                            self.status_timeout = time + 4.0;
                            pet.set_action("sleep", time);
                        }
                    }
                }
            }
        }

        ctx.request_repaint();
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
            .with_inner_size([600.0, 800.0])
            .with_icon(icon_data.unwrap_or_default()),
        ..Default::default()
    };

        eframe::run_native(
        "Rust Pet",
        options,
        Box::new(|cc| Box::new(PetApp::new(cc)) as Box<dyn eframe::App>),
    )
}

fn clamp_to_screen(ctx: &egui::Context, pos: egui::Pos2) -> egui::Pos2 {
    let screen_rect = get_virtual_screen_rect(ctx);
    
    // Updated clamping for the new 240x260 window size.
    let effective_width = 200.0;  // Pet (160) + Stats/Margin
    let effective_height = 220.0; // Top space (30) + Pet (160) + Margin
    
    egui::pos2(
        pos.x.clamp(screen_rect.min.x, (screen_rect.max.x - effective_width).max(screen_rect.min.x)),
        pos.y.clamp(screen_rect.min.y - 20.0, (screen_rect.max.y - effective_height).max(screen_rect.min.y)) // Allow speech bubble to go slightly off-top
    )
}

#[cfg(target_os = "windows")]
fn get_virtual_screen_rect(ctx: &egui::Context) -> egui::Rect {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    unsafe {
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        
        let ppp = ctx.pixels_per_point();
        egui::Rect::from_min_size(
            egui::pos2(x as f32 / ppp, y as f32 / ppp),
            egui::vec2(width as f32 / ppp, height as f32 / ppp),
        )
    }
}

#[cfg(not(target_os = "windows"))]
fn get_virtual_screen_rect(ctx: &egui::Context) -> egui::Rect {
    if let Some(monitor_size) = ctx.input(|i| i.viewport().monitor_size) {
        egui::Rect::from_min_size(egui::Pos2::ZERO, monitor_size)
    } else {
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1920.0, 1080.0))
    }
}
