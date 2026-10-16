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
struct CategoryDef {
    id: &'static str,
    label: &'static str,
    group: &'static str,
}

const DEFAULT_CATEGORIES: &[CategoryDef] = &[
    CategoryDef { id: "physician", label: "醫師", group: "醫政科" },
    CategoryDef { id: "tcm_physician", label: "中醫師", group: "醫政科" },
    CategoryDef { id: "dentist", label: "牙醫師", group: "醫政科" },
    CategoryDef { id: "nurse", label: "護理師(護士)", group: "醫政科" },
    CategoryDef { id: "midwife", label: "助產師(士)", group: "醫政科" },
    CategoryDef { id: "pharmacist", label: "藥師", group: "食藥科" },
    CategoryDef { id: "assistant_pharmacist", label: "藥劑生", group: "食藥科" },
    CategoryDef { id: "medical_technologist", label: "醫事檢驗師(生)", group: "醫政科" },
    CategoryDef { id: "medical_radiation", label: "醫事放射師(士)", group: "醫政科" },
    CategoryDef { id: "dental_technologist", label: "牙體技術師(生)", group: "醫政科" },
    CategoryDef { id: "optometrist", label: "驗光師(生)", group: "醫政科" },
    CategoryDef { id: "physical_therapist", label: "物理治療師(生)", group: "醫政科" },
    CategoryDef { id: "occupational_therapist", label: "職能治療師(生)", group: "醫政科" },
    CategoryDef { id: "speech_therapist", label: "語言治療師", group: "醫政科" },
    CategoryDef { id: "audiologist", label: "聽力師", group: "醫政科" },
    CategoryDef { id: "respiratory_therapist", label: "呼吸治療師", group: "醫政科" },
    CategoryDef { id: "clinical_psychologist", label: "臨床心理師", group: "心健科" },
    CategoryDef { id: "counseling_psychologist", label: "諮商心理師", group: "心健科" },
    CategoryDef { id: "nutritionist", label: "營養師", group: "食藥科" },
];

struct ItemDef {
    id: &'static str,
    label: &'static str,
}

const DEFAULT_ITEMS: &[ItemDef] = &[
    ItemDef { id: "register", label: "執業(現歇業)登記" },
    ItemDef { id: "suspend", label: "停業登記" },
    ItemDef { id: "resume", label: "復業(現停業)登記" },
    ItemDef { id: "cessation", label: "歇業(離職)登記" },
    ItemDef { id: "dept_change", label: "(科別)變更" },
    ItemDef { id: "name_change", label: "(姓名)變更" },
    ItemDef { id: "inst_change", label: "機構變更" },
    ItemDef { id: "cat_change", label: "類別變更" },
    ItemDef { id: "lost_reissue", label: "遺失補發" },
    ItemDef { id: "damage_reissue", label: "損壞補發" },
    ItemDef { id: "renew", label: "到期換發" },
    ItemDef { id: "seniority_cert", label: "年資證明" },
];

/// Runtime 申請類別 — the user can add/edit/delete these (see the editor
/// sheet), so the live list is owned by `App` and persisted under
/// `STORAGE_CATEGORIES`; `DEFAULT_CATEGORIES` only seeds it. `id` is the
/// stable identity (`ITEM_CONDITIONAL_DOC_CODES` and the tour demo key on it),
/// so relabelling a default keeps its document rules.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
struct Category {
    id: String,
    label: String,
    group: String,
}

/// Runtime 申請項目 — same arrangement as `Category`; `ITEM_DOC_CODES` keys on
/// `id`, so a user-added item simply has no item-specific 應備文件.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
struct Item {
    id: String,
    label: String,
}

const STORAGE_CATEGORIES: &str = "medgen_categories";
const STORAGE_ITEMS: &str = "medgen_items";
/// Caps for both the editor and the load-time guard.
const MAX_LIST_LEN: usize = 100;
const MAX_LABEL_CHARS: usize = 30;
const MAX_GROUP_CHARS: usize = 10;

fn default_categories() -> Vec<Category> {
    DEFAULT_CATEGORIES
        .iter()
        .map(|c| Category { id: c.id.to_string(), label: c.label.to_string(), group: c.group.to_string() })
        .collect()
}

fn default_items() -> Vec<Item> {
    DEFAULT_ITEMS
        .iter()
        .map(|i| Item { id: i.id.to_string(), label: i.label.to_string() })
        .collect()
}

/// Loads the saved 申請類別, re-validating like the other stored keys: drops
/// blank/oversized/duplicate-id rows, caps the length, and falls back to the
/// defaults when nothing usable is stored.
fn load_categories() -> Vec<Category> {
    let mut seen = HashSet::new();
    let list: Vec<Category> = LocalStorage::get::<Vec<Category>>(STORAGE_CATEGORIES)
        .unwrap_or_default()
        .into_iter()
        .filter(|c| {
            !c.id.is_empty()
                && c.id.len() <= 64
                && !c.label.trim().is_empty()
                && c.label.chars().count() <= MAX_LABEL_CHARS
                && !c.group.trim().is_empty()
                && c.group.chars().count() <= MAX_GROUP_CHARS
                && seen.insert(c.id.clone())
        })
        .take(MAX_LIST_LEN)
        .collect();
    if list.is_empty() { default_categories() } else { list }
}

fn load_items() -> Vec<Item> {
    let mut seen = HashSet::new();
    let list: Vec<Item> = LocalStorage::get::<Vec<Item>>(STORAGE_ITEMS)
        .unwrap_or_default()
        .into_iter()
        .filter(|i| {
            !i.id.is_empty()
                && i.id.len() <= 64
                && !i.label.trim().is_empty()
                && i.label.chars().count() <= MAX_LABEL_CHARS
                && seen.insert(i.id.clone())
        })
        .take(MAX_LIST_LEN)
        .collect();
    if list.is_empty() { default_items() } else { list }
}

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
    ItemDocs { item_id: "seniority_cert", codes: &[2, 7] },
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

/// Which part of the app a tour step spotlights. `Copy` resolves to a
/// different element per viewport (the desktop button lives in its own
/// result card; the mobile one lives in the sticky bottom bar) — every other
/// variant targets the same card on both.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TourTarget {
    Name,
    Category,
    Items,
    Copy,
    Docs,
}

/// One stop on the guided tour. `target: None` means the step has no
/// spotlight (intro/outro) and renders as a centered panel instead.
struct TutorialStep {
    title: &'static str,
    body: &'static str,
    target: Option<TourTarget>,
}

impl TutorialStep {
    /// `id` of the element that gets the spotlight ring — also the scroll
    /// target and, for every variant but mobile `Copy`, the panel's host.
    fn target_id(&self, is_desktop: bool) -> Option<&'static str> {
        Some(match self.target? {
            TourTarget::Name => "nameCard",
            TourTarget::Category => "categoryCard",
            TourTarget::Items => "itemsCard",
            TourTarget::Copy => if is_desktop { "resultCard" } else { "mobileCopyBtn" },
            TourTarget::Docs => "docsCard",
        })
    }
}

const TUTORIAL_STEPS: &[TutorialStep] = &[
    TutorialStep {
        title: "歡迎使用執業異動文字產生器",
        body: "只要三個步驟——姓名、申請類別、申請項目——就能自動組成公文主旨並複製到剪貼簿。這份導覽只講核心流程，其他功能會在您用到時另外提示一次，隨時可按右上角 ✕ 或 Esc 跳過。",
        target: None,
    },
    TutorialStep {
        title: "STEP 01・輸入姓名",
        body: "在這裡輸入申請人姓名，欄位右側的 ✕ 可一鍵清空。",
        target: Some(TourTarget::Name),
    },
    TutorialStep {
        title: "STEP 02・選擇申請類別",
        body: "上方頁籤可依類別分組篩選，下方清單為單選；切換申請類別會自動清空已選的申請項目。",
        target: Some(TourTarget::Category),
    },
    TutorialStep {
        title: "STEP 03・選擇申請項目",
        body: "申請項目可以複選，能一次勾選多種異動類型，例如同時辦理「機構變更」與「科別變更」。",
        target: Some(TourTarget::Items),
    },
    TutorialStep {
        title: "複製文字",
        body: "三個欄位都填好後會自動複製到剪貼簿；也可以直接按這個按鈕立即複製。",
        target: Some(TourTarget::Copy),
    },
    TutorialStep {
        title: "應備文件檢核表",
        body: "根據所選的申請項目，這裡會自動列出對應的應備文件，可以逐項打勾核對進度。",
        target: Some(TourTarget::Docs),
    },
    TutorialStep {
        title: "核心流程介紹完了",
        body: "複製紀錄、Ctrl+Enter 快捷鍵等功能，會在您第一次用到時另外提示一次；之後想重看導覽，按右上角「教學」按鈕即可。",
        target: None,
    },
];

/// Demo text typed into the name field during STEP 01 of the tour.
const TOUR_DEMO_NAME: &str = "王小明";
/// The two items auto-toggled during STEP 03 of the tour.
const TOUR_DEMO_ITEMS: [&str; 2] = ["register", "dept_change"];
/// The category auto-selected during STEP 02 of the tour.
const TOUR_DEMO_CATEGORY: &str = "physician";

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
    categories: &[Category],
    items: &[Item],
    placeholder_mode: bool,
) -> (String, bool) {
    let name_trimmed = name.trim();
    let display_name = if name_trimmed.is_empty() {
        if placeholder_mode { "（請輸入姓名）" } else { "" }
    } else {
        name_trimmed
    };

    let category_label = category_id
        .as_ref()
        .and_then(|cat_id| categories.iter().find(|c| &c.id == cat_id))
        .map(|cat| clean_parentheses(&cat.label))
        .filter(|s| !s.is_empty());
    let has_category = category_label.is_some();
    let cleaned_category = category_label.unwrap_or_else(|| {
        if placeholder_mode { "（請選擇類別）".to_string() } else { "".to_string() }
    });

    let cleaned_items_list: Vec<String> = selected_items
        .iter()
        .filter_map(|id| items.iter().find(|i| &i.id == id))
        .map(|item| clean_parentheses(&item.label))
        .filter(|s| !s.is_empty())
        .collect();

    let items_text = if cleaned_items_list.is_empty() {
        if placeholder_mode { "（請選擇項目）".to_string() } else { "".to_string() }
    } else {
        cleaned_items_list.join("、")
    };

    // Judged on what actually resolved, not on the raw selections: a label
    // that cleans down to nothing (e.g. a user-entered "(備註)") must not
    // count as a filled field.
    let is_complete = !name_trimmed.is_empty() && has_category && !cleaned_items_list.is_empty();

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
    TutorialStart,
    TutorialNext,
    TutorialPrev,
    TutorialJump(usize),
    TutorialSkip,
    TutorialFinish,
    // `u32` on every demo message is the `demo_gen` it was scheduled under —
    // see `schedule_tour_demo` for why a stale one is a silent no-op.
    DemoTypeName(u32, usize),
    DemoSelectCategory(u32),
    DemoToggleItem(u32, usize),
    DemoPulseCopyOff(u32),
    DemoCheckDoc(u32, usize),
    DismissHistoryCoach,
    DismissShortcutCoach,
    OpenEditor(EditorKind),
    CloseEditor,
    EditorAddRow,
    EditorUpdateLabel(usize, String),
    EditorUpdateGroup(usize, String),
    EditorMoveRow(usize, bool),
    EditorDeleteRow(usize),
    EditorResetDefaults,
    EditorSave,
}

/// Which list the editor sheet is working on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum EditorKind {
    Categories,
    Items,
}

impl EditorKind {
    fn noun(self) -> &'static str {
        match self {
            EditorKind::Categories => "申請類別",
            EditorKind::Items => "申請項目",
        }
    }
}

/// One editable row. `group` is unused for items.
#[derive(Clone, PartialEq)]
struct EditorRow {
    id: String,
    label: String,
    group: String,
}

/// Draft state of the open editor sheet. Edits stay here until 儲存, so
/// 取消 (or Esc / backdrop) throws the whole session away.
struct EditorState {
    kind: EditorKind,
    rows: Vec<EditorRow>,
    error: Option<String>,
    // Suffix for new rows' ids, so two rows added in the same millisecond
    // still get distinct ids.
    next_seq: u32,
}

/// Snapshot of the three form fields (plus the incidental filter/checklist
/// state) taken the moment the tour opens, so a demo that types/selects its
/// way through the fields can be undone verbatim on skip/finish rather than
/// hard-clearing whatever the user had already filled in. Any *real* edit the
/// user makes while the tour is running is written straight into the
/// snapshot too (see the Update* handlers), so a manual edit always survives
/// tour close instead of being reverted along with the demo's.
#[derive(Clone, Default)]
struct FormSnapshot {
    name: String,
    category: Option<String>,
    items: Vec<String>,
    group_tab: String,
    checked_docs: HashSet<u8>,
}

struct App {
    // User-editable option lists (see `Category`/`Item`).
    categories: Vec<Category>,
    items: Vec<Item>,
    editor: Option<EditorState>,
    // Set by EditorAddRow so `rendered` focuses (and scrolls to) the new row.
    editor_focus_last: bool,
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
    // None = tutorial not running. Set to Some(0) on first visit (unless
    // medgen_tutorial_seen is already true) or by the "?" reopen button.
    tutorial_step: Option<usize>,
    tutorial_seen: bool,
    // Which step's target has already been scrolled into view, so `rendered`
    // only scrolls once per step change rather than on every re-render.
    tutorial_scrolled_step: Option<usize>,
    // Taken when the tour opens (`Some`), restored and cleared on skip/finish.
    tour_snapshot: Option<FormSnapshot>,
    // Bumped on every step change and on every real (non-demo) field edit;
    // each scheduled demo message carries the generation it was scheduled
    // under and is a no-op if that no longer matches, which is how a step
    // change or a manual edit cancels in-flight demo timers without needing
    // to track and drop a `Timeout` handle per tick.
    demo_gen: u32,
    demo_copy_pulse: bool,
    // "Seen once" coach marks for the two features the tour no longer walks
    // through step-by-step — shown the first time they'd actually be useful
    // (history has an entry / the form is complete) rather than during the
    // tour, and persisted so dismissing one is permanent.
    history_coach_dismissed: bool,
    shortcut_coach_dismissed: bool,
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

        let tutorial_seen: bool = LocalStorage::get::<bool>("medgen_tutorial_seen").unwrap_or(false);
        let history_coach_dismissed: bool = LocalStorage::get::<bool>("medgen_history_coach_seen").unwrap_or(false);
        let shortcut_coach_dismissed: bool = LocalStorage::get::<bool>("medgen_shortcut_coach_seen").unwrap_or(false);

        // Keydown shortcut setup: Ctrl/Cmd + Enter copies; Escape closes the
        // tour; ←/→/Enter step through it. The closure can't read component
        // state, so the tour keys are sent unconditionally — every Tutorial*
        // handler no-ops (and returns `false`, so no re-render) when no tour
        // is running, which keeps plain Enter and the arrow keys inert during
        // ordinary use of the form.
        let link = ctx.link().clone();
        let keydown_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if (event.ctrl_key() || event.meta_key()) && event.key() == "Enter" {
                event.prevent_default();
                link.send_message(Msg::CopyText);
            } else if event.key() == "Escape" {
                // Both idempotent; at most one of the two is ever open.
                link.send_message(Msg::CloseEditor);
                link.send_message(Msg::TutorialSkip);
            } else {
                match event.key().as_str() {
                    "ArrowRight" | "Enter" => link.send_message(Msg::TutorialNext),
                    "ArrowLeft" => link.send_message(Msg::TutorialPrev),
                    _ => {}
                }
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
            categories: load_categories(),
            items: load_items(),
            editor: None,
            editor_focus_last: false,
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
            tutorial_step: if tutorial_seen { None } else { Some(0) },
            tutorial_seen,
            tutorial_scrolled_step: None,
            // The auto-launched first-visit tour opens over a guaranteed-blank
            // form, so every field here is the blank value — except
            // `group_tab`, whose blank value is "全部", not `String::default()`'s
            // "". Restoring an empty string on `close_tour` matched no tab and
            // no category's `group`, silently emptying STEP 02's grid for
            // every first-time visitor who skipped or finished the tour.
            tour_snapshot: if tutorial_seen {
                None
            } else {
                Some(FormSnapshot {
                    group_tab: "全部".to_string(),
                    ..Default::default()
                })
            },
            demo_gen: 0,
            demo_copy_pulse: false,
            history_coach_dismissed,
            shortcut_coach_dismissed,
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
                self.interrupt_demo();
                self.schedule_auto_copy(ctx);
                true
            }
            Msg::ClearName => {
                self.applicant_name.clear();
                self.interrupt_demo();
                self.schedule_auto_copy(ctx);
                true
            }
            Msg::SelectCategory(cat_id) => {
                self.selected_category = Some(cat_id);
                self.selected_items.clear();
                self.interrupt_demo();
                self.schedule_auto_copy(ctx);
                true
            }
            Msg::ToggleItem(item_id) => {
                if let Some(pos) = self.selected_items.iter().position(|x| x == &item_id) {
                    self.selected_items.remove(pos);
                } else {
                    self.selected_items.push(item_id);
                }
                self.interrupt_demo();
                self.schedule_auto_copy(ctx);
                true
            }
            Msg::SelectTab(tab) => {
                self.selected_group_tab = tab;
                self.interrupt_demo();
                true
            }
            Msg::ToggleDoc(code) => {
                if !self.checked_docs.remove(&code) {
                    self.checked_docs.insert(code);
                }
                self.interrupt_demo();
                true
            }
            Msg::CopyText => {
                // The tour still owns the fields — allowing typing/selecting
                // during it is the point (see the module doc for `FormSnapshot`)
                // — but a real clipboard write of whatever the demo currently
                // has on screen (or of the user's still-mid-edit real data)
                // isn't something either side asked for, so this stays inert
                // for the duration of the tour.
                if self.tutorial_step.is_some() || self.editor.is_some() {
                    return false;
                }
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
                    &self.categories,
                    &self.items,
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
                    &self.categories,
                    &self.items,
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
                self.interrupt_demo();
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
                    self.interrupt_demo();
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
            Msg::TutorialStart => {
                self.tour_snapshot = Some(FormSnapshot {
                    name: self.applicant_name.clone(),
                    category: self.selected_category.clone(),
                    items: self.selected_items.clone(),
                    group_tab: self.selected_group_tab.clone(),
                    checked_docs: self.checked_docs.clone(),
                });
                self.tutorial_step = Some(0);
                self.tutorial_scrolled_step = None;
                self.schedule_tour_demo(ctx, 0);
                true
            }
            Msg::TutorialNext => {
                let Some(cur) = self.tutorial_step else { return false };
                match self.tutorial_next_index(cur) {
                    Some(next) => {
                        self.tutorial_step = Some(next);
                        self.tutorial_scrolled_step = None;
                        self.schedule_tour_demo(ctx, next);
                        true
                    }
                    // Enter/→ on the last step finishes, mirroring the panel's
                    // 完成 button (which sends Msg::TutorialFinish itself, so
                    // this arm is only ever reached from the keyboard).
                    None => {
                        self.finish_tour(ctx);
                        true
                    }
                }
            }
            Msg::TutorialPrev => {
                let Some(cur) = self.tutorial_step else { return false };
                let Some(prev) = self.tutorial_prev_index(cur) else { return false };
                self.tutorial_step = Some(prev);
                self.tutorial_scrolled_step = None;
                self.schedule_tour_demo(ctx, prev);
                true
            }
            Msg::TutorialJump(idx) => {
                if idx < TUTORIAL_STEPS.len() {
                    self.tutorial_step = Some(idx);
                    self.tutorial_scrolled_step = None;
                    self.schedule_tour_demo(ctx, idx);
                }
                true
            }
            Msg::TutorialSkip => {
                // Idempotent on purpose — it's the unconditional target of the
                // global Escape key, whether or not a tour happens to be open.
                self.close_tour(ctx);
                true
            }
            Msg::TutorialFinish => {
                self.finish_tour(ctx);
                true
            }
            Msg::DemoTypeName(gen, chars_typed) => {
                if gen != self.demo_gen {
                    return false;
                }
                self.applicant_name = TOUR_DEMO_NAME.chars().take(chars_typed).collect();
                if chars_typed < TOUR_DEMO_NAME.chars().count() {
                    let link = ctx.link().clone();
                    Timeout::new(200, move || link.send_message(Msg::DemoTypeName(gen, chars_typed + 1))).forget();
                }
                true
            }
            Msg::DemoSelectCategory(gen) => {
                if gen != self.demo_gen {
                    return false;
                }
                // The user may have deleted or never had 醫師 — fall back to
                // whatever comes first so the demo still has something to pick.
                let cat = self
                    .categories
                    .iter()
                    .find(|c| c.id == TOUR_DEMO_CATEGORY)
                    .or_else(|| self.categories.first());
                if let Some(cat) = cat {
                    self.selected_category = Some(cat.id.clone());
                    self.selected_items.clear();
                }
                true
            }
            Msg::DemoToggleItem(gen, which) => {
                if gen != self.demo_gen {
                    return false;
                }
                if let Some(item_id) = self.tour_demo_item_ids().into_iter().nth(which) {
                    if !self.selected_items.contains(&item_id) {
                        self.selected_items.push(item_id);
                    }
                }
                true
            }
            Msg::DemoPulseCopyOff(gen) => {
                if gen != self.demo_gen {
                    return false;
                }
                self.demo_copy_pulse = false;
                true
            }
            Msg::DemoCheckDoc(gen, which) => {
                if gen != self.demo_gen {
                    return false;
                }
                let docs = required_doc_codes(&self.selected_items, &self.selected_category);
                if let Some(&code) = docs.get(which) {
                    self.checked_docs.insert(code);
                }
                true
            }
            Msg::DismissHistoryCoach => {
                self.history_coach_dismissed = true;
                let _ = LocalStorage::set("medgen_history_coach_seen", &true);
                true
            }
            Msg::DismissShortcutCoach => {
                self.shortcut_coach_dismissed = true;
                let _ = LocalStorage::set("medgen_shortcut_coach_seen", &true);
                true
            }
            Msg::OpenEditor(kind) => {
                // The tour owns the screen while it runs (and restores a form
                // snapshot keyed on these lists' ids when it closes).
                if self.tutorial_step.is_some() {
                    return false;
                }
                self.open_editor(kind);
                self.name_suggestions_open = false;
                true
            }
            Msg::CloseEditor => self.editor.take().is_some(),
            Msg::EditorAddRow => {
                let tab = self.selected_group_tab.clone();
                let Some(editor) = self.editor.as_mut() else { return false };
                if editor.rows.len() >= MAX_LIST_LEN {
                    editor.error = Some(format!("最多只能有 {} 個{}", MAX_LIST_LEN, editor.kind.noun()));
                    return true;
                }
                // New categories land in the tab the user is looking at, or
                // in the last row's group — the likeliest intent either way.
                let group = if editor.kind == EditorKind::Categories {
                    if tab != "全部" {
                        tab
                    } else {
                        editor.rows.last().map(|r| r.group.clone()).unwrap_or_else(|| "醫政科".to_string())
                    }
                } else {
                    String::new()
                };
                editor.next_seq += 1;
                editor.rows.push(EditorRow {
                    id: format!("custom_{}_{}", js_sys::Date::now() as u64, editor.next_seq),
                    label: String::new(),
                    group,
                });
                editor.error = None;
                self.editor_focus_last = true;
                true
            }
            Msg::EditorUpdateLabel(idx, value) => {
                let Some(editor) = self.editor.as_mut() else { return false };
                if let Some(row) = editor.rows.get_mut(idx) {
                    row.label = value;
                }
                editor.error = None;
                true
            }
            Msg::EditorUpdateGroup(idx, value) => {
                let Some(editor) = self.editor.as_mut() else { return false };
                if let Some(row) = editor.rows.get_mut(idx) {
                    row.group = value;
                }
                editor.error = None;
                true
            }
            Msg::EditorMoveRow(idx, up) => {
                let Some(editor) = self.editor.as_mut() else { return false };
                let target = if up { idx.checked_sub(1) } else { Some(idx + 1) };
                match target {
                    Some(t) if t < editor.rows.len() && idx < editor.rows.len() => {
                        editor.rows.swap(idx, t);
                        true
                    }
                    _ => false,
                }
            }
            Msg::EditorDeleteRow(idx) => {
                let Some(editor) = self.editor.as_mut() else { return false };
                if idx < editor.rows.len() {
                    editor.rows.remove(idx);
                }
                editor.error = None;
                true
            }
            Msg::EditorResetDefaults => {
                let Some(editor) = self.editor.as_mut() else { return false };
                editor.rows = match editor.kind {
                    EditorKind::Categories => default_categories()
                        .into_iter()
                        .map(|c| EditorRow { id: c.id, label: c.label, group: c.group })
                        .collect(),
                    EditorKind::Items => default_items()
                        .into_iter()
                        .map(|i| EditorRow { id: i.id, label: i.label, group: String::new() })
                        .collect(),
                };
                editor.error = None;
                true
            }
            Msg::EditorSave => {
                match self.save_editor(ctx) {
                    Ok(kind) => {
                        self.toast = Some(ToastState {
                            message: format!("已儲存{}", kind.noun()),
                            is_error: false,
                        });
                        self.schedule_toast_clear(ctx);
                    }
                    Err(message) => {
                        if let Some(editor) = self.editor.as_mut() {
                            editor.error = Some(message);
                        }
                    }
                }
                true
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if std::mem::take(&mut self.editor_focus_last) {
            if let Some(input) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.query_selector(".editor-row:last-child .editor-input").ok().flatten())
                .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                // Focusing scrolls the overflowed list to the new row too.
                let _ = input.focus();
            }
        }

        if let Some(idx) = self.tutorial_step {
            if self.tutorial_scrolled_step != Some(idx) {
                self.tutorial_scrolled_step = Some(idx);
                let target_id = TUTORIAL_STEPS
                    .get(idx)
                    .and_then(|step| step.target_id(is_desktop_viewport()));
                if let Some(id) = target_id {
                    if let Some(el) = web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.get_element_by_id(id))
                    {
                        let opts = web_sys::ScrollIntoViewOptions::new();
                        opts.set_behavior(web_sys::ScrollBehavior::Smooth);
                        // Centered rather than Start: the panel now renders as
                        // part of the target (nested below/above it), so it
                        // extends past the target's own box and Start could
                        // leave it hanging off the bottom of the viewport.
                        opts.set_block(web_sys::ScrollLogicalPosition::Center);
                        el.scroll_into_view_with_scroll_into_view_options(&opts);
                    }
                }
            }
        }

        if !first_render && !self.focus_name_pending {
            return;
        }
        self.focus_name_pending = false;

        // Fields stay interactive during the tour, but grabbing real keyboard
        // focus out from under whatever the user is looking at just because
        // this happens to be the first render (the tour can auto-launch on
        // that very frame) would still be rude — the demo drives the name
        // field's value directly without needing focus.
        if self.tutorial_step.is_some() || !is_desktop_viewport() {
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
            &self.categories,
            &self.items,
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

        // Tab list is derived from the distinct `group` values in the live
        // (user-editable) category list
        let mut group_tabs = vec!["全部"];
        for cat in &self.categories {
            if !group_tabs.contains(&cat.group.as_str()) {
                group_tabs.push(cat.group.as_str());
            }
        }

        let filtered_categories: Vec<&Category> = self
            .categories
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

        // ─── Tutorial spotlight ───
        // The current step's semantic target (if any) drives the
        // `.tutorial-target` ring on whichever card it names, and the nested
        // panel is rendered as that same card's last child (see the
        // `.tutorial-target` CSS comment for why that makes `position:
        // absolute` work with no JS measurement). `mobile_bar_lifted` counters
        // `.mobile-bar` being its own stacking context (`position: fixed` +
        // `z-index`), so its spotlighted button's raised z-index would
        // otherwise be capped within it.
        let is_desktop = is_desktop_viewport();
        let tour_idx = self.tutorial_step;
        let tour_kind: Option<TourTarget> = tour_idx.and_then(|i| TUTORIAL_STEPS.get(i)).and_then(|s| s.target);
        let is_name_target = tour_kind == Some(TourTarget::Name);
        let is_category_target = tour_kind == Some(TourTarget::Category);
        let is_items_target = tour_kind == Some(TourTarget::Items);
        let is_copy_target = tour_kind == Some(TourTarget::Copy);
        let is_docs_target = tour_kind == Some(TourTarget::Docs);
        let mobile_bar_lifted = is_copy_target && !is_desktop;

        let show_history_coach = !self.copy_history.is_empty() && tour_idx.is_none() && !self.history_coach_dismissed;
        let show_shortcut_coach = is_complete && tour_idx.is_none() && !self.shortcut_coach_dismissed;

        html! {
            <div class="app-shell">
                // Decorative color field the glass surfaces blur (see
                // `.app-bg` in style.css).
                <div class="app-bg" aria-hidden="true">
                    <div class="app-bg-blob app-bg-blob-a"></div>
                    <div class="app-bg-blob app-bg-blob-b"></div>
                    <div class="app-bg-blob app-bg-blob-c"></div>
                </div>
                // ─── Header ───
                <header class="nav">
                    <span class="nav-brand">{"醫事人員執業異動文字產生器"}</span>
                    <div class="nav-actions">
                        {if self.deferred_prompt.is_some() {
                            html! {
                                <button
                                    id="installBtn"
                                    class="btn btn-glass nav-pill-btn"
                                    onclick={ctx.link().callback(|_| Msg::TriggerInstall)}
                                    aria-label="安裝應用"
                                >
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
                        // The two record-keeping actions share one
                        // segmented glass capsule split by a hairline (see
                        // `.nav-group`) rather than reading as two unrelated
                        // pills; the capsule owns the surface, its members
                        // are transparent.
                        <div class="nav-group">
                            <div class="coach-anchor">
                                <button
                                    id="historyBtn"
                                    class="btn btn-icon nav-icon-btn history-btn"
                                    onclick={ctx.link().callback(|_| Msg::ToggleHistory)}
                                    aria-label="複製紀錄"
                                >
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
                                {if show_history_coach {
                                    html! {
                                        <div class="coach-mark coach-mark-history">
                                            <p class="coach-mark-text">{"點這裡可以看最近複製過的紀錄，點一下就能重新複製。"}</p>
                                            <button type="button" class="coach-mark-dismiss" onclick={ctx.link().callback(|_| Msg::DismissHistoryCoach)}>{"知道了"}</button>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                            <div class="nav-group-divider"></div>
                            <button
                                id="resetBtn"
                                class="btn btn-icon nav-icon-btn nav-icon-btn-danger"
                                onclick={ctx.link().callback(|_| Msg::ResetAll)}
                                aria-label="清除重填"
                            >
                                <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M3 12a9 9 0 019-9 9.75 9.75 0 016.74 2.74L21 8"/>
                                    <path d="M21 3v5h-5"/>
                                    <path d="M21 12a9 9 0 01-9 9 9.75 9.75 0 01-6.74-2.74L3 16"/>
                                    <path d="M3 21v-5h5"/>
                                </svg>
                            </button>
                        </div>
                        // Labelled rather than icon-only: the tour is opt-in
                        // after the first visit, and a bare "?" glyph reads as
                        // generic help. The label collapses back to an icon
                        // below 800px, where the nav has no room for it.
                        <button
                            id="tutorialReopenBtn"
                            class="btn btn-glass nav-pill-btn nav-tutorial-btn"
                            onclick={ctx.link().callback(|_| Msg::TutorialStart)}
                            title="重新查看操作教學"
                            aria-label="操作教學"
                        >
                            <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="12" cy="12" r="10"/>
                                <path d="M9.09 9a3 3 0 015.83 1c0 2-3 2-3 4"/>
                                <line x1="12" y1="17" x2="12.01" y2="17"/>
                            </svg>
                            <span class="nav-btn-label">{"教學"}</span>
                        </button>
                    </div>
                </header>

                <div class="app-layout">
                    // ─── Input column ───
                    <div class="col-input">

                        // STEP 01 — name
                        <section id="nameCard" class={classes!("card", "anim-in", is_name_target.then_some("tutorial-target"))} aria-label="輸入申請人姓名">
                            <div class="card-head">
                                <div>
                                    <div class="card-kicker">{"STEP 01"}</div>
                                    <div class="card-title">{"申請人姓名"}</div>
                                </div>
                                <span class={name_tag_class}>{name_tag_label}</span>
                            </div>
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
                            {if is_name_target { self.view_tour_panel(ctx, tour_idx.unwrap(), "tutorial-panel-below") } else { html! {} }}
                        </section>

                        // STEP 02 — category
                        <section id="categoryCard" class={classes!("card", "anim-in", "anim-in-2", is_category_target.then_some("tutorial-target"))} aria-label="選擇申請類別">
                            <div class="card-head">
                                <div>
                                    <div class="card-kicker">{"STEP 02"}</div>
                                    <div class="card-title">{"申請類別"}</div>
                                </div>
                                <div class="tag-group">
                                    {self.view_edit_btn(ctx, EditorKind::Categories)}
                                    <span class="tag tag-outline tag-mode">{"單選"}</span>
                                    <span class={cat_tag_class}>{cat_tag_label}</span>
                                </div>
                            </div>
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
                                    let cat_id = cat.id.clone();
                                    let selected = self.selected_category.as_ref() == Some(&cat_id);
                                    html! {
                                        <button
                                            type="button"
                                            class={if selected { "btn opt-btn selected" } else { "btn opt-btn" }}
                                            aria-pressed={if selected { "true" } else { "false" }}
                                            onclick={ctx.link().callback(move |_| Msg::SelectCategory(cat_id.clone()))}
                                        >
                                            {cat.label.clone()}
                                        </button>
                                    }
                                })}
                            </div>
                            {if is_category_target { self.view_tour_panel(ctx, tour_idx.unwrap(), "tutorial-panel-below") } else { html! {} }}
                        </section>

                        // STEP 03 — items
                        <section id="itemsCard" class={classes!("card", "anim-in", "anim-in-3", is_items_target.then_some("tutorial-target"))} aria-label="選擇申請項目">
                            <div class="card-head">
                                <div>
                                    <div class="card-kicker">{"STEP 03"}</div>
                                    <div class="card-title">{"申請項目"}</div>
                                </div>
                                <div class="tag-group">
                                    {self.view_edit_btn(ctx, EditorKind::Items)}
                                    <span class="tag tag-outline tag-mode">{"複選"}</span>
                                    <span class={items_tag_class}>{items_tag_label}</span>
                                </div>
                            </div>
                            <div class="items-grid">
                                {for self.items.iter().map(|item| {
                                    let item_id = item.id.clone();
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
                                            {item.label.clone()}
                                        </button>
                                    }
                                })}
                            </div>
                            {if is_items_target { self.view_tour_panel(ctx, tour_idx.unwrap(), "tutorial-panel-above") } else { html! {} }}
                        </section>
                    </div>

                    // ─── Preview column ───
                    <div class={classes!("col-preview", "anim-in", "anim-in-4", ((is_copy_target && is_desktop) || is_docs_target).then_some("tutorial-lift"))}>

                        <section id="resultCard" class={classes!("card", (is_copy_target && is_desktop).then_some("tutorial-target"))}>
                            <div class="card-head card-head-center">
                                <div class="card-title">{"產生結果"}</div>
                                <span class={if is_complete { "tag tag-accent" } else { "tag tag-neutral" }}>
                                    {if is_complete { "可複製" } else { "未完成" }}
                                </span>
                            </div>
                            <div class="preview-box">
                                <p id="outputResult" class={classes!("preview-text", (!show_result).then_some("placeholder"))}>
                                    {desktop_preview_text}
                                </p>
                            </div>
                            <div class="copy-btn-anchor">
                                {if self.demo_copy_pulse { html! { <div class="copy-pulse-ring"></div> } } else { html! {} }}
                                <button
                                    class={classes!("btn", "btn-block", "btn-copy", self.copied_morph.then_some("copied"))}
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
                            </div>
                            <div class="coach-anchor">
                                <div id="shortcutHint" class="shortcut-hint">
                                    <span class="key-cap">{"Ctrl"}</span>
                                    <span>{"+"}</span>
                                    <span class="key-cap">{"Enter"}</span>
                                    <span>{"快速複製"}</span>
                                </div>
                                {if show_shortcut_coach {
                                    html! {
                                        <div class="coach-mark coach-mark-shortcut">
                                            <p class="coach-mark-text">{"填完三個欄位後，直接按 Ctrl（Mac 為 ⌘）+ Enter 也能快速複製。"}</p>
                                            <button type="button" class="coach-mark-dismiss" onclick={ctx.link().callback(|_| Msg::DismissShortcutCoach)}>{"知道了"}</button>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                            {if is_copy_target && is_desktop { self.view_tour_panel(ctx, tour_idx.unwrap(), "tutorial-panel-below") } else { html! {} }}
                        </section>

                        // 應備文件檢核表 — derived from the selected 申請項目
                        <section id="docsCard" class={classes!("card", "docs-card", is_docs_target.then_some("tutorial-target"))} aria-label="應備文件檢核表">
                            <div class="card-head card-head-center">
                                <div class="card-title">{"應備文件檢核表"}</div>
                                {if doc_codes.is_empty() {
                                    html! {}
                                } else {
                                    html! {
                                        <span class="docs-count">
                                            {format!("{} / {}", docs_checked_count, doc_codes.len())}
                                        </span>
                                    }
                                }}
                            </div>
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
                            {if is_docs_target { self.view_tour_panel(ctx, tour_idx.unwrap(), "tutorial-panel-above") } else { html! {} }}
                        </section>

                        <section
                            id="historyCard"
                            class={classes!("card", "history-card", self.history_open.then_some("history-open-mobile"))}
                        >
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
                <div class={classes!("mobile-bar", mobile_bar_lifted.then_some("tutorial-lift"))}>
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
                        class={classes!("btn", "btn-copy", "mobile-copy-btn", self.copied_morph.then_some("copied"), (is_copy_target && !is_desktop).then_some("tutorial-target"))}
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
                    {if is_copy_target && !is_desktop { self.view_tour_panel(ctx, tour_idx.unwrap(), "tutorial-panel-above tutorial-panel-mobile-copy") } else { html! {} }}
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

                // ─── Tutorial overlay ───
                // Per-target panels are rendered inline next to their card
                // above; only the no-target (welcome/finish) steps use this
                // fixed, viewport-centered one.
                {if tour_idx.is_some() {
                    html! { <div class="tutorial-backdrop" onclick={ctx.link().callback(|_| Msg::TutorialSkip)}></div> }
                } else {
                    html! {}
                }}
                {if tour_kind.is_none() {
                    if let Some(idx) = tour_idx {
                        self.view_tour_panel(ctx, idx, "tutorial-panel-centered")
                    } else {
                        html! {}
                    }
                } else {
                    html! {}
                }}

                // ─── Option editor sheet ───
                {self.view_editor(ctx)}
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
            &self.categories,
            &self.items,
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

    /// The two items STEP 03's demo ticks: `TOUR_DEMO_ITEMS` where they still
    /// exist, topped up from the front of the (user-editable) list otherwise.
    fn tour_demo_item_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = TOUR_DEMO_ITEMS
            .iter()
            .filter(|id| self.items.iter().any(|i| i.id == **id))
            .map(|id| id.to_string())
            .collect();
        for item in &self.items {
            if ids.len() >= TOUR_DEMO_ITEMS.len() {
                break;
            }
            if !ids.contains(&item.id) {
                ids.push(item.id.clone());
            }
        }
        ids
    }

    /// Opens the editor sheet with a draft copy of the chosen list.
    fn open_editor(&mut self, kind: EditorKind) {
        let rows = match kind {
            EditorKind::Categories => self
                .categories
                .iter()
                .map(|c| EditorRow { id: c.id.clone(), label: c.label.clone(), group: c.group.clone() })
                .collect(),
            EditorKind::Items => self
                .items
                .iter()
                .map(|i| EditorRow { id: i.id.clone(), label: i.label.clone(), group: String::new() })
                .collect(),
        };
        self.editor = Some(EditorState { kind, rows, error: None, next_seq: 0 });
    }

    /// Validates the draft and, if it passes, makes it the live list:
    /// persists it (or clears the key when it matches the defaults, so later
    /// changes to the built-in tables still reach this browser), then drops
    /// any selection that pointed at a deleted row. Returns the error message
    /// on failure, leaving everything untouched.
    fn save_editor(&mut self, ctx: &Context<Self>) -> Result<EditorKind, String> {
        let Some(editor) = self.editor.as_ref() else { return Err(String::new()) };
        let kind = editor.kind;
        let noun = kind.noun();

        if editor.rows.is_empty() {
            return Err(format!("至少需要保留一個{}", noun));
        }
        let mut labels: HashSet<String> = HashSet::new();
        let mut rows: Vec<EditorRow> = Vec::with_capacity(editor.rows.len());
        for (n, row) in editor.rows.iter().enumerate() {
            let label = row.label.trim().to_string();
            let group = row.group.trim().to_string();
            if label.is_empty() {
                return Err(format!("第 {} 列的名稱不可空白", n + 1));
            }
            if label.chars().count() > MAX_LABEL_CHARS {
                return Err(format!("「{}」超過 {} 個字", label, MAX_LABEL_CHARS));
            }
            if clean_parentheses(&label).is_empty() {
                return Err(format!("「{}」去掉括號後沒有文字，無法產生主旨", label));
            }
            if !labels.insert(label.clone()) {
                return Err(format!("「{}」重複了", label));
            }
            if kind == EditorKind::Categories {
                if group.is_empty() {
                    return Err(format!("「{}」需要填寫分組", label));
                }
                if group.chars().count() > MAX_GROUP_CHARS || group == "全部" {
                    return Err(format!("「{}」的分組名稱無效（最多 {} 字，且不可為「全部」）", label, MAX_GROUP_CHARS));
                }
            }
            rows.push(EditorRow { id: row.id.clone(), label, group });
        }

        match kind {
            EditorKind::Categories => {
                self.categories = rows
                    .into_iter()
                    .map(|r| Category { id: r.id, label: r.label, group: r.group })
                    .collect();
                if self.categories == default_categories() {
                    LocalStorage::delete(STORAGE_CATEGORIES);
                } else {
                    let _ = LocalStorage::set(STORAGE_CATEGORIES, &self.categories);
                }
                if let Some(cat_id) = &self.selected_category {
                    if !self.categories.iter().any(|c| &c.id == cat_id) {
                        self.selected_category = None;
                    }
                }
                if self.selected_group_tab != "全部"
                    && !self.categories.iter().any(|c| c.group == self.selected_group_tab)
                {
                    self.selected_group_tab = "全部".to_string();
                }
            }
            EditorKind::Items => {
                self.items = rows
                    .into_iter()
                    .map(|r| Item { id: r.id, label: r.label })
                    .collect();
                if self.items == default_items() {
                    LocalStorage::delete(STORAGE_ITEMS);
                } else {
                    let _ = LocalStorage::set(STORAGE_ITEMS, &self.items);
                }
                let items = &self.items;
                self.selected_items.retain(|id| items.iter().any(|i| &i.id == id));
            }
        }

        self.editor = None;
        // A relabel changes the generated text even with the same selections.
        self.schedule_auto_copy(ctx);
        Ok(kind)
    }

    fn tutorial_next_index(&self, from: usize) -> Option<usize> {
        (from + 1 < TUTORIAL_STEPS.len()).then_some(from + 1)
    }

    fn tutorial_prev_index(&self, from: usize) -> Option<usize> {
        (from > 0).then_some(from - 1)
    }

    /// Cancels whatever demo tick is in flight (by invalidating `demo_gen`,
    /// see the enum's doc comment on the `Demo*` messages) and, if the tour
    /// is open, folds the just-made real edit into `tour_snapshot` so it
    /// survives `close_tour`'s restore instead of being reverted along with
    /// whatever the demo was doing. Called from every handler that lets the
    /// user genuinely edit a field, tour or not — a no-op outside the tour
    /// since there's nothing scheduled and no snapshot to update.
    fn interrupt_demo(&mut self) {
        self.demo_gen = self.demo_gen.wrapping_add(1);
        self.demo_copy_pulse = false;
        if let Some(snap) = self.tour_snapshot.as_mut() {
            snap.name = self.applicant_name.clone();
            snap.category = self.selected_category.clone();
            snap.items = self.selected_items.clone();
            snap.group_tab = self.selected_group_tab.clone();
            snap.checked_docs = self.checked_docs.clone();
        }
    }

    /// Ends the tour (skip or finish) and restores whatever was in the three
    /// fields before it opened — see the `FormSnapshot` doc comment for why
    /// that's preferable to the imported design's own hard-clear-to-blank.
    fn close_tour(&mut self, ctx: &Context<Self>) {
        self.demo_gen = self.demo_gen.wrapping_add(1);
        self.demo_copy_pulse = false;
        self.tutorial_step = None;
        self.tutorial_seen = true;
        let _ = LocalStorage::set("medgen_tutorial_seen", &true);
        if let Some(snap) = self.tour_snapshot.take() {
            self.applicant_name = snap.name;
            self.selected_category = snap.category;
            self.selected_items = snap.items;
            self.selected_group_tab = snap.group_tab;
            self.checked_docs = snap.checked_docs;
        }
        self.schedule_auto_copy(ctx);
    }

    /// `close_tour` plus the sign-off toast — the "reached the end" ending, as
    /// opposed to the silent one `Msg::TutorialSkip` takes. Shared by the
    /// panel's 完成 button and by Enter/→ on the last step.
    fn finish_tour(&mut self, ctx: &Context<Self>) {
        self.close_tour(ctx);
        self.toast = Some(ToastState {
            message: "教學結束，祝使用順利！".to_string(),
            is_error: false,
        });
        self.schedule_toast_clear(ctx);
    }

    /// Kicks off the new step's "watch it happen" demo. Bumps `demo_gen`
    /// first (invalidating any tick still scheduled from the previous step),
    /// then, for steps that have one, schedules the demo under the new
    /// generation — each fired `Demo*` message re-checks the generation it
    /// was scheduled under and silently no-ops if a later step change or a
    /// real user edit has since moved on, which is what lets a fire-and-forget
    /// `Timeout` (see `gloo_timers`) stand in for a cancellable one here.
    fn schedule_tour_demo(&mut self, ctx: &Context<Self>, idx: usize) {
        self.demo_gen = self.demo_gen.wrapping_add(1);
        self.demo_copy_pulse = false;
        let gen = self.demo_gen;
        let Some(step) = TUTORIAL_STEPS.get(idx) else { return };
        let link = ctx.link().clone();
        match step.target {
            None => {}
            Some(TourTarget::Name) => {
                self.applicant_name.clear();
                Timeout::new(400, move || link.send_message(Msg::DemoTypeName(gen, 1))).forget();
            }
            Some(TourTarget::Category) => {
                Timeout::new(500, move || link.send_message(Msg::DemoSelectCategory(gen))).forget();
            }
            Some(TourTarget::Items) => {
                let link2 = link.clone();
                Timeout::new(500, move || link2.send_message(Msg::DemoToggleItem(gen, 0))).forget();
                Timeout::new(1200, move || link.send_message(Msg::DemoToggleItem(gen, 1))).forget();
            }
            Some(TourTarget::Copy) => {
                self.demo_copy_pulse = true;
                Timeout::new(1800, move || link.send_message(Msg::DemoPulseCopyOff(gen))).forget();
            }
            Some(TourTarget::Docs) => {
                let link2 = link.clone();
                Timeout::new(500, move || link2.send_message(Msg::DemoCheckDoc(gen, 0))).forget();
                Timeout::new(1200, move || link.send_message(Msg::DemoCheckDoc(gen, 1))).forget();
            }
        }
    }

    /// The small pencil capsule in STEP 02/03's head that opens the editor.
    /// Disabled while the tour runs (`Msg::OpenEditor` is inert then anyway).
    fn view_edit_btn(&self, ctx: &Context<Self>, kind: EditorKind) -> Html {
        let label = format!("編輯{}", kind.noun());
        html! {
            <button
                type="button"
                class="edit-btn"
                title={label.clone()}
                aria-label={label}
                disabled={self.tutorial_step.is_some()}
                onclick={ctx.link().callback(move |_| Msg::OpenEditor(kind))}
            >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M12 20h9"/>
                    <path d="M16.5 3.5a2.12 2.12 0 013 3L7 19l-4 1 1-4z"/>
                </svg>
                {"編輯"}
            </button>
        }
    }

    /// The add/edit/delete/reorder sheet for whichever list `self.editor`
    /// holds a draft of. Rendered at the app-shell level (like the centered
    /// tour panel) so its backdrop-filter has the page to sample.
    fn view_editor(&self, ctx: &Context<Self>) -> Html {
        let Some(editor) = self.editor.as_ref() else { return html! {} };
        let kind = editor.kind;
        let noun = kind.noun();
        let is_categories = kind == EditorKind::Categories;
        let last = editor.rows.len().saturating_sub(1);

        let mut groups: Vec<&str> = Vec::new();
        for row in &editor.rows {
            let g = row.group.trim();
            if !g.is_empty() && !groups.contains(&g) {
                groups.push(g);
            }
        }

        html! {
            <>
                <div class="editor-backdrop" onclick={ctx.link().callback(|_| Msg::CloseEditor)}></div>
                <div class="editor-sheet" role="dialog" aria-modal="true" aria-label={format!("編輯{}", noun)}>
                    <div class="editor-head">
                        <div>
                            <div class="card-kicker">{"自訂選項"}</div>
                            <div class="card-title">{format!("編輯{}", noun)}</div>
                        </div>
                        <button
                            type="button"
                            class="tutorial-close"
                            onclick={ctx.link().callback(|_| Msg::CloseEditor)}
                            aria-label="關閉"
                        >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round">
                                <path d="M18 6L6 18M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>
                    <p class="editor-hint">
                        {"按「儲存」後生效，只保存在這個瀏覽器。括號內的文字不會出現在產生結果，例如「護理師(護士)」會產生「護理師」。"}
                        {if is_categories { "分組即上方的篩選頁籤。" } else { "自行新增的項目不會自動列出應備文件。" }}
                    </p>
                    <div class="editor-list">
                        {for editor.rows.iter().enumerate().map(|(idx, row)| {
                            html! {
                                <div class="editor-row" key={row.id.clone()}>
                                    <div class="editor-fields">
                                        <input
                                            type="text"
                                            class="input editor-input"
                                            placeholder={format!("{}名稱", noun)}
                                            aria-label={format!("第 {} 列名稱", idx + 1)}
                                            maxlength={MAX_LABEL_CHARS.to_string()}
                                            value={row.label.clone()}
                                            oninput={ctx.link().callback(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                Msg::EditorUpdateLabel(idx, input.value())
                                            })}
                                        />
                                        {if is_categories {
                                            html! {
                                                <input
                                                    type="text"
                                                    class="input editor-input editor-group-input"
                                                    list="editorGroupOptions"
                                                    placeholder="分組"
                                                    aria-label={format!("第 {} 列分組", idx + 1)}
                                                    maxlength={MAX_GROUP_CHARS.to_string()}
                                                    value={row.group.clone()}
                                                    oninput={ctx.link().callback(move |e: InputEvent| {
                                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                        Msg::EditorUpdateGroup(idx, input.value())
                                                    })}
                                                />
                                            }
                                        } else {
                                            html! {}
                                        }}
                                    </div>
                                    <div class="editor-row-actions">
                                        <button
                                            type="button"
                                            class="editor-icon-btn"
                                            aria-label="上移"
                                            disabled={idx == 0}
                                            onclick={ctx.link().callback(move |_| Msg::EditorMoveRow(idx, true))}
                                        >
                                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><polyline points="18 15 12 9 6 15"/></svg>
                                        </button>
                                        <button
                                            type="button"
                                            class="editor-icon-btn"
                                            aria-label="下移"
                                            disabled={idx == last}
                                            onclick={ctx.link().callback(move |_| Msg::EditorMoveRow(idx, false))}
                                        >
                                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
                                        </button>
                                        <button
                                            type="button"
                                            class="editor-icon-btn editor-icon-btn-danger"
                                            aria-label="刪除"
                                            onclick={ctx.link().callback(move |_| Msg::EditorDeleteRow(idx))}
                                        >
                                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                                <polyline points="3 6 5 6 21 6"/>
                                                <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6"/>
                                                <path d="M10 11v6M14 11v6"/>
                                                <path d="M9 6V4a1 1 0 011-1h4a1 1 0 011 1v2"/>
                                            </svg>
                                        </button>
                                    </div>
                                </div>
                            }
                        })}
                        {if editor.rows.is_empty() {
                            html! { <div class="docs-empty">{format!("目前沒有{}", noun)}</div> }
                        } else {
                            html! {}
                        }}
                    </div>
                    {if is_categories {
                        html! {
                            <datalist id="editorGroupOptions">
                                {for groups.iter().map(|g| html! { <option value={g.to_string()} /> })}
                            </datalist>
                        }
                    } else {
                        html! {}
                    }}
                    <button type="button" class="btn btn-secondary editor-add" onclick={ctx.link().callback(|_| Msg::EditorAddRow)}>
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round"><path d="M12 5v14M5 12h14"/></svg>
                        {format!("新增{}", noun)}
                    </button>
                    {if let Some(err) = &editor.error {
                        html! { <p class="editor-error" role="alert">{err}</p> }
                    } else {
                        html! {}
                    }}
                    <div class="editor-foot">
                        <button type="button" class="btn editor-reset" onclick={ctx.link().callback(|_| Msg::EditorResetDefaults)}>
                            {"恢復預設"}
                        </button>
                        <div class="editor-foot-spacer"></div>
                        <button type="button" class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::CloseEditor)}>
                            {"取消"}
                        </button>
                        <button type="button" class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::EditorSave)}>
                            {"儲存"}
                        </button>
                    </div>
                </div>
            </>
        }
    }

    /// Renders one tour panel: the shared card (progress, close, title, body,
    /// step dots, prev/next-or-finish) wrapped in `placement_class`, which
    /// picks its position — `tutorial-panel-centered` (fixed, viewport-
    /// centered, used for the no-target intro/outro steps) or
    /// `tutorial-panel-below`/`-above` (absolute, nested inside the
    /// spotlighted card itself so it needs no JS positioning at all).
    fn view_tour_panel(&self, ctx: &Context<Self>, idx: usize, placement_class: &'static str) -> Html {
        let Some(step) = TUTORIAL_STEPS.get(idx) else { return html! {} };
        let total = TUTORIAL_STEPS.len();
        let is_first = idx == 0;
        let is_last = idx == total - 1;
        let next_label = if is_first { "開始導覽" } else { "下一步" };

        html! {
            <div class={classes!("tutorial-panel", placement_class)} role="dialog" aria-modal="true" aria-label="操作教學">
                <div class="tutorial-panel-head">
                    <span class="tutorial-progress">{format!("{} / {}", idx + 1, total)}</span>
                    <button
                        type="button"
                        class="tutorial-close"
                        onclick={ctx.link().callback(|_| Msg::TutorialSkip)}
                        aria-label="跳過教學"
                    >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round">
                            <path d="M18 6L6 18M6 6l12 12"/>
                        </svg>
                    </button>
                </div>
                <div class="tutorial-title">{step.title}</div>
                <p class="tutorial-body">{step.body}</p>
                {if step.target.is_some() {
                    html! { <p class="tutorial-note">{"畫面正在自動示範這個步驟，操作結束後可以自行修改。"}</p> }
                } else {
                    html! {}
                }}
                <div class="tutorial-dots">
                    {for (0..total).map(|i| {
                        let active = i == idx;
                        html! {
                            <button
                                type="button"
                                class={if active { "tutorial-dot active" } else { "tutorial-dot" }}
                                aria-label="跳到步驟"
                                onclick={ctx.link().callback(move |_| Msg::TutorialJump(i))}
                            ></button>
                        }
                    })}
                </div>
                <div class="tutorial-controls">
                    {if is_first {
                        html! { <div class="tutorial-controls-spacer"></div> }
                    } else {
                        html! {
                            <button type="button" class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::TutorialPrev)}>
                                {"上一步"}
                            </button>
                        }
                    }}
                    {if is_last {
                        html! {
                            <button type="button" class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::TutorialFinish)}>
                                {"完成"}
                            </button>
                        }
                    } else {
                        html! {
                            <button type="button" class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::TutorialNext)}>
                                {next_label}
                            </button>
                        }
                    }}
                </div>
                // Hidden below 800px in style.css — a phone has no arrow keys.
                <div class="tutorial-keyhint">{"← → 鍵切換・Enter 下一步・Esc 跳過"}</div>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
