use gloo_storage::{LocalStorage, Storage};
use gloo_timers::callback::Timeout;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Event, KeyboardEvent, MouseEvent};
use yew::prelude::*;

// ═══════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════
struct Category {
    id: &'static str,
    label: &'static str,
    group: &'static str,
}

const CATEGORIES: &[Category] = &[
    Category { id: "physician", label: "醫師", group: "醫師類" },
    Category { id: "tcm_physician", label: "中醫師", group: "醫師類" },
    Category { id: "dentist", label: "牙醫師", group: "醫師類" },
    Category { id: "nurse", label: "護理師(護士)", group: "護理與助產類" },
    Category { id: "midwife", label: "助產師(士)", group: "護理與助產類" },
    Category { id: "pharmacist", label: "藥師", group: "藥事類" },
    Category { id: "assistant_pharmacist", label: "藥劑生", group: "藥事類" },
    Category { id: "medical_technologist", label: "醫事檢驗師(生)", group: "醫事技術類" },
    Category { id: "medical_radiation", label: "醫事放射師(士)", group: "醫事技術類" },
    Category { id: "dental_technologist", label: "牙體技術師(生)", group: "醫事技術類" },
    Category { id: "optometrist", label: "驗光師(生)", group: "醫事技術類" },
    Category { id: "physical_therapist", label: "物理治療師(生)", group: "復健與治療類" },
    Category { id: "occupational_therapist", label: "職能治療師(生)", group: "復健與治療類" },
    Category { id: "speech_therapist", label: "語言治療師", group: "復健與治療類" },
    Category { id: "audiologist", label: "聽力師", group: "復健與治療類" },
    Category { id: "respiratory_therapist", label: "呼吸治療師", group: "復健與治療類" },
    Category { id: "clinical_psychologist", label: "臨床心理師", group: "心理類" },
    Category { id: "counseling_psychologist", label: "諮商心理師", group: "心理類" },
    Category { id: "nutritionist", label: "營養師", group: "藥事類" },
    Category { id: "other", label: "其他", group: "其他專業類" },
];

struct Item {
    id: &'static str,
    label: &'static str,
}

const ITEMS: &[Item] = &[
    Item { id: "register", label: "執業(現歇業)登記" },
    Item { id: "suspend", label: "停業登記" },
    Item { id: "resume", label: "復業(現停業)登記" },
    Item { id: "cessation", label: "歇業(離職)登記" },
    Item { id: "dept_change", label: "(科別)變更" },
    Item { id: "name_change", label: "(姓名)變更" },
    Item { id: "inst_change", label: "機構變更" },
    Item { id: "cat_change", label: "類別變更" },
    Item { id: "lost_reissue", label: "遺失補發" },
    Item { id: "damage_reissue", label: "損壞補發" },
    Item { id: "renew", label: "到期換發" },
];

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
struct HistoryEntry {
    text: String,
    time: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct ToastState {
    message: String,
    is_error: bool,
}

// ═══════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════
fn clean_parentheses(text: &str) -> String {
    if text.contains("(科別)") || text.contains("（科別）") {
        return "科別變更".to_string();
    }
    if text.contains("(姓名)") || text.contains("（姓名）") {
        return "姓名變更".to_string();
    }
    let mut result = String::new();
    let mut in_parentheses = 0;
    for c in text.chars() {
        if c == '(' || c == '（' {
            in_parentheses += 1;
        } else if (c == ')' || c == '）') && in_parentheses > 0 {
            in_parentheses -= 1;
        } else if in_parentheses == 0 {
            result.push(c);
        }
    }
    result
}

fn get_generated_text(
    name: &str,
    category_id: &Option<String>,
    selected_items: &[String],
    placeholder_mode: bool,
) -> (String, bool) {
    let name_trimmed = name.trim();
    let display_name = if name_trimmed.is_empty() {
        if placeholder_mode { "（請輸入姓名）" } else { "" }
    } else {
        name_trimmed
    };

    let cleaned_category = if let Some(cat_id) = category_id {
        if let Some(cat) = CATEGORIES.iter().find(|c| c.id == cat_id) {
            clean_parentheses(cat.label)
        } else {
            if placeholder_mode { "（請選擇類別）".to_string() } else { "".to_string() }
        }
    } else {
        if placeholder_mode { "（請選擇類別）".to_string() } else { "".to_string() }
    };

    let cleaned_items_list: Vec<String> = selected_items
        .iter()
        .map(|id| {
            if let Some(item) = ITEMS.iter().find(|i| i.id == id) {
                clean_parentheses(item.label)
            } else {
                "".to_string()
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    let items_text = if cleaned_items_list.is_empty() {
        if placeholder_mode { "（請選擇項目）".to_string() } else { "".to_string() }
    } else {
        cleaned_items_list.join("、")
    };

    let is_complete = !name_trimmed.is_empty() && category_id.is_some() && !selected_items.is_empty();

    let text = format!("{}申辦{}{}", display_name, cleaned_category, items_text);
    (text, is_complete)
}

// Helper to copy text asynchronously using JS Clipboard API
async fn copy_to_clipboard_async(text: String) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window available"))?;
    let navigator = window.navigator();
    let clipboard = navigator.clipboard();
    let promise = clipboard.write_text(&text);
    JsFuture::from(promise).await?;
    Ok(())
}

// ═══════════════════════════════════════════════════
// COMPONENT
// ═══════════════════════════════════════════════════
enum Msg {
    UpdateName(String),
    ClearName,
    SelectCategory(String),
    ToggleItem(String),
    SelectTab(String),
    CopyText(MouseEvent),
    CopySuccess(String),
    CopyError,
    CopyFromHistory(String),
    ToggleHistory,
    ClearHistory,
    ResetAll,
    HideSuggestions,
    ShowSuggestions,
    SelectSuggestion(String),
    HideToast,
    ResetMorph,
    TriggerInstall,
    InstallPromptAvailable(JsValue),
    AppInstalled,
}

struct App {
    applicant_name: String,
    selected_category: Option<String>,
    selected_items: Vec<String>,
    selected_group_tab: String,
    copy_history: Vec<HistoryEntry>,
    recent_names: Vec<String>,
    history_open: bool,
    toast: Option<ToastState>,
    name_suggestions_open: bool,
    copied_morph: bool,
    toast_timeout: Option<Timeout>,
    morph_timeout: Option<Timeout>,
    suggestions_timeout: Option<Timeout>,
    deferred_prompt: Option<JsValue>,
    _keydown_listener: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    _before_install_listener: Option<Closure<dyn FnMut(Event)>>,
    _app_installed_listener: Option<Closure<dyn FnMut(Event)>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Load configurations from LocalStorage
        let copy_history: Vec<HistoryEntry> = LocalStorage::get::<Vec<HistoryEntry>>("medgen_history")
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .filter(|e| e.text.len() <= 500)
            .take(20)
            .collect();

        let recent_names: Vec<String> = LocalStorage::get::<Vec<String>>("medgen_names")
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .filter(|n| n.len() <= 50)
            .take(8)
            .collect();

        // Keydown shortcut setup (Ctrl + Enter)
        let link = ctx.link().clone();
        let keydown_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if (event.ctrl_key() || event.meta_key()) && event.key() == "Enter" {
                event.prevent_default();
                // Pass dummy MouseEvent (or we can just copy)
                link.send_message(Msg::CopyText(MouseEvent::new("click").unwrap()));
            }
        }) as Box<dyn FnMut(KeyboardEvent)>);

        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "keydown",
                keydown_closure.as_ref().unchecked_ref(),
            );
        }

        // PWA listener setup
        let link = ctx.link().clone();
        let before_install_closure = Closure::wrap(Box::new(move |e: Event| {
            e.prevent_default();
            link.send_message(Msg::InstallPromptAvailable(e.into()));
        }) as Box<dyn FnMut(Event)>);

        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "beforeinstallprompt",
                before_install_closure.as_ref().unchecked_ref(),
            );
        }

        let link = ctx.link().clone();
        let app_installed_closure = Closure::wrap(Box::new(move |_e: Event| {
            link.send_message(Msg::AppInstalled);
        }) as Box<dyn FnMut(Event)>);

        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "appinstalled",
                app_installed_closure.as_ref().unchecked_ref(),
            );
        }

        Self {
            applicant_name: String::new(),
            selected_category: None,
            selected_items: Vec::new(),
            selected_group_tab: "全部".to_string(),
            copy_history,
            recent_names,
            history_open: false,
            toast: None,
            name_suggestions_open: false,
            copied_morph: false,
            toast_timeout: None,
            morph_timeout: None,
            suggestions_timeout: None,
            deferred_prompt: None,
            _keydown_listener: Some(keydown_closure),
            _before_install_listener: Some(before_install_closure),
            _app_installed_listener: Some(app_installed_closure),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateName(name) => {
                self.applicant_name = name;
                true
            }
            Msg::ClearName => {
                self.applicant_name.clear();
                // Refocus input can be done in render or JS, we just clear here
                true
            }
            Msg::SelectCategory(cat_id) => {
                self.selected_category = Some(cat_id);
                self.selected_items.clear();
                true
            }
            Msg::ToggleItem(item_id) => {
                if let Some(pos) = self.selected_items.iter().position(|x| x == &item_id) {
                    self.selected_items.remove(pos);
                } else {
                    self.selected_items.push(item_id);
                }
                true
            }
            Msg::SelectTab(tab) => {
                self.selected_group_tab = tab;
                true
            }
            Msg::CopyText(e) => {
                // Perform checks
                let name_trimmed = self.applicant_name.trim();
                if name_trimmed.is_empty() {
                    ctx.link().send_message(Msg::CopyError);
                    self.toast = Some(ToastState {
                        message: "請先輸入申請人姓名".to_string(),
                        is_error: true,
                    });
                    self.schedule_toast_clear(ctx);
                    return true;
                }
                if self.selected_category.is_none() {
                    self.toast = Some(ToastState {
                        message: "請先選擇申請類別".to_string(),
                        is_error: true,
                    });
                    self.schedule_toast_clear(ctx);
                    return true;
                }
                if self.selected_items.is_empty() {
                    self.toast = Some(ToastState {
                        message: "請至少選擇一個申請項目".to_string(),
                        is_error: true,
                    });
                    self.schedule_toast_clear(ctx);
                    return true;
                }

                // Compile result text
                let (text, _) = get_generated_text(
                    &self.applicant_name,
                    &self.selected_category,
                    &self.selected_items,
                    false,
                );

                // Add ripple effect inside copy button if MouseEvent occurred
                self.handle_ripple_effect(e);

                // Copy async
                let text_clone = text.clone();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match copy_to_clipboard_async(text_clone).await {
                        Ok(_) => link.send_message(Msg::CopySuccess(text)),
                        Err(_) => link.send_message(Msg::CopyError),
                    }
                });
                false
            }
            Msg::CopySuccess(text) => {
                // Add to history list
                self.copy_history.retain(|h| h.text != text);
                let now = js_sys::Date::now() as u64;
                self.copy_history.insert(0, HistoryEntry { text: text.clone(), time: now });
                if self.copy_history.len() > 20 {
                    self.copy_history.truncate(20);
                }
                let _ = LocalStorage::set("medgen_history", &self.copy_history);

                // Add to recent names
                let name = self.applicant_name.trim().to_string();
                self.recent_names.retain(|n| n != &name);
                self.recent_names.insert(0, name);
                if self.recent_names.len() > 8 {
                    self.recent_names.truncate(8);
                }
                let _ = LocalStorage::set("medgen_names", &self.recent_names);

                // Show toast
                self.toast = Some(ToastState {
                    message: format!("已複製：{}", text),
                    is_error: false,
                });
                self.schedule_toast_clear(ctx);

                // Morph copy button
                self.copied_morph = true;
                let link = ctx.link().clone();
                self.morph_timeout = Some(Timeout::new(1600, move || {
                    link.send_message(Msg::ResetMorph);
                }));

                true
            }
            Msg::CopyError => {
                self.toast = Some(ToastState {
                    message: "複製失敗，請手動選取複製".to_string(),
                    is_error: true,
                });
                self.schedule_toast_clear(ctx);
                true
            }
            Msg::CopyFromHistory(text) => {
                let text_clone = text.clone();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match copy_to_clipboard_async(text_clone).await {
                        Ok(_) => link.send_message(Msg::CopySuccess(text)),
                        Err(_) => link.send_message(Msg::CopyError),
                    }
                });
                false
            }
            Msg::ToggleHistory => {
                self.history_open = !self.history_open;
                true
            }
            Msg::ClearHistory => {
                self.copy_history.clear();
                let _ = LocalStorage::set("medgen_history", &self.copy_history);
                self.toast = Some(ToastState {
                    message: "已清除複製紀錄".to_string(),
                    is_error: false,
                });
                self.schedule_toast_clear(ctx);
                true
            }
            Msg::ResetAll => {
                self.applicant_name.clear();
                self.selected_category = None;
                self.selected_items.clear();
                self.selected_group_tab = "全部".to_string();
                self.toast = Some(ToastState {
                    message: "已清除所有選擇".to_string(),
                    is_error: false,
                });
                self.schedule_toast_clear(ctx);
                true
            }
            Msg::ShowSuggestions => {
                self.name_suggestions_open = true;
                true
            }
            Msg::HideSuggestions => {
                // Add delay to prevent closing dropdown before click registration
                let link = ctx.link().clone();
                self.suggestions_timeout = Some(Timeout::new(150, move || {
                    link.send_message(Msg::SelectSuggestion("".to_string()));
                }));
                false
            }
            Msg::SelectSuggestion(name) => {
                self.name_suggestions_open = false;
                if !name.is_empty() {
                    self.applicant_name = name;
                    true
                } else {
                    true
                }
            }
            Msg::HideToast => {
                self.toast = None;
                true
            }
            Msg::ResetMorph => {
                self.copied_morph = false;
                true
            }
            Msg::TriggerInstall => {
                if let Some(ref prompt_ev) = self.deferred_prompt {
                    let _ = js_sys::Reflect::get(prompt_ev, &JsValue::from_str("prompt"))
                        .and_then(|func| {
                            if func.is_function() {
                                let func_obj = func.dyn_into::<js_sys::Function>()?;
                                let _ = func_obj.call0(prompt_ev);
                            }
                            Ok(JsValue::UNDEFINED)
                        });
                    self.deferred_prompt = None;
                }
                false
            }
            Msg::InstallPromptAvailable(ev) => {
                self.deferred_prompt = Some(ev);
                true
            }
            Msg::AppInstalled => {
                self.deferred_prompt = None;
                self.toast = Some(ToastState {
                    message: "感謝安裝！現在可以從桌面開啟".to_string(),
                    is_error: false,
                });
                self.schedule_toast_clear(ctx);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Evaluate completion status
        let (combined_text, is_complete) = get_generated_text(
            &self.applicant_name,
            &self.selected_category,
            &self.selected_items,
            true,
        );

        let has_any_input = !self.applicant_name.trim().is_empty()
            || self.selected_category.is_some()
            || !self.selected_items.is_empty();

        let desktop_preview_text = if is_complete || has_any_input {
            &combined_text
        } else {
            "請依序填寫左側欄位"
        };

        let mobile_preview_text = if is_complete || has_any_input {
            &combined_text
        } else {
            "請填寫上方欄位"
        };

        // Name status check
        let is_name_filled = !self.applicant_name.trim().is_empty();
        let dot1_class = if is_name_filled { "card-label-dot completed" } else { "card-label-dot active" };

        // Category status check
        let is_cat_selected = self.selected_category.is_some();
        let dot2_class = if is_cat_selected {
            "card-label-dot completed"
        } else if is_name_filled {
            "card-label-dot active"
        } else {
            "card-label-dot"
        };

        // Items status check
        let is_items_selected = !self.selected_items.is_empty();
        let dot3_class = if is_items_selected {
            "card-label-dot completed"
        } else if is_cat_selected {
            "card-label-dot active"
        } else {
            "card-label-dot"
        };

        // Setup filter values for tab buttons
        let mut group_tabs = vec!["全部"];
        for cat in CATEGORIES {
            if !group_tabs.contains(&cat.group) {
                group_tabs.push(cat.group);
            }
        }

        // Filter categories according to selected tab
        let filtered_categories: Vec<&Category> = CATEGORIES
            .iter()
            .filter(|cat| self.selected_group_tab == "全部" || cat.group == self.selected_group_tab)
            .collect();

        // Check if history is non-empty
        let history_badge_class = if !self.copy_history.is_empty() { "badge visible" } else { "badge" };
        let history_chevron_class = if self.history_open { "history-toggle open" } else { "history-toggle" };
        let history_body_class = if self.history_open { "history-body open" } else { "history-body" };

        // Render Suggestions List
        let current_input = self.applicant_name.trim();
        let filtered_suggestions: Vec<&String> = self.recent_names
            .iter()
            .filter(|name| {
                if current_input.is_empty() {
                    true
                } else {
                    name.contains(current_input) && name != &current_input
                }
            })
            .collect();

        let suggestions_box_class = if self.name_suggestions_open && !filtered_suggestions.is_empty() {
            "name-suggestions open"
        } else {
            "name-suggestions"
        };

        // PWA install button class
        let install_btn_class = if self.deferred_prompt.is_some() { "install-btn visible" } else { "install-btn" };

        html! {
            <>
                // Header
                <header class="app-header">
                    <div class="header-brand">
                        <div class="header-logo">
                            <svg width="18" height="18" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2"/>
                                <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
                            </svg>
                        </div>
                        <span class="header-title">{"醫事人員執業異動文字產生器"}</span>
                    </div>
                    <div class="header-actions">
                        <button class={install_btn_class} onclick={ctx.link().callback(|_| Msg::TriggerInstall)} aria-label="安裝應用">
                            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
                                <polyline points="7 10 12 15 17 10"/>
                                <line x1="12" y1="15" x2="12" y2="3"/>
                            </svg>
                            {"安裝"}
                        </button>
                        <button class="icon-btn" onclick={ctx.link().callback(|_| Msg::ToggleHistory)} aria-label="複製紀錄">
                            <svg width="18" height="18" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <polyline points="12 8 12 12 14 14"/>
                                <circle cx="12" cy="12" r="10"/>
                            </svg>
                            <span id="historyBadge" class={history_badge_class}>{self.copy_history.len()}</span>
                        </button>
                        <button class="icon-btn" onclick={ctx.link().callback(|_| Msg::ResetAll)} aria-label="清除重填">
                            <svg width="18" height="18" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M3 12a9 9 0 019-9 9.75 9.75 0 016.74 2.74L21 8"/>
                                <path d="M21 3v5h-5"/>
                                <path d="M21 12a9 9 0 01-9 9 9.75 9.75 0 01-6.74-2.74L3 16"/>
                                <path d="M3 21v-5h5"/>
                            </svg>
                        </button>
                    </div>
                </header>

                // Layout
                <div class="app-layout">
                    // Left Input Column
                    <div class="col-input">
                        // Applicant Name Card
                        <div class="card anim-in anim-in-1" aria-label="輸入申請人姓名">
                            <div class="card-header">
                                <div class="card-label">
                                    <span id="dot1" class={dot1_class}></span>
                                    <span class="card-label-text">{"申請人姓名"}</span>
                                </div>
                            </div>
                            <div class="card-body">
                                <div class="name-field">
                                    <input
                                        type="text"
                                        id="applicantName"
                                        class="name-input"
                                        placeholder="輸入姓名，例如：陳小明"
                                        autocomplete="off"
                                        maxlength="50"
                                        aria-label="申請人姓名"
                                        value={self.applicant_name.clone()}
                                        oninput={ctx.link().callback(|e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            Msg::UpdateName(input.value())
                                        })}
                                        onfocus={ctx.link().callback(|_| Msg::ShowSuggestions)}
                                        onblur={ctx.link().callback(|_| Msg::HideSuggestions)}
                                    />
                                    <button
                                        id="nameClear"
                                        class={if is_name_filled { "name-clear visible" } else { "name-clear" }}
                                        onclick={ctx.link().callback(|_| Msg::ClearName)}
                                        aria-label="清除姓名"
                                    >
                                        <svg width="14" height="14" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5" stroke-linecap="round">
                                            <path d="M18 6L6 18M6 6l12 12"/>
                                        </svg>
                                    </button>
                                    <div class={suggestions_box_class}>
                                        {for filtered_suggestions.into_iter().map(|name| {
                                            let n_clone = name.clone();
                                            html! {
                                                <button
                                                    type="button"
                                                    class="name-suggestion-item"
                                                    onmousedown={ctx.link().callback(move |_| Msg::SelectSuggestion(n_clone.clone()))}
                                                >
                                                    <span>{name}</span>
                                                    <span class="hint">{"最近使用"}</span>
                                                </button>
                                            }
                                        })}
                                    </div>
                                </div>
                            </div>
                        </div>

                        // Category Card
                        <div class="card anim-in anim-in-2" aria-label="選擇申請類別">
                            <div class="card-header">
                                <div class="card-label">
                                    <span id="dot2" class={dot2_class}></span>
                                    <span class="card-label-text">{"申請類別"}</span>
                                </div>
                                <span class="card-badge">{"單選"}</span>
                            </div>
                            <div class="card-body">
                                <div class="tabs-scroll">
                                    {for group_tabs.into_iter().map(|tab| {
                                        let tab_str = tab.to_string();
                                        let active = self.selected_group_tab == tab_str;
                                        html! {
                                            <button
                                                type="button"
                                                class={if active { "tab-pill active" } else { "tab-pill" }}
                                                onclick={ctx.link().callback(move |_| Msg::SelectTab(tab_str.clone()))}
                                            >
                                                {tab}
                                            </button>
                                        }
                                    })}
                                </div>
                                <div class="cat-grid">
                                    {for filtered_categories.into_iter().map(|cat| {
                                        let cat_id = cat.id.to_string();
                                        let selected = self.selected_category.as_ref() == Some(&cat_id);
                                        html! {
                                            <button
                                                type="button"
                                                class={if selected { "cat-btn selected" } else { "cat-btn" }}
                                                aria-pressed={if selected { "true" } else { "false" }}
                                                onclick={ctx.link().callback(move |_| Msg::SelectCategory(cat_id.clone()))}
                                            >
                                                {cat.label}
                                            </button>
                                        }
                                    })}
                                </div>
                            </div>
                        </div>

                        // Items Card
                        <div class="card anim-in anim-in-3" aria-label="選擇申請項目">
                            <div class="card-header">
                                <div class="card-label">
                                    <span id="dot3" class={dot3_class}></span>
                                    <span class="card-label-text">{"申請項目"}</span>
                                </div>
                                <div style="display:flex;align-items:center;gap:0.35rem;">
                                    <span class="card-badge">{"複選"}</span>
                                    <span id="itemCountBadge" class={if is_items_selected { "item-count-badge visible" } else { "item-count-badge" }}>
                                        {self.selected_items.len()}
                                    </span>
                                </div>
                            </div>
                            <div class="card-body">
                                <div class="items-grid">
                                    {for ITEMS.iter().map(|item| {
                                        let item_id = item.id.to_string();
                                        let selected = self.selected_items.contains(&item_id);
                                        html! {
                                            <button
                                                type="button"
                                                class={if selected { "item-chip selected" } else { "item-chip" }}
                                                aria-pressed={if selected { "true" } else { "false" }}
                                                onclick={ctx.link().callback(move |_| Msg::ToggleItem(item_id.clone()))}
                                            >
                                                <span class="check-dot">
                                                    <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"/>
                                                    </svg>
                                                </span>
                                                <span>{item.label}</span>
                                            </button>
                                        }
                                    })}
                                </div>
                            </div>
                        </div>
                    </div>

                    // Right column
                    <div class="col-preview anim-in anim-in-4">
                        // Preview result
                        <div class="preview-card">
                            <div class="preview-header">
                                <span class="preview-label">{"產生結果"}</span>
                                <span id="previewStatus" class={if is_complete { "preview-status ready" } else { "preview-status incomplete" }}>
                                    {if is_complete { "可複製" } else { "未完成" }}
                                </span>
                            </div>
                            <div class="preview-body">
                                <div class="preview-result">
                                    <p id="outputResult" class={if is_complete || has_any_input { "preview-result-text" } else { "preview-result-text placeholder" }}>
                                        {desktop_preview_text}
                                    </p>
                                </div>
                                <button
                                    class="copy-btn primary"
                                    id="desktopCopyBtn"
                                    disabled={!is_complete}
                                    onclick={ctx.link().callback(|e: MouseEvent| Msg::CopyText(e))}
                                >
                                    <svg id="copyIcon" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        {if self.copied_morph {
                                            html! { <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7"/> }
                                        } else {
                                            html! {
                                                <>
                                                    <rect x="9" y="9" width="13" height="13" rx="2"/>
                                                    <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
                                                </>
                                            }
                                        }}
                                    </svg>
                                    <span id="copyBtnText">
                                        {if self.copied_morph { "已複製！" } else { "複製文字" }}
                                    </span>
                                </button>
                                <div class="shortcut-hint">
                                    <span class="key-badge">{"Ctrl"}</span>
                                    <span>{"+"}</span>
                                    <span class="key-badge">{"Enter"}</span>
                                    <span>{"快速複製"}</span>
                                </div>
                            </div>
                        </div>

                        // History
                        <div class="history-card">
                            <div class="history-header" onclick={ctx.link().callback(|_| Msg::ToggleHistory)}>
                                <span class="history-label">
                                    <svg width="14" height="14" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round">
                                        <polyline points="12 8 12 12 14 14"/>
                                        <circle cx="12" cy="12" r="10"/>
                                    </svg>
                                    {"複製紀錄"}
                                </span>
                                <div style="display:flex;align-items:center;gap:0.35rem;">
                                    {if !self.copy_history.is_empty() {
                                        html! {
                                            <button
                                                class="history-clear-btn"
                                                id="historyClearBtn"
                                                onclick={ctx.link().callback(|e: MouseEvent| {
                                                    e.stop_propagation();
                                                    Msg::ClearHistory
                                                })}
                                            >
                                                {"清除"}
                                            </button>
                                        }
                                    } else {
                                        html! {}
                                    }}
                                    <svg id="historyChevron" class={history_chevron_class} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                        <polyline points="6 9 12 15 18 9"/>
                                    </svg>
                                </div>
                            </div>
                            <div class={history_body_class} id="historyBody">
                                <div class="history-list" id="historyList">
                                    {if self.copy_history.is_empty() {
                                        html! { <div class="history-empty">{"尚無複製紀錄"}</div> }
                                    } else {
                                        html! {
                                            {for self.copy_history.iter().map(|entry| {
                                                let txt = entry.text.clone();
                                                html! {
                                                    <button
                                                        type="button"
                                                        class="history-item"
                                                        onclick={ctx.link().callback(move |_| Msg::CopyFromHistory(txt.clone()))}
                                                    >
                                                        <span class="history-item-text">{&entry.text}</span>
                                                        <span class="history-item-copy">
                                                            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round">
                                                                <rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
                                                            </svg>
                                                        </span>
                                                    </button>
                                                }
                                            })}
                                        }
                                    }}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                // Sticky Bottom Mobile Bar
                <div class="mobile-bar">
                    <div class="mobile-bar-preview" id="mobilePreview">
                        {if is_complete {
                            html! { <strong>{mobile_preview_text}</strong> }
                        } else {
                            html! { <>{mobile_preview_text}</> }
                        }}
                    </div>
                    <button
                        class="mobile-copy-btn"
                        id="mobileCopyBtn"
                        disabled={!is_complete}
                        onclick={ctx.link().callback(|e: MouseEvent| Msg::CopyText(e))}
                    >
                        <svg width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            {if self.copied_morph {
                                html! { <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7"/> }
                            } else {
                                html! {
                                    <>
                                        <rect x="9" y="9" width="13" height="13" rx="2"/>
                                        <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
                                    </>
                                }
                            }}
                        </svg>
                        <span id="mobileCopyText">
                            {if self.copied_morph { "已複製！" } else { "複製文字" }}
                        </span>
                    </button>
                </div>

                // Toast Notification
                <div class={if self.toast.is_some() { "toast show" } else { "toast" }} id="toastEl" role="alert" aria-live="polite">
                    {if let Some(ref t) = self.toast {
                        let inner_class = if t.is_error { "toast-inner error" } else { "toast-inner success" };
                        let icon_class = if t.is_error { "toast-icon error" } else { "toast-icon success" };
                        let bar_class = if t.is_error { "toast-bar error animate" } else { "toast-bar success animate" };
                        html! {
                            <div class={inner_class} id="toastInner">
                                <svg class={icon_class} id="toastIcon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    {if t.is_error {
                                        html! { <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12"/> }
                                    } else {
                                        html! { <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7"/> }
                                    }}
                                </svg>
                                <span id="toastContent">{&t.message}</span>
                                <div class={bar_class} id="toastBar"></div>
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </div>
            </>
        }
    }
}

impl App {
    fn schedule_toast_clear(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        self.toast_timeout = Some(Timeout::new(2800, move || {
            link.send_message(Msg::HideToast);
        }));
    }

    fn handle_ripple_effect(&self, e: MouseEvent) {
        if let Some(target) = e.current_target() {
            if let Ok(btn) = target.dyn_into::<web_sys::HtmlElement>() {
                let rect = btn.get_bounding_client_rect();
                let size = rect.width().max(rect.height());
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    if let Ok(ripple) = doc.create_element("span") {
                        let _ = ripple.set_attribute("class", "ripple");
                        let _ = ripple.set_attribute("style", &format!(
                            "width: {}px; height: {}px; left: {}px; top: {}px; position: absolute; border-radius: 50%; background: rgba(255,255,255,0.25); transform: scale(0); pointer-events: none; animation: ripple 0.5s ease-out;",
                            size,
                            size,
                            e.client_x() as f64 - rect.left() - size / 2.0,
                            e.client_y() as f64 - rect.top() - size / 2.0
                        ));
                        let _ = btn.append_child(&ripple);
                        let ripple_clone = ripple.clone();
                        let closure = Closure::wrap(Box::new(move || {
                            ripple_clone.remove();
                        }) as Box<dyn FnMut()>);
                        let _ = ripple.add_event_listener_with_callback("animationend", closure.as_ref().unchecked_ref());
                        closure.forget();
                    }
                }
            }
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
