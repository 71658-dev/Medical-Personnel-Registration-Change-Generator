use gloo_storage::{LocalStorage, Storage};
use gloo_timers::callback::Timeout;
use std::collections::HashSet;
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

/// 應備文件 master list. The numeric `code` is the stable identity — the
/// per-item tables below reference documents by code, and `checked_docs`
/// stores codes, so relabelling a document never loses a tick.
struct DocumentRow {
    code: u8,
    label: &'static str,
}

const DOCUMENTS: &[DocumentRow] = &[
    DocumentRow { code: 1, label: "公會證明文件(執業、換照、變更、歇業、停業、復業)" },
    DocumentRow { code: 2, label: "證書及專科證書正本(影本)" },
    DocumentRow { code: 3, label: "新服務機構在職證明" },
    DocumentRow { code: 4, label: "服務機構停業、復業證明" },
    DocumentRow { code: 5, label: "服務機構登記變更證明" },
    DocumentRow { code: 6, label: "服務機構離職證明" },
    DocumentRow { code: 7, label: "身分證正本(影本)" },
    DocumentRow { code: 8, label: "原執業執照" },
    DocumentRow { code: 9, label: "執業執照遺失切結書" },
    DocumentRow { code: 10, label: "規費300元" },
    DocumentRow { code: 11, label: "照片1吋2張(近照三個月)" },
    DocumentRow { code: 12, label: "繼續教育學分證明" },
    DocumentRow { code: 13, label: "委託書、被委託人身分證影本(非本人辦理)" },
];

/// Required for every 申請項目, appended once any item is selected.
const BASE_DOC_CODES: &[u8] = &[13];

struct ItemDocs {
    item_id: &'static str,
    codes: &'static [u8],
}

const ITEM_DOC_CODES: &[ItemDocs] = &[
    ItemDocs { item_id: "register", codes: &[1, 2, 3, 7, 10, 11] },
    ItemDocs { item_id: "suspend", codes: &[2, 4, 7, 8] },
    ItemDocs { item_id: "resume", codes: &[2, 4, 7, 8] },
    ItemDocs { item_id: "cessation", codes: &[1, 2, 6, 7, 8] },
    ItemDocs { item_id: "dept_change", codes: &[1, 2, 5, 7, 8, 10, 11] },
    ItemDocs { item_id: "name_change", codes: &[1, 2, 5, 7, 8, 10, 11] },
    ItemDocs { item_id: "inst_change", codes: &[1, 2, 3, 6, 7, 8, 10, 11] },
    ItemDocs { item_id: "cat_change", codes: &[1, 2, 5, 7, 8, 10, 11] },
    ItemDocs { item_id: "lost_reissue", codes: &[2, 7, 9, 10, 11] },
    ItemDocs { item_id: "damage_reissue", codes: &[2, 7, 8, 10, 11] },
    ItemDocs { item_id: "renew", codes: &[1, 2, 7, 8, 10, 11, 12] },
];

/// Documents an item only needs for certain 申請類別.
struct ConditionalDoc {
    item_id: &'static str,
    categories: &'static [&'static str],
    code: u8,
}

// 停業辦理人為護理師(護士)或醫師時，需另附公會證明文件(1)。
const ITEM_CONDITIONAL_DOC_CODES: &[ConditionalDoc] = &[ConditionalDoc {
    item_id: "suspend",
    categories: &["nurse", "physician"],
    code: 1,
}];

/// How long the three fields must sit unchanged before the text is copied
/// automatically.
const AUTO_COPY_DELAY_MS: u32 = 600;

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

/// The 應備文件 for the current selection: the union of every selected item's
/// documents (plus any 類別-conditional extras and the always-required base
/// set), sorted by code so the checklist order stays stable no matter which
/// order the items were ticked in. Empty until at least one item is selected.
fn required_doc_codes(selected_items: &[String], category_id: &Option<String>) -> Vec<u8> {
    if selected_items.is_empty() {
        return Vec::new();
    }

    let mut codes: Vec<u8> = Vec::new();
    fn push(codes: &mut Vec<u8>, code: u8) {
        if !codes.contains(&code) {
            codes.push(code);
        }
    }

    for item_id in selected_items {
        if let Some(entry) = ITEM_DOC_CODES.iter().find(|e| e.item_id == item_id) {
            for &code in entry.codes {
                push(&mut codes, code);
            }
        }
        for cond in ITEM_CONDITIONAL_DOC_CODES.iter().filter(|c| c.item_id == item_id) {
            let matches_category = category_id
                .as_ref()
                .map_or(false, |cat| cond.categories.contains(&cat.as_str()));
            if matches_category {
                push(&mut codes, cond.code);
            }
        }
    }
    for &code in BASE_DOC_CODES {
        push(&mut codes, code);
    }

    codes.sort_unstable();
    codes
}

fn doc_label(code: u8) -> &'static str {
    DOCUMENTS
        .iter()
        .find(|d| d.code == code)
        .map_or("", |d| d.label)
}

// Auto-focus is desktop-only: on a phone it pops the virtual keyboard on load
// and again after every copy, shoving the sticky `.mobile-bar` up the screen.
// Queried as a media query rather than `inner_width` so it stays locked to the
// same 800px breakpoint `style.css` collapses the two-column layout at.
fn is_desktop_viewport() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(min-width: 800px)").ok().flatten())
        .map_or(false, |mql| mql.matches())
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
    ToggleDoc(u8),
    CopyText,
    CopySuccess(String),
    CopyError,
    AutoCopy,
    AutoCopySuccess(String),
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
    // Ticked 應備文件, by document code. Session-only and deliberately not
    // persisted — it tracks one visit to the counter, not a standing list.
    checked_docs: HashSet<u8>,
    copy_history: Vec<HistoryEntry>,
    recent_names: Vec<String>,
    history_open: bool,
    toast: Option<ToastState>,
    name_suggestions_open: bool,
    copied_morph: bool,
    name_ref: NodeRef,
    // Set when something should hand focus back to the name input on the next
    // render (see `rendered`); first render focuses unconditionally.
    focus_name_pending: bool,
    // Programmatic focus fires a `focus` event just like a click would; this
    // swallows the resulting ShowSuggestions so the 最近使用 dropdown only
    // opens when the user actually reaches for the field.
    suppress_focus_suggestions: bool,
    // What the clipboard currently holds, so an auto-copy never repeats itself
    // and never fights a manual copy of the same text.
    last_copied_text: String,
    toast_timeout: Option<Timeout>,
    morph_timeout: Option<Timeout>,
    suggestions_timeout: Option<Timeout>,
    auto_copy_timeout: Option<Timeout>,
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
                link.send_message(Msg::CopyText);
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
            checked_docs: HashSet::new(),
            copy_history,
            recent_names,
            history_open: false,
            toast: None,
            name_suggestions_open: false,
            copied_morph: false,
            name_ref: NodeRef::default(),
            focus_name_pending: false,
            suppress_focus_suggestions: false,
            last_copied_text: String::new(),
            toast_timeout: None,
            morph_timeout: None,
            suggestions_timeout: None,
            auto_copy_timeout: None,
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
                self.schedule_auto_copy(ctx);
                true
            }
            Msg::ClearName => {
                self.applicant_name.clear();
                self.schedule_auto_copy(ctx);
                true
            }
            Msg::SelectCategory(cat_id) => {
                self.selected_category = Some(cat_id);
                self.selected_items.clear();
                self.schedule_auto_copy(ctx);
                true
            }
            Msg::ToggleItem(item_id) => {
                if let Some(pos) = self.selected_items.iter().position(|x| x == &item_id) {
                    self.selected_items.remove(pos);
                } else {
                    self.selected_items.push(item_id);
                }
                self.schedule_auto_copy(ctx);
                true
            }
            Msg::SelectTab(tab) => {
                self.selected_group_tab = tab;
                true
            }
            Msg::ToggleDoc(code) => {
                if !self.checked_docs.remove(&code) {
                    self.checked_docs.insert(code);
                }
                true
            }
            Msg::CopyText => {
                // Perform checks
                let name_trimmed = self.applicant_name.trim();
                if name_trimmed.is_empty() {
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
                // Keeps the debounced auto-copy from immediately repeating this
                self.last_copied_text = text.clone();
                self.auto_copy_timeout = None;

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

                // Hand focus back so the next 申請人 can be typed straight away
                self.focus_name_pending = true;

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
            Msg::AutoCopy => {
                // Re-check: the debounce may have been armed before a later
                // edit made the form incomplete again.
                let (text, is_complete) = get_generated_text(
                    &self.applicant_name,
                    &self.selected_category,
                    &self.selected_items,
                    false,
                );
                if !is_complete || text == self.last_copied_text {
                    return false;
                }

                let text_clone = text.clone();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    // Failure stays silent on purpose. The user never asked for
                    // this copy, so nagging them with an error toast (browsers
                    // that require a user gesture reject it every time) would be
                    // pure noise — the copy button still reports its own errors.
                    if copy_to_clipboard_async(text_clone).await.is_ok() {
                        link.send_message(Msg::AutoCopySuccess(text));
                    }
                });
                false
            }
            Msg::AutoCopySuccess(text) => {
                self.last_copied_text = text.clone();
                // Deliberately not recorded in 複製紀錄 / 最近使用 — those stay
                // reserved for copies the user pressed the button for.
                self.toast = Some(ToastState {
                    message: format!("已自動複製：{}", text),
                    is_error: false,
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
                self.checked_docs.clear();
                self.schedule_auto_copy(ctx);
                self.toast = Some(ToastState {
                    message: "已清除所有選擇".to_string(),
                    is_error: false,
                });
                self.schedule_toast_clear(ctx);
                true
            }
            Msg::ShowSuggestions => {
                if self.suppress_focus_suggestions {
                    self.suppress_focus_suggestions = false;
                    return false;
                }
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
                    self.schedule_auto_copy(ctx);
                }
                true
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
                true
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

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if !first_render && !self.focus_name_pending {
            return;
        }
        self.focus_name_pending = false;

        if !is_desktop_viewport() {
            return;
        }

        if let Some(input) = self.name_ref.cast::<web_sys::HtmlInputElement>() {
            // `focus()` on the already-focused element fires no event, so only
            // arm the suppression when one is actually coming — otherwise the
            // flag would go stale and eat the user's next real focus.
            let node: &web_sys::Node = input.as_ref();
            let already_focused = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.active_element())
                .map_or(false, |el| el.is_same_node(Some(node)));

            if !already_focused {
                self.suppress_focus_suggestions = true;
                let _ = input.focus();
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (combined_text, is_complete) = get_generated_text(
            &self.applicant_name,
            &self.selected_category,
            &self.selected_items,
            true,
        );

        let has_any_input = !self.applicant_name.trim().is_empty()
            || self.selected_category.is_some()
            || !self.selected_items.is_empty();
        let show_result = is_complete || has_any_input;

        let desktop_preview_text = if show_result { &combined_text } else { "請依序填寫左側欄位" };
        let mobile_preview_text = if show_result { &combined_text } else { "請填寫上方欄位" };

        // Step completion tags replace the old progress dots
        let is_name_filled = !self.applicant_name.trim().is_empty();
        let is_cat_selected = self.selected_category.is_some();
        let is_items_selected = !self.selected_items.is_empty();

        let name_tag_class = if is_name_filled { "tag tag-accent" } else { "tag tag-neutral" };
        let name_tag_label = if is_name_filled { "已填寫" } else { "待填寫" };
        let cat_tag_class = if is_cat_selected { "tag tag-accent" } else { "tag tag-neutral" };
        let cat_tag_label = if is_cat_selected { "已選擇" } else { "待選擇" };
        let items_tag_class = if is_items_selected { "tag tag-accent" } else { "tag tag-neutral" };
        let items_tag_label = if is_items_selected {
            format!("已選 {}", self.selected_items.len())
        } else {
            "待選擇".to_string()
        };

        // Tab list is derived from the distinct `group` values in CATEGORIES
        let mut group_tabs = vec!["全部"];
        for cat in CATEGORIES {
            if !group_tabs.contains(&cat.group) {
                group_tabs.push(cat.group);
            }
        }

        let filtered_categories: Vec<&Category> = CATEGORIES
            .iter()
            .filter(|cat| self.selected_group_tab == "全部" || cat.group == self.selected_group_tab)
            .collect();

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
        let suggestions_visible = self.name_suggestions_open && !filtered_suggestions.is_empty();

        let doc_codes = required_doc_codes(&self.selected_items, &self.selected_category);
        let docs_checked_count = doc_codes.iter().filter(|c| self.checked_docs.contains(c)).count();

        html! {
            <div class="app-shell">
                // ─── Header ───
                <header class="nav">
                    <span class="nav-brand">{"醫事人員執業異動文字產生器"}</span>
                    <div class="nav-actions">
                        {if self.deferred_prompt.is_some() {
                            html! {
                                <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::TriggerInstall)} aria-label="安裝應用">
                                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
                                        <polyline points="7 10 12 15 17 10"/>
                                        <line x1="12" y1="15" x2="12" y2="3"/>
                                    </svg>
                                    {"安裝"}
                                </button>
                            }
                        } else {
                            html! {}
                        }}
                        <button class="btn btn-icon btn-secondary history-btn" onclick={ctx.link().callback(|_| Msg::ToggleHistory)} aria-label="複製紀錄">
                            <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <polyline points="12 8 12 12 14 14"/>
                                <circle cx="12" cy="12" r="10"/>
                            </svg>
                            {if !self.copy_history.is_empty() {
                                html! { <span class="history-badge">{self.copy_history.len()}</span> }
                            } else {
                                html! {}
                            }}
                        </button>
                        <button class="btn btn-icon btn-secondary" onclick={ctx.link().callback(|_| Msg::ResetAll)} aria-label="清除重填">
                            <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M3 12a9 9 0 019-9 9.75 9.75 0 016.74 2.74L21 8"/>
                                <path d="M21 3v5h-5"/>
                                <path d="M21 12a9 9 0 01-9 9 9.75 9.75 0 01-6.74-2.74L3 16"/>
                                <path d="M3 21v-5h5"/>
                            </svg>
                        </button>
                    </div>
                </header>

                <div class="app-layout">
                    // ─── Input column ───
                    <div class="col-input">

                        // STEP 01 — name
                        <section class="card anim-in" aria-label="輸入申請人姓名">
                            <div class="card-head">
                                <div>
                                    <div class="card-kicker">{"STEP 01"}</div>
                                    <div class="card-title">{"申請人姓名"}</div>
                                </div>
                                <span class={name_tag_class}>{name_tag_label}</span>
                            </div>
                            <div class="hr hr-tight"></div>
                            <div class="name-field">
                                <input
                                    type="text"
                                    id="applicantName"
                                    class="input name-input"
                                    placeholder="輸入姓名，例如：陳小明"
                                    autocomplete="off"
                                    maxlength="50"
                                    aria-label="申請人姓名"
                                    ref={self.name_ref.clone()}
                                    value={self.applicant_name.clone()}
                                    oninput={ctx.link().callback(|e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        Msg::UpdateName(input.value())
                                    })}
                                    onfocus={ctx.link().callback(|_| Msg::ShowSuggestions)}
                                    onblur={ctx.link().callback(|_| Msg::HideSuggestions)}
                                />
                                {if is_name_filled {
                                    html! {
                                        <button class="name-clear" onclick={ctx.link().callback(|_| Msg::ClearName)} aria-label="清除姓名">
                                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round">
                                                <path d="M18 6L6 18M6 6l12 12"/>
                                            </svg>
                                        </button>
                                    }
                                } else {
                                    html! {}
                                }}
                                {if suggestions_visible {
                                    html! {
                                        <div class="name-suggestions">
                                            {for filtered_suggestions.into_iter().map(|name| {
                                                let n_clone = name.clone();
                                                html! {
                                                    <button
                                                        type="button"
                                                        class="name-suggestion-item"
                                                        onmousedown={ctx.link().callback(move |_| Msg::SelectSuggestion(n_clone.clone()))}
                                                    >
                                                        <span>{name}</span>
                                                        <span class="name-suggestion-hint">{"最近使用"}</span>
                                                    </button>
                                                }
                                            })}
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                        </section>

                        // STEP 02 — category
                        <section class="card anim-in anim-in-2" aria-label="選擇申請類別">
                            <div class="card-head">
                                <div>
                                    <div class="card-kicker">{"STEP 02"}</div>
                                    <div class="card-title">{"申請類別"}</div>
                                </div>
                                <div class="tag-group">
                                    <span class="tag tag-outline tag-mode">{"單選"}</span>
                                    <span class={cat_tag_class}>{cat_tag_label}</span>
                                </div>
                            </div>
                            <div class="hr hr-tight"></div>
                            <div class="tabs-scroll">
                                {for group_tabs.into_iter().map(|tab| {
                                    let tab_str = tab.to_string();
                                    let active = self.selected_group_tab == tab_str;
                                    html! {
                                        <button
                                            type="button"
                                            class={if active { "tab-btn active" } else { "tab-btn" }}
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
                                            class={if selected { "btn opt-btn selected" } else { "btn opt-btn" }}
                                            aria-pressed={if selected { "true" } else { "false" }}
                                            onclick={ctx.link().callback(move |_| Msg::SelectCategory(cat_id.clone()))}
                                        >
                                            {cat.label}
                                        </button>
                                    }
                                })}
                            </div>
                        </section>

                        // STEP 03 — items
                        <section class="card anim-in anim-in-3" aria-label="選擇申請項目">
                            <div class="card-head">
                                <div>
                                    <div class="card-kicker">{"STEP 03"}</div>
                                    <div class="card-title">{"申請項目"}</div>
                                </div>
                                <div class="tag-group">
                                    <span class="tag tag-outline tag-mode">{"複選"}</span>
                                    <span class={items_tag_class}>{items_tag_label}</span>
                                </div>
                            </div>
                            <div class="hr hr-tight"></div>
                            <div class="items-grid">
                                {for ITEMS.iter().map(|item| {
                                    let item_id = item.id.to_string();
                                    let selected = self.selected_items.contains(&item_id);
                                    html! {
                                        <button
                                            type="button"
                                            class={if selected { "btn opt-btn item-btn selected" } else { "btn opt-btn item-btn" }}
                                            aria-pressed={if selected { "true" } else { "false" }}
                                            onclick={ctx.link().callback(move |_| Msg::ToggleItem(item_id.clone()))}
                                        >
                                            <span class="check-box">
                                                {if selected {
                                                    html! {
                                                        <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="var(--color-bg)" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round">
                                                            <path d="M5 13l4 4L19 7"/>
                                                        </svg>
                                                    }
                                                } else {
                                                    html! {}
                                                }}
                                            </span>
                                            {item.label}
                                        </button>
                                    }
                                })}
                            </div>
                        </section>
                    </div>

                    // ─── Preview column ───
                    <div class="col-preview anim-in anim-in-4">

                        <section class="card">
                            <div class="card-head card-head-center">
                                <div class="card-title">{"產生結果"}</div>
                                <span class={if is_complete { "tag tag-accent" } else { "tag tag-neutral" }}>
                                    {if is_complete { "可複製" } else { "未完成" }}
                                </span>
                            </div>
                            <div class="hr hr-tight"></div>
                            <div class="preview-box">
                                <p id="outputResult" class={if show_result { "preview-text" } else { "preview-text placeholder" }}>
                                    {desktop_preview_text}
                                </p>
                            </div>
                            <button
                                class="btn btn-block btn-copy"
                                id="desktopCopyBtn"
                                disabled={!is_complete}
                                onclick={ctx.link().callback(|_| Msg::CopyText)}
                            >
                                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                    {if self.copied_morph {
                                        html! { <path d="M5 13l4 4L19 7"/> }
                                    } else {
                                        html! {
                                            <>
                                                <rect x="9" y="9" width="13" height="13" rx="1"/>
                                                <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
                                            </>
                                        }
                                    }}
                                </svg>
                                {if self.copied_morph { "已複製！" } else { "複製文字" }}
                            </button>
                            <div class="shortcut-hint">
                                <span class="key-cap">{"Ctrl"}</span>
                                <span>{"+"}</span>
                                <span class="key-cap">{"Enter"}</span>
                                <span>{"快速複製"}</span>
                            </div>
                        </section>

                        // 應備文件檢核表 — derived from the selected 申請項目
                        <section class="card docs-card" aria-label="應備文件檢核表">
                            <div class="card-head card-head-center">
                                <div class="card-title">{"應備文件檢核表"}</div>
                                {if doc_codes.is_empty() {
                                    html! {}
                                } else {
                                    html! {
                                        <span class="tag tag-outline tag-mode">
                                            {format!("{}/{}", docs_checked_count, doc_codes.len())}
                                        </span>
                                    }
                                }}
                            </div>
                            <div class="hr hr-tight"></div>
                            {if doc_codes.is_empty() {
                                html! { <div class="docs-empty">{"選擇申請項目後，將自動列出應備文件"}</div> }
                            } else {
                                html! {
                                    <div class="docs-list">
                                        {for doc_codes.iter().map(|&code| {
                                            let checked = self.checked_docs.contains(&code);
                                            html! {
                                                <button
                                                    type="button"
                                                    class={if checked { "doc-item checked" } else { "doc-item" }}
                                                    aria-pressed={if checked { "true" } else { "false" }}
                                                    onclick={ctx.link().callback(move |_| Msg::ToggleDoc(code))}
                                                >
                                                    <span class="check-box">
                                                        {if checked {
                                                            html! {
                                                                <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="var(--color-bg)" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round">
                                                                    <path d="M5 13l4 4L19 7"/>
                                                                </svg>
                                                            }
                                                        } else {
                                                            html! {}
                                                        }}
                                                    </span>
                                                    <span class="doc-label">{doc_label(code)}</span>
                                                </button>
                                            }
                                        })}
                                    </div>
                                }
                            }}
                        </section>

                        <section class="card history-card">
                            <div class="history-header" onclick={ctx.link().callback(|_| Msg::ToggleHistory)}>
                                <span class="history-label">
                                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                        <polyline points="12 8 12 12 14 14"/>
                                        <circle cx="12" cy="12" r="10"/>
                                    </svg>
                                    {"複製紀錄"}
                                </span>
                                <div class="history-header-actions">
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
                                    <svg
                                        width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
                                        class={if self.history_open { "history-chevron open" } else { "history-chevron" }}
                                    >
                                        <polyline points="6 9 12 15 18 9"/>
                                    </svg>
                                </div>
                            </div>
                            {if self.history_open {
                                html! {
                                    <>
                                        <div class="hr" style="margin:0;"></div>
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
                                                                {&entry.text}
                                                            </button>
                                                        }
                                                    })}
                                                }
                                            }}
                                        </div>
                                    </>
                                }
                            } else {
                                html! {}
                            }}
                        </section>
                    </div>
                </div>

                // ─── Sticky mobile bar (shown below 800px) ───
                <div class="mobile-bar">
                    <div
                        id="mobilePreview"
                        class={if !show_result {
                            "mobile-preview placeholder"
                        } else if is_complete {
                            "mobile-preview ready"
                        } else {
                            "mobile-preview"
                        }}
                    >
                        {mobile_preview_text}
                    </div>
                    <button
                        class="btn btn-copy mobile-copy-btn"
                        id="mobileCopyBtn"
                        disabled={!is_complete}
                        onclick={ctx.link().callback(|_| Msg::CopyText)}
                    >
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                            {if self.copied_morph {
                                html! { <path d="M5 13l4 4L19 7"/> }
                            } else {
                                html! {
                                    <>
                                        <rect x="9" y="9" width="13" height="13" rx="1"/>
                                        <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
                                    </>
                                }
                            }}
                        </svg>
                        {if self.copied_morph { "已複製！" } else { "複製文字" }}
                    </button>
                </div>

                // ─── Toast ───
                {if let Some(ref t) = self.toast {
                    html! {
                        <div class="toast-wrap" id="toastEl" role="alert" aria-live="polite">
                            <div class="toast-inner">
                                <svg class="toast-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--color-accent-700)" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
                                    {if t.is_error {
                                        html! { <path d="M6 18L18 6M6 6l12 12"/> }
                                    } else {
                                        html! { <path d="M5 13l4 4L19 7"/> }
                                    }}
                                </svg>
                                <span id="toastContent">{&t.message}</span>
                                <div class="toast-bar"></div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        }
    }
}

impl App {
    /// Arm (or re-arm) the debounced auto-copy after any edit to the three
    /// fields. Dropping the previous `Timeout` cancels it, so the copy only
    /// runs once the user has stopped for `AUTO_COPY_DELAY_MS` — otherwise
    /// every keystroke of a name would land a half-typed string on the
    /// clipboard.
    fn schedule_auto_copy(&mut self, ctx: &Context<Self>) {
        self.auto_copy_timeout = None;

        let (text, is_complete) = get_generated_text(
            &self.applicant_name,
            &self.selected_category,
            &self.selected_items,
            false,
        );
        if !is_complete || text == self.last_copied_text {
            return;
        }

        let link = ctx.link().clone();
        self.auto_copy_timeout = Some(Timeout::new(AUTO_COPY_DELAY_MS, move || {
            link.send_message(Msg::AutoCopy);
        }));
    }

    fn schedule_toast_clear(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        self.toast_timeout = Some(Timeout::new(2800, move || {
            link.send_message(Msg::HideToast);
        }));
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
