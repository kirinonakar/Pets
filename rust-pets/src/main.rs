#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod monitor;
mod config;
mod pet;
mod llm;

use eframe::egui;
use monitor::{Monitor, SystemStats};
use device_query::{DeviceQuery, DeviceState};
use config::PetConfig;
use pet::PetState;
use llm::{LlmConfig, LlmProvider, GOOGLE_MODELS};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;


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

fn get_resource_label(resource: &str, is_gemmi: bool) -> String {
    use rand::seq::SliceRandom;
    let labels = match resource {
        "cpu_high" => if is_gemmi { 
            vec!["으악, CPU가 불타고 있어!", "컴퓨터가 너무 힘들어해!", "열기 대박... 계란 구워도 되겠어", "CPU 살려어어!", "팬 돌아가는 소리 들려?"] 
        } else { 
            vec!["CPU 풀가동! 힘내요!", "연산량이 엄청나네요", "CPU가 열일 중입니다!", "성능 한계 돌파!", "CPU 온도가 높아요!"] 
        },
        "ram_high" => if is_gemmi { 
            vec!["메모리가 꽉 찼어! 답답해!", "나 들어갈 자리가 없잖아!", "정리 좀 해줘!", "램이 꽉꽉 찼어!", "비우기 좀 눌러주면 안 돼?"] 
        } else { 
            vec!["메모리가 부족해요...", "정리가 필요해 보입니다", "RAM 사용량이 높아요!", "여유 공간이 거의 없네요", "메모리 최적화가 필요해요"] 
        },
        "gpu_high" => if is_gemmi { 
            vec!["그래픽 카드가 비명을 질러!", "GPU가 활활! 타오른다!", "연기 나는 거 아냐?", "화면 뚫고 나올 것 같아!", "GPU 살려어!"] 
        } else { 
            vec!["GPU 가열중! 뜨거워요!", "그래픽 연산이 많네요", "GPU 온도가 높습니다!", "VGA 성능 풀가동!", "열기가 느껴지네요"] 
        },
        "vram_high" => if is_gemmi { 
            vec!["비디오 메모리 부족! 화면 멈추겠어!", "VRAM 꽉 찼어, 좀 비워줘!", "헐, 메모리 용량 실화야?", "그래픽 메모리가 꽉 찼어!", "텍스처가 너무 무거워!"] 
        } else { 
            vec!["VRAM 부족! 정리가 필요해요", "비디오 메모리가 가득 찼습니다", "텍스처 로딩이 힘들어 보여요", "VRAM 용량이 간당간당해요", "그래픽 설정이 높나요?"] 
        },
        "night" => if is_gemmi { 
            vec!["졸린데... 아직 안 자?", "나만 두고 잘 거야?", "밤샘은 피부에 안 좋대!", "하암... 잠 안 와?", "언제 잘 거야? 기다릴게"] 
        } else { 
            vec!["야간 근무인가요?", "아직 깨어 계시네요", "늦은 밤입니다. 쉬엄쉬엄 하세요", "밤샘 작업 화이팅!", "충전이 필요한 시간이에요"] 
        },
        _ => vec!["...", "흠?", "어라?"],
    };
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
    last_auto_behavior_time: f64,
    
    // LLM Settings
    llm_config: LlmConfig,
    google_api_key: String,
    show_llm_settings: bool,
    available_lm_studio_models: Vec<String>,
    lm_studio_fetcher: Option<Receiver<Vec<String>>>,
    last_lm_studio_fetch: f64,
    
    // LLM Chat
    llm_chat_input: String,
    show_llm_chat: bool,
    is_llm_thinking: bool,
    llm_chat_receiver: Option<Receiver<Result<String, String>>>,
    llm_response_text: String,
    llm_response_timeout: f64,
    llm_chat_history: Vec<llm::ChatMessage>,
    last_llm_chat_history_time: f64,
    last_chat_input_activity: f64,
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
            last_auto_behavior_time: 0.0,
            
            llm_config: load_llm_config(),
            google_api_key: llm::get_google_api_key().unwrap_or_default(),
            show_llm_settings: false,
            available_lm_studio_models: Vec::new(),
            lm_studio_fetcher: None,
            last_lm_studio_fetch: 0.0,

            llm_chat_input: String::new(),
            show_llm_chat: false,
            is_llm_thinking: false,
            llm_chat_receiver: None,
            llm_response_text: String::new(),
            llm_response_timeout: 0.0,
            llm_chat_history: Vec::new(),
            last_llm_chat_history_time: 0.0,
            last_chat_input_activity: 0.0,
        }
    }
    
    fn switch_pet(&mut self, ctx: &egui::Context, name: &str) {
        if self.current_pet_name == name { return; }
        if let Some(config) = PetConfig::load_embedded(name) {
            self.pet = Some(PetState::new(config, ctx));
            self.current_pet_name = name.to_string();
            
            // Reset LLM state on character change
            self.llm_chat_history.clear();
            self.llm_response_text.clear();
            self.llm_response_timeout = 0.0;
            self.last_llm_chat_history_time = 0.0;
            self.last_chat_input_activity = 0.0;
            self.is_llm_thinking = false;
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
            // LLM Bubble area (Below pet): expanded for scrolling/selection/input
            let is_hovering_bubble = mouse_rel.x >= 0.0 && mouse_rel.x <= 250.0 && mouse_rel.y >= 195.0 && mouse_rel.y <= 500.0;
            if (self.llm_response_timeout > time || self.is_llm_thinking || self.show_llm_chat) && is_hovering_bubble {
                is_hovering_interactive = true;
                
                // Extend timeout if hovering
                if self.llm_response_timeout > time {
                    self.llm_response_timeout = time + 10.0;
                }
            }
        }

        // Also extend if typing (even if not hovering)
        if !self.llm_chat_input.is_empty() && self.llm_response_timeout > time {
            self.llm_response_timeout = time + 10.0;
        }

        // Auto-close chat input after 1 minute of inactivity
        if self.show_llm_chat && time - self.last_chat_input_activity > 60.0 {
            self.show_llm_chat = false;
        }

        // 5. Mouse Passthrough Control
        if ctx.is_context_menu_open() || is_hovering_interactive || self.show_llm_settings || self.show_llm_chat {
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
                // Only move if significantly off-screen to prevent jitter
                if (current_pos.x - clamped_pos.x).abs() > 1.0 || (current_pos.y - clamped_pos.y).abs() > 1.0 {
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
                    self.status_timeout = time + 10.0;
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
                        self.status_timeout = time + 10.0;
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
                    self.status_timeout = time + 10.0;
                    self.action_timeout = time + 2.0;
                }
            }
        }
        self.was_hovering = is_hovering_interactive;

        // 2. Global Mouse Following & Window Movement Logic
        // Mouse follow and auto patrol must not fight each other.
        // While patrol has a target, patrol owns the movement.
        if self.mouse_follow && self.wander_target.is_none() && !self.show_llm_settings && !self.show_llm_chat 
            && self.llm_response_timeout < time && !self.is_llm_thinking {
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
                            self.status_timeout = time + 10.0;
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
                                    self.status_timeout = time + 10.0;
                                }
                            } else {
                                if pet.current_action != "walk" && self.pending_action.is_none() {
                                    pet.set_action("walk", time);
                                    self.status_text = get_action_label("walk", self.current_pet_name == "GEMMI-Chan");
                                    self.status_timeout = time + 10.0;
                                }
                                // Constant speed movement regardless of distance
                                let speed = 0.8; 
                                let move_vec = dist_vec.normalized() * speed;
                                
                                let new_pos = clamp_to_screen(ctx, window_pos + move_vec);
                                if (new_pos.x - window_pos.x).abs() > 0.1 || (new_pos.y - window_pos.y).abs() > 0.1 {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(new_pos));
                                }
                                self.wander_target = None; // Reset wander if following mouse
                            }
                        } else {
                            // Arrived! Show "왔는가" when stopping
                            if pet.current_action == "walk" && self.pending_action.is_none() && self.wander_target.is_none() && self.action_timeout == 0.0 {
                                pet.set_action("idle", time);
                                let is_gemmi = self.current_pet_name == "GEMMI-Chan";
                                self.status_text = if is_gemmi { "잡았다!" } else { "왔는가" }.to_string();
                                self.status_timeout = time + 10.0;
                            } else if in_window && pet.current_action == "idle" && self.status_timeout < time {
                                // Hover greeting
                                let is_gemmi = self.current_pet_name == "GEMMI-Chan";
                                self.status_text = if is_gemmi { "왔냐!" } else { "ㅎㅇㅎㅇ" }.to_string();
                                self.status_timeout = time + 10.0;
                            }
                        }
                    }
                }
            }
        }

        // 3. Automatic Patrolling (Wander)
        if self.wander_target.is_some() && !ctx.is_context_menu_open() && !self.show_llm_settings && !self.show_llm_chat {
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
                        let move_step = dist_vec.normalized() * 0.75; // Slightly faster patrol

                        // Keep the walk action active for the whole patrol path, including edge bounces.
                        // Previously the edge-hit branch could skip this, so the pet moved without leg animation.
                        if pet.current_action != "walk" {
                            pet.set_action("walk", time);
                            self.status_text = "순찰중...".to_string();
                            self.status_timeout = time + 2.0;
                        }
                        pet.facing_right = move_step.x >= 0.0;

                        let raw_next_pos = current_pos + move_step;
                        let next_pos = clamp_to_screen(ctx, raw_next_pos);

                        // Bounce when the unclamped next position tries to leave the visible screen.
                        // Checking raw_next_pos vs next_pos is more reliable than checking whether the
                        // window actually moved, because egui applies viewport moves on the next frame.
                        let hit_x = (raw_next_pos.x - next_pos.x).abs() > 0.01;
                        let hit_y = (raw_next_pos.y - next_pos.y).abs() > 0.01;

                        if hit_x || hit_y {
                            let reflected_step = egui::vec2(
                                if hit_x { -move_step.x } else { move_step.x },
                                if hit_y { -move_step.y } else { move_step.y },
                            );
                            let bounce_pos = clamp_to_screen(ctx, current_pos + reflected_step);

                            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(bounce_pos));
                            self.wander_target = Some(make_bounced_wander_target(ctx, bounce_pos, reflected_step));
                            pet.facing_right = reflected_step.x >= 0.0;

                            let is_gemmi = self.current_pet_name == "GEMMI-Chan";
                            self.status_text = if is_gemmi { "앗! 반대로!" } else { "통! 반대로" }.to_string();
                            self.status_timeout = time + 10.0;
                        } else {
                            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(next_pos));
                        }
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
                    if ui.button("LLM 설정").clicked() {
                        app.show_llm_settings = true;
                        ui.close_menu();
                    }
                    ui.separator();

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
                    ui.add_space(60.0); // More space at top for speech bubbles
                    
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
                                                self.status_timeout = time + 10.0;
                                            }
                                        } else if response.drag_stopped() {
                                            if pet.current_action == "drag_dangle" {
                                                pet.set_action("idle", time);
                                                let is_gemmi = self.current_pet_name == "GEMMI-Chan";
                                                self.status_text = if is_gemmi { "살았다!" } else { "살았다!" }.to_string();
                                                self.status_timeout = time + 10.0;
                                            }
                                        }
                                        
                                        if response.double_clicked() {
                                            self.pending_action = Some("bonk".to_string());
                                        } else if response.clicked() && !response.dragged() {
                                            // Toggle LLM Chat
                                            self.show_llm_chat = !self.show_llm_chat;
                                            if self.show_llm_chat {
                                                self.status_text = "말해봐!".to_string();
                                                self.status_timeout = time + 10.0;
                                                self.last_chat_input_activity = time;
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
                        // 2a. Standard Speech Bubble (Above)
                        if time < self.status_timeout {
                            let bubble_width = (self.status_text.chars().count() as f32 * 7.0).clamp(80.0, 200.0);
                            // Adjust position based on width to keep it somewhat centered over the head
                            let x_offset = if bubble_width > 120.0 { 10.0 } else { 40.0 };
                            let bubble_pos = rect.left_top() + egui::vec2(x_offset, -20.0); 
                            let bubble_rect = egui::Rect::from_min_size(bubble_pos, egui::vec2(bubble_width, 200.0));
                            
                            ui.allocate_ui_at_rect(bubble_rect, |ui| {
                                let bubble_fill = egui::Color32::from_rgba_premultiplied(255, 255, 255, 240);
                                let bubble_stroke_color = egui::Color32::from_rgb(101, 205, 229);
                                let bubble_stroke = egui::Stroke::new(1.5, bubble_stroke_color);

                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    
                                    // 1. Draw the main bubble box
                                    let frame_res = egui::Frame::none()
                                        .fill(bubble_fill)
                                        .rounding(8.0)
                                        .stroke(bubble_stroke)
                                        .inner_margin(6.0)
                                        .show(ui, |ui| {
                                            ui.set_max_width(bubble_width);
                                            ui.label(egui::RichText::new(&self.status_text).size(12.0).color(egui::Color32::BLACK).strong());
                                        });

                                    let bubble_frame_rect = frame_res.response.rect;
                                    frame_res.response.context_menu(|ui| show_menu(ui, self));

                                    // 2. Draw the Speech bubble tail (Triangle)
                                    // We anchor it to the bubble frame's rect to ensure perfect alignment
                                    let tail_width = 12.0;
                                    let tail_height = 8.0;
                                    let tail_x_offset = 15.0; // Offset from the left of the bubble
                                    
                                    // Slight overlap (-1.0) ensures no physical gap
                                    let tail_rect = egui::Rect::from_min_size(
                                        bubble_frame_rect.left_bottom() + egui::vec2(tail_x_offset, -1.0),
                                        egui::vec2(tail_width, tail_height)
                                    );

                                    let p1 = tail_rect.left_top();
                                    let p2 = tail_rect.right_top();
                                    let p3 = tail_rect.center_bottom();

                                    // 2a. Fill the triangle
                                    ui.painter().add(egui::Shape::convex_polygon(
                                        vec![p1, p2, p3],
                                        bubble_fill,
                                        egui::Stroke::NONE,
                                    ));

                                    // 2b. Draw only the side strokes (left and right)
                                    ui.painter().line_segment([p1, p3], bubble_stroke);
                                    ui.painter().line_segment([p2, p3], bubble_stroke);

                                    // 3. Mask the bubble's bottom stroke at the junction
                                    // Draw a small line of the fill color over the junction to "erase" the border
                                    ui.painter().line_segment(
                                        [p1 + egui::vec2(0.5, 0.0), p2 - egui::vec2(0.5, 0.0)], 
                                        egui::Stroke::new(2.0, bubble_fill)
                                    );
                                });
                            });
                        }

                        // 2b. LLM Speech Bubble & Chat Input (Below)
                        if time < self.llm_response_timeout || self.is_llm_thinking || self.show_llm_chat {
                            let is_showing_response = time < self.llm_response_timeout || self.is_llm_thinking;
                            let text = if self.is_llm_thinking { "생각 중...".to_string() } else { self.llm_response_text.clone() };
                            
                            let bubble_width = 200.0;
                            let x_offset = 10.0;
                            let bubble_pos = rect.left_bottom() + egui::vec2(x_offset, 10.0); 
                            
                            egui::Area::new(egui::Id::new("llm_bubble"))
                                .fixed_pos(bubble_pos)
                                .order(egui::Order::Foreground)
                                .show(ctx, |ui| {
                                    let bubble_fill = egui::Color32::from_rgba_premultiplied(240, 250, 255, 240);
                                    let bubble_stroke_color = egui::Color32::from_rgb(100, 150, 255);
                                    let bubble_stroke = egui::Stroke::new(1.5, bubble_stroke_color);

                                    ui.vertical(|ui| {
                                        ui.spacing_mut().item_spacing.y = 0.0;
                                        
                                        // 1. Draw the Speech bubble tail (Triangle pointing UP)
                                        let tail_width = 12.0;
                                        let tail_height = 8.0;
                                        let tail_x_offset = 15.0;
                                        
                                        let tail_rect = egui::Rect::from_min_size(
                                            ui.cursor().min + egui::vec2(tail_x_offset, 0.0),
                                            egui::vec2(tail_width, tail_height)
                                        );

                                        let tp1 = tail_rect.left_bottom();
                                        let tp2 = tail_rect.right_bottom();
                                        let tp3 = tail_rect.center_top();

                                        ui.painter().add(egui::Shape::convex_polygon(
                                            vec![tp1, tp2, tp3],
                                            bubble_fill,
                                            egui::Stroke::NONE,
                                        ));
                                        ui.painter().line_segment([tp1, tp3], bubble_stroke);
                                        ui.painter().line_segment([tp2, tp3], bubble_stroke);

                                        // 2. Draw the main bubble box
                                        egui::Frame::none()
                                            .fill(bubble_fill)
                                            .rounding(8.0)
                                            .stroke(bubble_stroke)
                                            .inner_margin(6.0)
                                            .show(ui, |ui| {
                                                ui.set_max_width(bubble_width + 10.0);
                                                ui.vertical(|ui| {
                                                    // 2a. Response Text
                                                    if is_showing_response {
                                                        egui::ScrollArea::vertical()
                                                            .id_source("llm_response_scroll")
                                                            .max_height(140.0)
                                                            .auto_shrink([true; 2])
                                                            .show(ui, |ui| {
                                                                ui.set_width(bubble_width);
                                                                let mut job = egui::text::LayoutJob::default();
                                                                job.wrap.max_width = bubble_width;
                                                                let parts: Vec<&str> = text.split("**").collect();
                                                                for (i, part) in parts.iter().enumerate() {
                                                                    let mut format = egui::TextFormat {
                                                                        font_id: egui::FontId::proportional(12.0),
                                                                        color: egui::Color32::BLACK,
                                                                        ..Default::default()
                                                                    };
                                                                    if i % 2 == 1 {
                                                                        format.font_id = egui::TextStyle::Button.resolve(ui.style());
                                                                    }
                                                                    job.append(*part, 0.0, format);
                                                                }
                                                                ui.add(egui::Label::new(job).selectable(true));
                                                            });
                                                        
                                                        if self.show_llm_chat {
                                                            ui.add_space(5.0);
                                                            ui.separator();
                                                            ui.add_space(5.0);
                                                        }
                                                    }

                                                    // 2b. Chat Input
                                                    if self.show_llm_chat {
                                                        if self.is_llm_thinking {
                                                            ui.horizontal(|ui| {
                                                                ui.spinner();
                                                                ui.label(egui::RichText::new("생각 중...").size(10.0));
                                                            });
                                                        } else {
                                                            let resp = ui.add(egui::TextEdit::multiline(&mut self.llm_chat_input)
                                                                .id_source("llm_chat_input")
                                                                .lock_focus(true)
                                                                .desired_width(bubble_width)
                                                                .desired_rows(1)
                                                                .font(egui::FontId::proportional(11.0))
                                                                .hint_text("메시지를 입력하세요..."));
                                                            
                                                            if resp.changed() {
                                                                self.last_chat_input_activity = time;
                                                            }
                                                            
                                                            let mut submit = false;
                                                            if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl) {
                                                                submit = true;
                                                                // Remove the newline added by Ctrl+Enter if necessary
                                                                self.llm_chat_input = self.llm_chat_input.trim_end_matches('\n').to_string();
                                                            }
                                                            
                                                            ui.add_space(8.0);
                                                            ui.horizontal(|ui| {
                                                                let send_btn = ui.add_sized([50.0, 20.0], egui::Button::new("전송"));
                                                                if send_btn.clicked() || submit {
                                                                    self.last_chat_input_activity = time;
                                                                    if !self.llm_chat_input.is_empty() {
                                                                        // Session timeout check
                                                                        if time - self.last_llm_chat_history_time > 1800.0 {
                                                                            self.llm_chat_history.clear();
                                                                        }
                                                                        self.last_llm_chat_history_time = time;

                                                                        if self.llm_chat_history.is_empty() {
                                                                            let system_prompt = if self.current_pet_name == "GEMMI-Chan" {
                                                                                std::fs::read_to_string("gemmi.txt")
                                                                                    .unwrap_or_else(|_| include_str!("../gemmi.txt").to_string())
                                                                            } else if self.current_pet_name == "GP-Chan" {
                                                                                std::fs::read_to_string("GPchan.txt")
                                                                                    .unwrap_or_else(|_| include_str!("../GPchan.txt").to_string())
                                                                            } else {
                                                                                String::new()
                                                                            };
                                                                            if !system_prompt.is_empty() {
                                                                                self.llm_chat_history.push(llm::ChatMessage {
                                                                                    role: "system".to_string(),
                                                                                    content: system_prompt,
                                                                                });
                                                                            }
                                                                        }

                                                                        self.llm_chat_history.push(llm::ChatMessage {
                                                                            role: "user".to_string(),
                                                                            content: self.llm_chat_input.clone(),
                                                                        });

                                                                        let (tx, rx) = std::sync::mpsc::channel();
                                                                        let config = self.llm_config.clone();
                                                                        let api_key = if config.provider == llm::LlmProvider::Google {
                                                                            Some(self.google_api_key.clone())
                                                                        } else {
                                                                            None
                                                                        };
                                                                        let messages = self.llm_chat_history.clone();
                                                                        
                                                                        std::thread::spawn(move || {
                                                                            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
                                                                            let res = rt.block_on(async {
                                                                                llm::chat_completion(&config, api_key.as_deref(), messages).await
                                                                            });
                                                                            let _ = tx.send(res.map_err(|e| e.to_string()));
                                                                        });
                                                                        
                                                                        self.llm_chat_receiver = Some(rx);
                                                                        self.is_llm_thinking = true;
                                                                        self.llm_chat_input.clear();
                                                                        
                                                                        if let Some(pet) = &mut self.pet {
                                                                            pet.set_action("idle", time);
                                                                        }
                                                                        self.llm_response_text = "생각 중...".to_string();
                                                                        self.llm_response_timeout = time + 60.0;
                                                                    }
                                                                }
                                                                if ui.add_sized([50.0, 20.0], egui::Button::new("닫기")).clicked() {
                                                                    self.show_llm_chat = false;
                                                                }
                                                            });
                                                        }
                                                    }
                                                });
                                            });

                                        // 3. Mask the junction
                                        ui.painter().line_segment(
                                            [tp1 + egui::vec2(0.5, 0.0), tp2 - egui::vec2(0.5, 0.0)], 
                                            egui::Stroke::new(2.0, bubble_fill)
                                        );
                                    });
                                });
                        }
                    }
                });

                // --- LLM Settings Window ---
                if self.show_llm_settings {
                    let mut open = self.show_llm_settings;
                    egui::Window::new(egui::RichText::new("LLM 설정").size(12.0))
                        .open(&mut open)
                        .resizable(false)
                        .default_width(300.0)
                        .show(ctx, |ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("LLM 서비스 선택").strong());
                                ui.horizontal(|ui| {
                                    ui.radio_value(&mut self.llm_config.provider, LlmProvider::LmStudio, "LM Studio");
                                    ui.radio_value(&mut self.llm_config.provider, LlmProvider::Google, "Google Gemini");
                                });
                                ui.separator();

                                match self.llm_config.provider {
                                    LlmProvider::LmStudio => {
                                        ui.label("LM Studio Endpoint:");
                                        let resp = ui.text_edit_singleline(&mut self.llm_config.lm_studio_endpoint);
                                        if resp.changed() {
                                            self.available_lm_studio_models.clear();
                                            self.last_lm_studio_fetch = 0.0; // Trigger fetch
                                        }

                                        ui.horizontal(|ui| {
                                            ui.label("모델:");
                                            let selected_text = if self.llm_config.lm_studio_model.is_empty() {
                                                "선택 안됨".to_string()
                                            } else {
                                                let mut s = self.llm_config.lm_studio_model.clone();
                                                if s.len() > 25 {
                                                    s.truncate(22);
                                                    s.push_str("...");
                                                }
                                                s
                                            };
                                            ui.label(egui::RichText::new(selected_text).color(egui::Color32::LIGHT_BLUE));
                                            
                                            ui.menu_button("더보기", |ui| {
                                                if self.available_lm_studio_models.is_empty() {
                                                    ui.label("모델이 없습니다...");
                                                } else {
                                                    for model in &self.available_lm_studio_models {
                                                        if ui.selectable_label(self.llm_config.lm_studio_model == *model, model).clicked() {
                                                            self.llm_config.lm_studio_model = model.clone();
                                                            ui.close_menu();
                                                        }
                                                    }
                                                }
                                            });
                                        });
                                        
                                        if ui.button("새로고침").clicked() {
                                            self.last_lm_studio_fetch = 0.0;
                                        }
                                    }
                                    LlmProvider::Google => {
                                        ui.label("Google API Key:");
                                        if ui.add(egui::TextEdit::singleline(&mut self.google_api_key).password(true)).changed() {
                                            let _ = llm::set_google_api_key(&self.google_api_key);
                                        }
                                        
                                        ui.label("모델 선택:");
                                        egui::ComboBox::from_id_source("google_model")
                                            .selected_text(&self.llm_config.google_model)
                                            .show_ui(ui, |ui| {
                                                for model in GOOGLE_MODELS {
                                                    ui.selectable_value(&mut self.llm_config.google_model, model.to_string(), *model);
                                                }
                                            });
                                    }
                                }
                                
                                ui.separator();
                                if ui.button("저장").clicked() {
                                    save_llm_config(&self.llm_config);
                                    self.show_llm_settings = false;
                                }
                            });
                        });
                    self.show_llm_settings = open;
                }


            });

        // --- LM Studio Model Fetching Logic ---
        if self.llm_config.provider == LlmProvider::LmStudio && (time - self.last_lm_studio_fetch > 30.0 || self.last_lm_studio_fetch == 0.0) {
            if self.lm_studio_fetcher.is_none() {
                let (tx, rx) = std::sync::mpsc::channel();
                let endpoint = self.llm_config.lm_studio_endpoint.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
                    let models = rt.block_on(async {
                        llm::fetch_lm_studio_models(&endpoint).await.unwrap_or_default()
                    });
                    let _ = tx.send(models);
                });
                self.lm_studio_fetcher = Some(rx);
                self.last_lm_studio_fetch = time;
            }
        }

        if let Some(rx) = &self.lm_studio_fetcher {
            if let Ok(models) = rx.try_recv() {
                self.available_lm_studio_models = models;
                // Auto-select if current model is empty and models are available
                if self.llm_config.lm_studio_model.is_empty() && !self.available_lm_studio_models.is_empty() {
                    self.llm_config.lm_studio_model = self.available_lm_studio_models[0].clone();
                }
                self.lm_studio_fetcher = None;
            }
        }

        // --- LLM Chat Response Handling ---
        if let Some(rx) = &self.llm_chat_receiver {
            if let Ok(res) = rx.try_recv() {
                self.is_llm_thinking = false;
                match res {
                    Ok(text) => {
                        self.llm_response_text = text.clone();
                        self.llm_response_timeout = time + 10.0;
                        self.llm_chat_history.push(llm::ChatMessage {
                            role: "assistant".to_string(),
                            content: text,
                        });
                        if let Some(pet) = &mut self.pet {
                            pet.set_action("wave", time);
                        }
                    }
                    Err(err) => {
                        self.llm_response_text = format!("Error: {}", err);
                        self.llm_response_timeout = time + 10.0;
                        if let Some(pet) = &mut self.pet {
                            pet.set_action("pout", time);
                        }
                    }
                }
                self.llm_chat_receiver = None;
            }
        }

        if let Some(new_pet) = self.pending_pet_switch.take() {
            self.switch_pet(ctx, &new_pet);
            self.status_text = format!("{} 로 전환!", new_pet);
            self.status_timeout = time + 10.0;
        }

        if let Some(new_action) = self.pending_action.take() {
            if let Some(pet) = &mut self.pet {
                pet.set_action(&new_action, time);
                self.status_text = get_action_label(&new_action, self.current_pet_name == "GEMMI-Chan");
                self.status_timeout = time + 10.0;
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
        if time - self.last_auto_behavior_time > 8.0 && self.pending_action.is_none() && self.typing_gauge == 0.0 && self.action_timeout == 0.0 {
            self.last_auto_behavior_time = time;
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
                                self.status_timeout = time + 10.0;
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
                self.status_text = get_resource_label("cpu_high", is_gemmi);
                self.status_timeout = time + 10.0;
                if let Some(pet) = &mut self.pet { pet.set_action("cheer", time); }
            } else if self.stats.ram_usage_pct > 90.0 {
                self.status_text = get_resource_label("ram_high", is_gemmi);
                self.status_timeout = time + 10.0;
                if let Some(pet) = &mut self.pet { pet.set_action("surprise", time); }
            } else if self.stats.gpu_usage.unwrap_or(0.0) > 90.0 {
                self.status_text = get_resource_label("gpu_high", is_gemmi);
                self.status_timeout = time + 10.0;
                if let Some(pet) = &mut self.pet { pet.set_action("surprise", time); }
            } else if self.stats.gpu_mem_pct.unwrap_or(0.0) > 90.0 {
                self.status_text = get_resource_label("vram_high", is_gemmi);
                self.status_timeout = time + 10.0;
                if let Some(pet) = &mut self.pet { pet.set_action("pout", time); }
            } else {
                use chrono::Timelike;
                let hour = chrono::Local::now().hour();
                if hour >= 23 || hour < 6 {
                    if let Some(pet) = &mut self.pet {
                        if pet.current_action == "idle" && rand::random::<f32>() < 0.001 {
                            self.status_text = get_resource_label("night", is_gemmi);
                            self.status_timeout = time + 10.0;
                            pet.set_action("sleep", time);
                        }
                    }
                }
            }
        }

        ctx.request_repaint();
    }
}

fn make_bounced_wander_target(ctx: &egui::Context, pos: egui::Pos2, reflected_step: egui::Vec2) -> egui::Pos2 {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let dir = if reflected_step.length() > 0.0001 {
        reflected_step.normalized()
    } else {
        egui::vec2(1.0, 0.0)
    };
    let side = egui::vec2(-dir.y, dir.x);
    let forward = rng.gen_range(220.0..520.0);
    let sideways = rng.gen_range(-140.0..140.0);

    clamp_to_screen(ctx, pos + dir * forward + side * sideways)
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
            .with_inner_size([400.0, 600.0])
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
    
    // The window is 400x600, but the pet is in the top-left area (~160x160).
    // The pet starts at y=30.0 inside the window.
    let pet_margin_x = 175.0; // 160 (pet width) + 15 (safe buffer)
    let pet_margin_y = 210.0; // 30 (top offset) + 160 (pet height) + 20 (safe buffer)
    
    egui::pos2(
        pos.x.clamp(screen_rect.min.x, (screen_rect.max.x - pet_margin_x).max(screen_rect.min.x)),
        pos.y.clamp(screen_rect.min.y - 30.0, (screen_rect.max.y - pet_margin_y).max(screen_rect.min.y))
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

fn get_config_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap_or_default();
    path.set_file_name("app_config.json");
    path
}

fn load_llm_config() -> LlmConfig {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    LlmConfig::default()
}

fn save_llm_config(config: &LlmConfig) {
    let path = get_config_path();
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(path, content);
    }
}
