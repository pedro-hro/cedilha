//! Rust backend for Cedilha — file I/O, dialogs, and window helpers.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::SystemTime;

use tauri::window::Window;
use tauri::Manager;

/// Shared application state threaded through the command handlers.
pub struct AppState {
    /// The file currently open in the editor, if any.
    pub current_path: Mutex<Option<PathBuf>>,
    /// Persisted user preferences (theme, font size, ...).
    pub prefs: Mutex<crate::config::Prefs>,
    /// Files the user picked through a native dialog. The read/write commands refuse
    /// every other path, so script running in the webview can't touch arbitrary files.
    pub allowed: Mutex<HashSet<PathBuf>>,
    /// Modification time of the open file when we last read or wrote it; used to
    /// detect edits made by other programs before autosaving over them.
    pub disk_mtime: Mutex<Option<SystemTime>>,
    /// A window close is waiting on the frontend's unsaved-changes check.
    pub close_requested: AtomicBool,
    /// File named on the command line (`cedilha notes.md`), handed to the frontend once.
    pub initial_file: Mutex<Option<PathBuf>>,
}

impl AppState {
    pub fn new(prefs: crate::config::Prefs) -> Self {
        AppState {
            current_path: Mutex::new(None),
            prefs: Mutex::new(prefs),
            allowed: Mutex::new(HashSet::new()),
            disk_mtime: Mutex::new(None),
            close_requested: AtomicBool::new(false),
            initial_file: Mutex::new(None),
        }
    }

    fn allow(&self, path: &Path) {
        self.allowed.lock().unwrap().insert(canonical(path));
    }

    fn is_allowed(&self, path: &Path) -> bool {
        self.allowed.lock().unwrap().contains(&canonical(path))
    }

    fn remember_mtime(&self, path: &Path) {
        *self.disk_mtime.lock().unwrap() = mtime_of(path);
    }
}

/// Let the preview show images that live next to a document (and in folders below it).
/// The scope is enforced here, in Rust: nothing outside these folders can be shown, whatever
/// a Markdown file asks for. Folders opened earlier in the session stay allowed.
fn allow_document_images(app: &tauri::AppHandle, doc: &Path) {
    if let Some(dir) = canonical(doc).parent() {
        let _ = app.asset_protocol_scope().allow_directory(dir, true);
    }
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn mtime_of(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Write `bytes` to `path` without ever leaving a half-written file behind: the data goes
/// to a temporary file in the same directory, is flushed to disk, then renamed over the
/// target. Symlinks are followed and the original permissions are kept.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let target = canonical(path);
    let dir = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = target
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no file name"))?;
    let tmp = dir.join(format!(".{}.cedilha-tmp-{}", name.to_string_lossy(), std::process::id()));

    let result = (|| {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        if let Ok(meta) = fs::metadata(&target) {
            let _ = fs::set_permissions(&tmp, meta.permissions());
        }
        fs::rename(&tmp, &target)
    })();

    if let Err(e) = result {
        let _ = fs::remove_file(&tmp);
        // A directory we can't create files in (but can write the file itself): plain write.
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            return fs::File::create(&target).and_then(|mut f| f.write_all(bytes));
        }
        return Err(e);
    }
    Ok(())
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoadResult {
    pub path: Option<String>,
    pub content: String,
    pub prefs: crate::config::Prefs,
}

/// Extension filters offered by the open / save dialogs.
const MD_FILTERS: &[(&str, &[&str])] = &[
    ("Markdown", &["md", "mdown", "markdown", "mdx"]),
    ("Text", &["txt", "text"]),
    ("All Files", &["*"]),
];

/// Run an asynchronous file-open dialog.
async fn pick_async(title: &'static str) -> Option<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new().set_title(title);
    for &(name, exts) in MD_FILTERS {
        dialog = dialog.add_filter(name, exts);
    }
    dialog.pick_file().await.map(|f| f.path().to_path_buf())
}

/// Run an asynchronous save-as dialog.
async fn save_async(title: &'static str) -> Option<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new().set_title(title);
    for &(name, exts) in MD_FILTERS {
        dialog = dialog.add_filter(name, exts);
    }
    dialog.save_file().await.map(|f| f.path().to_path_buf())
}

/// Open a file through the native OS dialog.
#[tauri::command]
pub async fn open_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<LoadResult, String> {
    let path = pick_async("Open Document").await;
    let Some(path) = path else {
        return Ok(LoadResult {
            path: None,
            content: String::new(),
            prefs: state.prefs.lock().unwrap().clone(),
        });
    };

    let content = fs::read_to_string(&path).map_err(|e| format!("read failed: {e}"))?;
    state.allow(&path);
    allow_document_images(&app, &path);
    state.remember_mtime(&path);
    *state.current_path.lock().unwrap() = Some(path.clone());

    Ok(LoadResult {
        path: Some(path.to_string_lossy().into_owned()),
        content,
        prefs: state.prefs.lock().unwrap().clone(),
    })
}

/// Turn a command-line argument into a path. Launchers may pass `file:///a%20b.md` URLs
/// (desktop `%u`) instead of plain paths; decode those.
pub fn path_from_arg(arg: &str) -> PathBuf {
    let Some(rest) = arg.strip_prefix("file://") else {
        return PathBuf::from(arg);
    };
    // Drop an optional host ("file://localhost/..."): the path starts at the next '/'.
    let rest = if rest.starts_with('/') { rest } else { rest.find('/').map_or("", |i| &rest[i..]) };
    let bytes = rest.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = |b: u8| (b as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push((hi * 16 + lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    PathBuf::from(String::from_utf8_lossy(&out).into_owned())
}

/// The file given on the command line, if any. A path typed on the command line counts
/// as chosen by the user, so it joins the allowed set. Returns it only once.
#[tauri::command]
pub async fn initial_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Option<LoadResult>, String> {
    let Some(path) = state.initial_file.lock().unwrap().take() else {
        return Ok(None);
    };
    let content = fs::read_to_string(&path).map_err(|e| format!("read failed for {}: {e}", path.display()))?;
    state.allow(&path);
    allow_document_images(&app, &path);
    state.remember_mtime(&path);
    *state.current_path.lock().unwrap() = Some(path.clone());
    Ok(Some(LoadResult {
        path: Some(path.to_string_lossy().into_owned()),
        content,
        prefs: state.prefs.lock().unwrap().clone(),
    }))
}

/// Read a document the user already opened or saved (paths from the native dialogs only).
#[tauri::command]
pub async fn read_document(path: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if !state.is_allowed(&p) {
        return Err(format!("not permitted: {path}"));
    }
    fs::read_to_string(&p).map_err(|e| format!("read failed for {path}: {e}"))
}

/// Persist document content to disk.
///
/// Unless `force` is set, refuses with a `CONFLICT:` error when the file was modified by
/// something else since we last read or wrote it (autosave must never clobber that).
#[tauri::command]
pub async fn save_document(
    path: String,
    content: String,
    force: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !state.is_allowed(&p) {
        return Err(format!("not permitted: {path}"));
    }

    if !force {
        let recorded = *state.disk_mtime.lock().unwrap();
        if let (Some(recorded), Some(now)) = (recorded, mtime_of(&p)) {
            if recorded != now {
                return Err("CONFLICT: the file was changed on disk by another program".into());
            }
        }
    }

    write_atomic(&p, content.as_bytes()).map_err(|e| format!("write failed for {path}: {e}"))?;
    state.remember_mtime(&p);
    *state.current_path.lock().unwrap() = Some(p);
    Ok(())
}

/// Save under a new location using the native save dialog.
#[tauri::command]
pub async fn save_as_file(
    content: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Option<String>, String> {
    let path = save_async("Save Document").await;
    let Some(path) = path else {
        return Ok(None);
    };

    write_atomic(&path, content.as_bytes())
        .map_err(|e| format!("write failed for {}: {e}", path.display()))?;

    state.allow(&path);
    allow_document_images(&app, &path);
    state.remember_mtime(&path);
    *state.current_path.lock().unwrap() = Some(path.clone());
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Quit after the frontend confirmed there is nothing left to save.
#[tauri::command]
pub fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

/// The user backed out of closing the window.
#[tauri::command]
pub fn cancel_close(state: tauri::State<AppState>) {
    state.close_requested.store(false, Ordering::SeqCst);
}

/// Create a new empty document.
#[tauri::command]
pub fn new_document(state: tauri::State<AppState>) -> String {
    *state.current_path.lock().unwrap() = None;
    String::new()
}

/// Minimize the window. (No "toggle": once minimized there is nothing to press to restore it,
/// and asking for focus right after minimizing makes GNOME restore the window at once.)
#[tauri::command]
pub fn minimize_window(window: Window) {
    window.minimize().ok();
}

/// Only web and mail links may be handed to the system opener.
fn is_openable_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    let scheme_ok = lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("mailto:");
    scheme_ok && !url.chars().any(|c| c.is_control() || c.is_whitespace())
}

/// Open a link from the preview in the user's browser / mail client.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !is_openable_url(&url) {
        return Err(format!("refusing to open {url}"));
    }
    // The URL starts with a scheme, so it can never be mistaken for an option.
    let mut child = std::process::Command::new("xdg-open")
        .arg(&url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("could not run xdg-open: {e}"))?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Maximize the window, or restore it if it already is maximized.
#[tauri::command]
pub fn toggle_maximize(window: Window) {
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().ok();
    } else {
        window.maximize().ok();
    }
}

/// Window managers that tile or otherwise draw their own borders and title handling, where
/// our own close/minimize/maximize buttons would only be clutter.
const TILING_DESKTOPS: &[&str] = &[
    "hyprland", "sway", "i3", "bspwm", "river", "niri", "dwl", "awesome", "qtile",
    "herbstluftwm", "leftwm", "xmonad", "miracle", "scroll", "mango", "swayfx",
];

fn is_tiling_desktop(value: &str) -> bool {
    value
        .split(':')
        .any(|part| TILING_DESKTOPS.contains(&part.trim().to_lowercase().as_str()))
}

/// Whether we are running under a tiling window manager.
#[tauri::command]
pub fn is_tiling_wm() -> bool {
    let by_name = ["XDG_CURRENT_DESKTOP", "XDG_SESSION_DESKTOP", "DESKTOP_SESSION"]
        .iter()
        .filter_map(|k| std::env::var(k).ok())
        .any(|v| is_tiling_desktop(&v));
    let by_socket = ["HYPRLAND_INSTANCE_SIGNATURE", "SWAYSOCK", "I3SOCK", "NIRI_SOCKET"]
        .iter()
        .any(|k| std::env::var_os(k).is_some());
    by_name || by_socket
}

/// Fetch the currently persisted preferences.
#[tauri::command]
pub fn get_prefs(state: tauri::State<AppState>) -> crate::config::Prefs {
    state.prefs.lock().unwrap().clone()
}

/// Persist a new set of preferences.
#[tauri::command]
pub fn save_prefs(
    new_prefs: crate::config::Prefs,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    crate::config::save(&new_prefs).map_err(|e| e.to_string())?;
    *state.prefs.lock().unwrap() = new_prefs;
    Ok(())
}

/// Fetch the active Omarchy Linux theme, if present on the system.
#[tauri::command]
pub fn get_omarchy_theme() -> Option<crate::omarchy::OmarchyTheme> {
    crate::omarchy::read_current_omarchy_theme()
}

/// Whether a link/image destination is safe to put in an `href`/`src`.
/// Relative URLs and a short list of schemes are allowed; everything else
/// (`javascript:`, `vbscript:`, `data:` documents, ...) is rejected.
fn is_safe_url(url: &str, image: bool) -> bool {
    // Browsers ignore whitespace/control characters inside a scheme ("java\tscript:").
    let cleaned: String = url
        .chars()
        .filter(|c| !c.is_control() && !c.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    let Some(colon) = cleaned.find(':') else {
        return true; // relative URL
    };
    if cleaned[..colon].contains(['/', '?', '#']) {
        return true; // the colon belongs to a path or query, not a scheme
    }
    match &cleaned[..colon] {
        "http" | "https" => true,
        "mailto" | "tel" => !image,
        "data" => {
            image
                && ["png", "jpeg", "jpg", "gif", "webp", "avif"]
                    .iter()
                    .any(|t| cleaned.starts_with(&format!("data:image/{t}")))
        }
        _ => false,
    }
}

/// Render markdown to HTML, tagging blocks and math with `data-src-start` /
/// `data-src-end` (UTF-16 offsets into `content`, matching JS string indices) so the
/// preview can map a selection back to an exact source range.
fn render_with_sourcepos(content: &str) -> String {
    use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_MATH);

    // Byte offset -> UTF-16 offset. Identity for ASCII documents.
    let utf16: Option<Vec<u32>> = if content.is_ascii() {
        None
    } else {
        let mut map = vec![0u32; content.len() + 1];
        let mut acc = 0u32;
        for (i, ch) in content.char_indices() {
            for slot in map.iter_mut().skip(i).take(ch.len_utf8()) {
                *slot = acc;
            }
            acc += ch.len_utf16() as u32;
        }
        map[content.len()] = acc;
        Some(map)
    };
    let conv = |b: usize| -> usize {
        match &utf16 {
            Some(m) => m[b] as usize,
            None => b,
        }
    };

    let esc = |t: &str| {
        t.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    };

    let mut open_tables = 0usize;
    let events = Parser::new_ext(content, options)
        .into_offset_iter()
        .flat_map(|(ev, range)| {
            let attrs = format!(
                " data-src-start=\"{}\" data-src-end=\"{}\"",
                conv(range.start),
                conv(range.end)
            );
            let mut out: Vec<Event> = Vec::with_capacity(1);
            match ev {
                Event::Start(Tag::Paragraph) => {
                    out.push(Event::Html(format!("<p{attrs}>").into()))
                }
                Event::Start(Tag::BlockQuote(None)) => {
                    out.push(Event::Html(format!("<blockquote{attrs}>\n").into()))
                }
                Event::Start(Tag::Item) => {
                    out.push(Event::Html(format!("<li{attrs}>").into()))
                }
                Event::Start(Tag::Heading { level, id, classes, attrs: extra }) => {
                    let mut tag = format!("<{level}{attrs}");
                    if let Some(id) = &id {
                        tag.push_str(&format!(" id=\"{}\"", esc(id)));
                    }
                    if !classes.is_empty() {
                        let cls: Vec<String> = classes.iter().map(|c| esc(c)).collect();
                        tag.push_str(&format!(" class=\"{}\"", cls.join(" ")));
                    }
                    // `extra` (arbitrary `{key=value}` heading attributes) is dropped on purpose:
                    // keys like `onclick` would be script injection.
                    let _ = &extra;
                    tag.push('>');
                    out.push(Event::Html(tag.into()));
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    let lang = match &kind {
                        CodeBlockKind::Fenced(info) => info.split_whitespace().next().unwrap_or(""),
                        CodeBlockKind::Indented => "",
                    };
                    let code = if lang.is_empty() {
                        "<code>".to_string()
                    } else {
                        format!("<code class=\"language-{}\">", esc(lang))
                    };
                    out.push(Event::Html(format!("<pre{attrs}>{code}").into()));
                }
                // Tables keep the stock writer (it tracks column alignment), so wrap instead.
                Event::Start(Tag::Table(al)) => {
                    open_tables += 1;
                    out.push(Event::Html(
                        format!("<div class=\"table-src\"{attrs} style=\"display:contents\">").into(),
                    ));
                    out.push(Event::Start(Tag::Table(al)));
                }
                Event::End(pulldown_cmark::TagEnd::Table) => {
                    out.push(Event::End(pulldown_cmark::TagEnd::Table));
                    if open_tables > 0 {
                        open_tables -= 1;
                        out.push(Event::Html("</div>".into()));
                    }
                }
                Event::InlineMath(t) => out.push(Event::Html(
                    format!("<span class=\"math math-inline\"{attrs}>{}</span>", esc(&t)).into(),
                )),
                Event::DisplayMath(t) => out.push(Event::Html(
                    format!("<span class=\"math math-display\"{attrs}>{}</span>", esc(&t)).into(),
                )),
                // Raw HTML in the document is shown as text, never injected into the page.
                Event::Html(t) | Event::InlineHtml(t) => out.push(Event::Text(t)),
                Event::Start(Tag::Link { link_type, dest_url, title, id }) => {
                    let dest_url = if is_safe_url(&dest_url, false) { dest_url } else { "#".into() };
                    out.push(Event::Start(Tag::Link { link_type, dest_url, title, id }));
                }
                Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
                    let dest_url = if is_safe_url(&dest_url, true) { dest_url } else { "".into() };
                    out.push(Event::Start(Tag::Image { link_type, dest_url, title, id }));
                }
                other => out.push(other),
            }
            out
        });

    let mut html = String::with_capacity(content.len() * 3 / 2);
    pulldown_cmark::html::push_html(&mut html, events);
    html
}

/// High-performance asynchronous markdown renderer using native pulldown-cmark in Rust.
/// Capable of parsing megabyte-scale documents in milliseconds off the main UI thread.
#[tauri::command]
pub async fn render_markdown(content: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        render_with_sourcepos(&content)
    })
    .await
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomic_replaces_content_keeps_mode_and_leaves_no_temp_files() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("cedilha-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("doc.md");
        std::fs::write(&file, "old").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o640)).unwrap();

        // through a symlink: the real file must be updated, the link kept
        let link = dir.join("link.md");
        std::os::unix::fs::symlink(&file, &link).unwrap();
        write_atomic(&link, b"new content").unwrap();

        assert_eq!(std::fs::read_to_string(&file).unwrap(), "new content");
        assert!(std::fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        assert_eq!(std::fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o640);
        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("cedilha-tmp"))
            .collect();
        assert!(leftovers.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cli_arguments_accept_paths_and_file_urls() {
        assert_eq!(path_from_arg("notes.md"), PathBuf::from("notes.md"));
        assert_eq!(path_from_arg("/home/a/b c.md"), PathBuf::from("/home/a/b c.md"));
        assert_eq!(path_from_arg("file:///home/a/b%20c.md"), PathBuf::from("/home/a/b c.md"));
        assert_eq!(path_from_arg("file:///home/a/ação.md"), PathBuf::from("/home/a/ação.md"));
        assert_eq!(path_from_arg("file:///home/a/a%C3%A7%C3%A3o.md"), PathBuf::from("/home/a/ação.md"));
        assert_eq!(path_from_arg("file://localhost/tmp/x.md"), PathBuf::from("/tmp/x.md"));
        assert_eq!(path_from_arg("file:///tmp/100%.md"), PathBuf::from("/tmp/100%.md"));
        // a stray '%' before a multi-byte character must not panic
        assert_eq!(path_from_arg("file:///tmp/%aç.md"), PathBuf::from("/tmp/%aç.md"));
    }

    #[test]
    fn only_web_and_mail_links_can_be_opened() {
        assert!(is_openable_url("https://ui.adsabs.harvard.edu/abs/2003MNRAS.344.1000B"));
        assert!(is_openable_url("http://example.org"));
        assert!(is_openable_url("mailto:someone@example.org"));
        assert!(is_openable_url("HTTPS://EXAMPLE.ORG"));
        assert!(!is_openable_url("javascript:alert(1)"));
        assert!(!is_openable_url("file:///etc/passwd"));
        assert!(!is_openable_url("ftp://example.org"));
        assert!(!is_openable_url("--help"));
        assert!(!is_openable_url("https://exa mple.org"));
        assert!(!is_openable_url("https://example.org\n--evil"));
        assert!(!is_openable_url(""));
    }

    #[test]
    fn tiling_desktop_detection() {
        assert!(is_tiling_desktop("Hyprland"));
        assert!(is_tiling_desktop("sway"));
        assert!(is_tiling_desktop("X-Cinnamon:i3"));
        assert!(!is_tiling_desktop("GNOME"));
        assert!(!is_tiling_desktop("ubuntu:GNOME"));
        assert!(!is_tiling_desktop("KDE"));
        assert!(!is_tiling_desktop(""));
    }

    #[test]
    fn test_sourcepos_attributes() {
        let input = "# Título\n\nação $x$ e\n\n$$y$$\n\n- a\n- b\n\n| a | b |\n|---|---|\n| 1 | 2 |\n";
        let html = render_with_sourcepos(input);
        println!("{html}");
        assert!(html.contains("<h1 data-src-start=\"0\" data-src-end=\"9\">"));
        assert!(html.contains("<p data-src-start=\"10\""));
        assert!(html.contains("math-inline\" data-src-start=\"15\" data-src-end=\"18\""));
        assert!(html.contains("<li data-src-start"));
        assert!(html.contains("table-src"));
    }

    #[test]
    fn test_render_is_sanitized() {
        let input = "<img src=x onerror=alert(1)>\n\n<script>alert(1)</script>\n\n\
            text <b onclick=x>b</b> [a](javascript:alert(1)) [b](JaVa\tScRiPt:alert(1)) \
            [ok](https://x.y/a:b) [rel](a/b.md) ![i](data:text/html;base64,AAAA) \
            ![p](data:image/png;base64,AAAA)\n\n# H {onclick=alert(1) #id .cls}\n";
        let html = render_with_sourcepos(input);
        println!("{html}");
        assert!(!html.contains("<img src=x"), "raw html must be escaped");
        assert!(!html.contains("<script"));
        assert!(!html.contains("<b onclick"));
        assert!(!html.contains("javascript:"));
        assert!(!html.to_lowercase().contains("href=\"java"));
        assert!(!html.contains("onclick=alert"));
        assert!(html.contains("href=\"https://x.y/a:b\""));
        assert!(html.contains("href=\"a/b.md\""));
        assert!(!html.contains("data:text/html"));
        assert!(html.contains("data:image/png"));
        assert!(html.contains("id=\"id\"") && html.contains("class=\"cls\""));
    }

    #[test]
    fn test_render_markdown_math() {
        tauri::async_runtime::block_on(async {
            let input = "Math with symbols: $x < y$ and $a > b$ and $a & b$\n";
            let res = render_markdown(input.to_string()).await.unwrap();
            println!("RENDERED HTML:\n{}", res);
        });
    }

    #[test]
    fn test_render_huge_galaxy_book() {
        let path = "/tmp/sample.md";
        if let Ok(content) = std::fs::read_to_string(path) {
            let start = std::time::Instant::now();
            let html = tauri::async_runtime::block_on(async {
                render_markdown(content).await.unwrap()
            });
            let elapsed = start.elapsed();
            println!("Huge file parsed in {:?}! HTML len: {}", elapsed, html.len());
        }
    }
}

