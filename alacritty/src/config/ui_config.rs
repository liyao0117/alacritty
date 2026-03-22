use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Formatter};
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;

use log::{error, warn};
use serde::de::{Error as SerdeError, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use unicode_width::UnicodeWidthChar;
use winit::keyboard::{Key, ModifiersState};

use alacritty_config::SerdeReplace;
use alacritty_config_derive::{ConfigDeserialize, SerdeReplace};
use alacritty_terminal::term::Config as TermConfig;
use alacritty_terminal::term::search::RegexSearch;
use alacritty_terminal::tty::{Options as PtyOptions, Shell};

use crate::config::LOG_TARGET_CONFIG;
use crate::config::bell::BellConfig;
use crate::config::bindings::{
    self, Action, Binding, BindingKey, KeyBinding, KeyLocation, ModeWrapper, ModsWrapper,
    MouseBinding,
};
use crate::config::color::Colors;
use crate::config::cursor::Cursor;
use crate::config::debug::Debug;
use crate::config::font::Font;
use crate::config::general::General;
use crate::config::mouse::Mouse;
use crate::config::scrolling::Scrolling;
use crate::config::selection::Selection;
use crate::config::terminal::Terminal;
use crate::config::window::WindowConfig;
use alacritty_terminal::term::buffer_search::matcher::SearchConfig as BufferSearchConfig;

/// Regex used for the default URL hint.
#[rustfmt::skip]
const URL_REGEX: &str = "(ipfs:|ipns:|magnet:|mailto:|gemini://|gopher://|https://|http://|news:|file:|git://|ssh:|ftp://)\
                         [^\u{0000}-\u{001F}\u{007F}-\u{009F}<>\"\\s{-}\\^⟨⟩`\\\\]+";

#[derive(ConfigDeserialize, Serialize, Default, Clone, Debug, PartialEq)]
pub struct UiConfig {
    /// Miscellaneous configuration options.
    pub general: General,

    /// Extra environment variables.
    pub env: HashMap<String, String>,

    /// How much scrolling history to keep.
    pub scrolling: Scrolling,

    /// Cursor configuration.
    pub cursor: Cursor,

    /// Selection configuration.
    pub selection: Selection,

    /// Font configuration.
    pub font: Font,

    /// Window configuration.
    pub window: WindowConfig,

    /// Mouse configuration.
    pub mouse: Mouse,

    /// Debug options.
    pub debug: Debug,

    /// Bell configuration.
    pub bell: BellConfig,

    /// RGB values for colors.
    pub colors: Colors,

    /// Path where config was loaded from.
    #[config(skip)]
    #[serde(skip_serializing)]
    pub config_paths: Vec<PathBuf>,

    /// Regex hints for interacting with terminal content.
    pub hints: Hints,

    /// Config for the alacritty_terminal itself.
    pub terminal: Terminal,

    /// Buffer search configuration.
    pub buffer_search: BufferSearchConfigSection,

    /// Keyboard configuration.
    keyboard: Keyboard,

    /// Path to a shell program to run on startup.
    #[config(deprecated = "use terminal.shell instead")]
    shell: Option<Program>,

    /// Configuration file imports.
    ///
    /// This is never read since the field is directly accessed through the config's
    /// [`toml::Value`], but still present to prevent unused field warnings.
    #[config(deprecated = "use general.import instead")]
    import: Option<Vec<String>>,

    /// Shell startup directory.
    #[config(deprecated = "use general.working_directory instead")]
    working_directory: Option<PathBuf>,

    /// Live config reload.
    #[config(deprecated = "use general.live_config_reload instead")]
    live_config_reload: Option<bool>,

    /// Offer IPC through a unix socket.
    #[cfg(unix)]
    #[config(deprecated = "use general.ipc_socket instead")]
    pub ipc_socket: Option<bool>,
}

impl UiConfig {
    /// Derive [`TermConfig`] from the config.
    pub fn term_options(&self) -> TermConfig {
        TermConfig {
            semantic_escape_chars: self.selection.semantic_escape_chars.clone(),
            scrolling_history: self.scrolling.history() as usize,
            vi_mode_cursor_style: self.cursor.vi_mode_style(),
            default_cursor_style: self.cursor.style(),
            osc52: self.terminal.osc52.0,
            kitty_keyboard: true,
        }
    }

    /// Get buffer search config.
    pub fn buffer_search_config(&self) -> BufferSearchConfig {
        BufferSearchConfig::from(&self.buffer_search)
    }

    /// Apply buffer search toggle key binding from configuration.
    pub fn apply_buffer_search_binding(&mut self) {
        use crate::config::bindings::{Action, BindingKey, BindingMode, KeyBinding, KeyLocation};
        use winit::keyboard::{Key, ModifiersState};
        
        // Parse the toggle_key string (e.g., "Ctrl+Shift+F")
        let toggle_key = &self.buffer_search.toggle_key;
        
        // Parse modifiers and key from the string
        let parts: Vec<&str> = toggle_key.split('+').collect();
        let mut mods = ModifiersState::empty();
        let mut key_str = "";
        
        for part in parts {
            match part.trim().to_lowercase().as_str() {
                "ctrl" | "control" => mods |= ModifiersState::CONTROL,
                "shift" => mods |= ModifiersState::SHIFT,
                "alt" => mods |= ModifiersState::ALT,
                "super" | "win" | "command" => mods |= ModifiersState::SUPER,
                _ => key_str = part.trim(),
            }
        }
        
        // Parse the key - letters and numbers use Character, special keys use NamedKey
        let key_val = match key_str {
            "F" | "f" => Key::Character("f".into()),
            "A" | "a" => Key::Character("a".into()),
            "B" | "b" => Key::Character("b".into()),
            "C" | "c" => Key::Character("c".into()),
            "D" | "d" => Key::Character("d".into()),
            "E" | "e" => Key::Character("e".into()),
            "G" | "g" => Key::Character("g".into()),
            "H" | "h" => Key::Character("h".into()),
            "I" | "i" => Key::Character("i".into()),
            "J" | "j" => Key::Character("j".into()),
            "K" | "k" => Key::Character("k".into()),
            "L" | "l" => Key::Character("l".into()),
            "M" | "m" => Key::Character("m".into()),
            "N" | "n" => Key::Character("n".into()),
            "O" | "o" => Key::Character("o".into()),
            "P" | "p" => Key::Character("p".into()),
            "Q" | "q" => Key::Character("q".into()),
            "R" | "r" => Key::Character("r".into()),
            "S" | "s" => Key::Character("s".into()),
            "T" | "t" => Key::Character("t".into()),
            "U" | "u" => Key::Character("u".into()),
            "V" | "v" => Key::Character("v".into()),
            "W" | "w" => Key::Character("w".into()),
            "X" | "x" => Key::Character("x".into()),
            "Y" | "y" => Key::Character("y".into()),
            "Z" | "z" => Key::Character("z".into()),
            "0" => Key::Character("0".into()),
            "1" => Key::Character("1".into()),
            "2" => Key::Character("2".into()),
            "3" => Key::Character("3".into()),
            "4" => Key::Character("4".into()),
            "5" => Key::Character("5".into()),
            "6" => Key::Character("6".into()),
            "7" => Key::Character("7".into()),
            "8" => Key::Character("8".into()),
            "9" => Key::Character("9".into()),
            _ => Key::Character(key_str.to_lowercase().into()),
        };
        
        // Create the binding using Keycode format
        let new_binding = KeyBinding {
            trigger: BindingKey::Keycode { key: key_val.clone(), location: KeyLocation::Any },
            mods: mods,
            action: Action::StartBufferFuzzySearch,
            mode: BindingMode::empty(),
            notmode: BindingMode::empty(),  // Remove the notmode restriction to allow toggle
        };
        
        // Remove ALL StartBufferFuzzySearch bindings to avoid conflicts
        // This allows the config to fully override the default binding
        self.keyboard.bindings.0.retain(|b| {
            !matches!(b.action, Action::StartBufferFuzzySearch)
        });
        
        // Add the new binding from config
        self.keyboard.bindings.0.push(new_binding);
        
        eprintln!("DEBUG: Applied buffer search binding: key={:?}, mods={:?}", key_val, mods);
    }

    /// Derive [`PtyOptions`] from the config.
    pub fn pty_config(&self) -> PtyOptions {
        let shell = self.terminal.shell.clone().or_else(|| self.shell.clone()).map(Into::into);
        let working_directory =
            self.working_directory.clone().or_else(|| self.general.working_directory.clone());
        PtyOptions {
            working_directory,
            shell,
            drain_on_exit: false,
            env: HashMap::new(),
            #[cfg(target_os = "windows")]
            escape_args: false,
        }
    }

    #[inline]
    pub fn window_opacity(&self) -> f32 {
        self.window.opacity.as_f32()
    }

    #[inline]
    pub fn key_bindings(&self) -> &[KeyBinding] {
        &self.keyboard.bindings.0
    }

    #[inline]
    pub fn mouse_bindings(&self) -> &[MouseBinding] {
        &self.mouse.bindings.0
    }

    #[inline]
    pub fn live_config_reload(&self) -> bool {
        self.live_config_reload.unwrap_or(self.general.live_config_reload)
    }

    #[cfg(unix)]
    #[inline]
    pub fn ipc_socket(&self) -> bool {
        self.ipc_socket.unwrap_or(self.general.ipc_socket)
    }
}

/// Keyboard configuration.
#[derive(ConfigDeserialize, Serialize, Default, Clone, Debug, PartialEq)]
struct Keyboard {
    /// Keybindings.
    #[serde(skip_serializing)]
    bindings: KeyBindings,
}

#[derive(SerdeReplace, Clone, Debug, PartialEq, Eq)]
struct KeyBindings(Vec<KeyBinding>);

impl Default for KeyBindings {
    fn default() -> Self {
        Self(bindings::default_key_bindings())
    }
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(deserialize_bindings(deserializer, Self::default().0)?))
    }
}

pub fn deserialize_bindings<'a, D, T>(
    deserializer: D,
    mut default: Vec<Binding<T>>,
) -> Result<Vec<Binding<T>>, D::Error>
where
    D: Deserializer<'a>,
    T: Clone + Eq,
    Binding<T>: Deserialize<'a>,
{
    let values = Vec::<toml::Value>::deserialize(deserializer)?;

    // Skip all invalid values.
    let mut bindings = Vec::with_capacity(values.len());
    for value in values {
        match Binding::<T>::deserialize(value) {
            Ok(binding) => bindings.push(binding),
            Err(err) => {
                error!(target: LOG_TARGET_CONFIG, "Config error: {err}; ignoring binding");
            },
        }
    }

    // Remove matching default bindings.
    for binding in bindings.iter() {
        default.retain(|b| !b.triggers_match(binding));
    }

    bindings.extend(default);

    Ok(bindings)
}

/// A delta for a point in a 2 dimensional plane.
#[derive(ConfigDeserialize, Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Delta<T: Default> {
    /// Horizontal change.
    pub x: T,
    /// Vertical change.
    pub y: T,
}

/// Regex terminal hints.
#[derive(ConfigDeserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Hints {
    /// Characters for the hint labels.
    alphabet: HintsAlphabet,

    /// All configured terminal hints.
    pub enabled: Vec<Rc<Hint>>,
}

impl Default for Hints {
    fn default() -> Self {
        // Add URL hint by default when no other hint is present.
        let pattern = LazyRegexVariant::Pattern(String::from(URL_REGEX));
        let regex = LazyRegex(Rc::new(RefCell::new(pattern)));
        let content = HintContent::new(Some(regex), true);

        #[cfg(not(any(target_os = "macos", windows)))]
        let action = HintAction::Command(Program::Just(String::from("xdg-open")));
        #[cfg(target_os = "macos")]
        let action = HintAction::Command(Program::Just(String::from("open")));
        #[cfg(windows)]
        let action = HintAction::Command(Program::WithArgs {
            program: String::from("cmd"),
            args: vec!["/c".to_string(), "start".to_string(), "".to_string()],
        });

        Self {
            enabled: vec![Rc::new(Hint {
                content,
                action,
                persist: false,
                post_processing: true,
                mouse: Some(HintMouse { enabled: true, mods: Default::default() }),
                binding: Some(HintBinding {
                    key: BindingKey::Keycode {
                        key: Key::Character("o".into()),
                        location: KeyLocation::Standard,
                    },
                    mods: ModsWrapper(ModifiersState::SHIFT | ModifiersState::CONTROL),
                    cache: Default::default(),
                    mode: Default::default(),
                }),
            })],
            alphabet: Default::default(),
        }
    }
}

impl Hints {
    /// Characters for the hint labels.
    pub fn alphabet(&self) -> &str {
        &self.alphabet.0
    }
}

#[derive(SerdeReplace, Serialize, Clone, Debug, PartialEq, Eq)]
struct HintsAlphabet(String);

impl Default for HintsAlphabet {
    fn default() -> Self {
        Self(String::from("jfkdls;ahgurieowpq"))
    }
}

impl<'de> Deserialize<'de> for HintsAlphabet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        let mut character_count = 0;
        for character in value.chars() {
            if character.width() != Some(1) {
                return Err(D::Error::custom("characters must be of width 1"));
            }
            character_count += 1;
        }

        if character_count < 2 {
            return Err(D::Error::custom("must include at last 2 characters"));
        }

        Ok(Self(value))
    }
}

/// Built-in actions for hint mode.
#[derive(ConfigDeserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum HintInternalAction {
    /// Copy the text to the clipboard.
    Copy,
    /// Write the text to the PTY/search.
    Paste,
    /// Select the text matching the hint.
    Select,
    /// Move the vi mode cursor to the beginning of the hint.
    MoveViModeCursor,
}

/// Actions for hint bindings.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum HintAction {
    /// Built-in hint action.
    #[serde(rename = "action")]
    Action(HintInternalAction),

    /// Command the text will be piped to.
    #[serde(rename = "command")]
    Command(Program),
}

/// Hint configuration.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Hint {
    /// Regex for finding matches.
    #[serde(flatten)]
    pub content: HintContent,

    /// Action executed when this hint is triggered.
    #[serde(flatten)]
    pub action: HintAction,

    /// Hint text post processing.
    #[serde(default)]
    pub post_processing: bool,

    /// Persist hints after selection.
    #[serde(default)]
    pub persist: bool,

    /// Hint mouse highlighting.
    pub mouse: Option<HintMouse>,

    /// Binding required to search for this hint.
    #[serde(skip_serializing)]
    pub binding: Option<HintBinding>,
}

#[derive(Serialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct HintContent {
    /// Regex for finding matches.
    pub regex: Option<LazyRegex>,

    /// Escape sequence hyperlinks.
    pub hyperlinks: bool,
}

impl HintContent {
    pub fn new(regex: Option<LazyRegex>, hyperlinks: bool) -> Self {
        Self { regex, hyperlinks }
    }
}

impl<'de> Deserialize<'de> for HintContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HintContentVisitor;
        impl<'a> Visitor<'a> for HintContentVisitor {
            type Value = HintContent;

            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("a mapping")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'a>,
            {
                let mut content = Self::Value::default();

                while let Some((key, value)) = map.next_entry::<String, toml::Value>()? {
                    match key.as_str() {
                        "regex" => match Option::<LazyRegex>::deserialize(value) {
                            Ok(regex) => content.regex = regex,
                            Err(err) => {
                                error!(
                                    target: LOG_TARGET_CONFIG,
                                    "Config error: hint's regex: {err}"
                                );
                            },
                        },
                        "hyperlinks" => match bool::deserialize(value) {
                            Ok(hyperlink) => content.hyperlinks = hyperlink,
                            Err(err) => {
                                error!(
                                    target: LOG_TARGET_CONFIG,
                                    "Config error: hint's hyperlinks: {err}"
                                );
                            },
                        },
                        "command" | "action" => (),
                        key => warn!(target: LOG_TARGET_CONFIG, "Unrecognized hint field: {key}"),
                    }
                }

                // Require at least one of hyperlinks or regex trigger hint matches.
                if content.regex.is_none() && !content.hyperlinks {
                    return Err(M::Error::custom(
                        "Config error: At least one of the hint's regex or hint's hyperlinks must \
                         be set",
                    ));
                }

                Ok(content)
            }
        }

        deserializer.deserialize_any(HintContentVisitor)
    }
}

/// Binding for triggering a keyboard hint.
#[derive(Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HintBinding {
    pub key: BindingKey,
    #[serde(default)]
    pub mods: ModsWrapper,
    #[serde(default)]
    pub mode: ModeWrapper,

    /// Cache for on-demand [`HintBinding`] to [`KeyBinding`] conversion.
    #[serde(skip)]
    cache: OnceCell<KeyBinding>,
}

impl HintBinding {
    /// Get the key binding for a hint.
    pub fn key_binding(&self, hint: &Rc<Hint>) -> &KeyBinding {
        self.cache.get_or_init(|| KeyBinding {
            trigger: self.key.clone(),
            mods: self.mods.0,
            mode: self.mode.mode,
            notmode: self.mode.not_mode,
            action: Action::Hint(hint.clone()),
        })
    }
}

impl fmt::Debug for HintBinding {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("HintBinding")
            .field("key", &self.key)
            .field("mods", &self.mods)
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

/// Hint mouse highlighting.
#[derive(ConfigDeserialize, Serialize, Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct HintMouse {
    /// Hint mouse highlighting availability.
    pub enabled: bool,

    /// Required mouse modifiers for hint highlighting.
    #[serde(skip_serializing)]
    pub mods: ModsWrapper,
}

/// Lazy regex with interior mutability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LazyRegex(Rc<RefCell<LazyRegexVariant>>);

impl LazyRegex {
    /// Execute a function with the compiled regex DFAs as parameter.
    pub fn with_compiled<T, F>(&self, f: F) -> Option<T>
    where
        F: FnMut(&mut RegexSearch) -> T,
    {
        self.0.borrow_mut().compiled().map(f)
    }
}

impl<'de> Deserialize<'de> for LazyRegex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let regex = LazyRegexVariant::Pattern(String::deserialize(deserializer)?);
        Ok(Self(Rc::new(RefCell::new(regex))))
    }
}

impl Serialize for LazyRegex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let variant = self.0.borrow();
        let regex = match &*variant {
            LazyRegexVariant::Compiled(regex, _) => regex,
            LazyRegexVariant::Uncompilable(regex) => regex,
            LazyRegexVariant::Pattern(regex) => regex,
        };
        serializer.serialize_str(regex)
    }
}

/// Regex which is compiled on demand, to avoid expensive computations at startup.
#[derive(Clone, Debug)]
pub enum LazyRegexVariant {
    Compiled(String, Box<RegexSearch>),
    Pattern(String),
    Uncompilable(String),
}

impl LazyRegexVariant {
    /// Get a reference to the compiled regex.
    ///
    /// If the regex is not already compiled, this will compile the DFAs and store them for future
    /// access.
    fn compiled(&mut self) -> Option<&mut RegexSearch> {
        // Check if the regex has already been compiled.
        let regex = match self {
            Self::Compiled(_, regex_search) => return Some(regex_search),
            Self::Uncompilable(_) => return None,
            Self::Pattern(regex) => mem::take(regex),
        };

        // Compile the regex.
        let regex_search = match RegexSearch::new(&regex) {
            Ok(regex_search) => regex_search,
            Err(err) => {
                error!("could not compile hint regex: {err}");
                *self = Self::Uncompilable(regex);
                return None;
            },
        };
        *self = Self::Compiled(regex, Box::new(regex_search));

        // Return a reference to the compiled DFAs.
        match self {
            Self::Compiled(_, dfas) => Some(dfas),
            _ => unreachable!(),
        }
    }
}

impl PartialEq for LazyRegexVariant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Pattern(regex), Self::Pattern(other_regex)) => regex == other_regex,
            _ => false,
        }
    }
}
impl Eq for LazyRegexVariant {}

/// Wrapper around f32 that represents a percentage value between 0.0 and 1.0.
#[derive(SerdeReplace, Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Percentage(f32);

impl Default for Percentage {
    fn default() -> Self {
        Percentage(1.0)
    }
}

impl Percentage {
    pub fn new(value: f32) -> Self {
        Percentage(value.clamp(0., 1.))
    }

    pub fn as_f32(self) -> f32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Percentage::new(f32::deserialize(deserializer)?))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged, deny_unknown_fields)]
pub enum Program {
    Just(String),
    WithArgs {
        program: String,
        #[serde(default)]
        args: Vec<String>,
    },
}

impl Program {
    pub fn program(&self) -> &str {
        match self {
            Program::Just(program) => program,
            Program::WithArgs { program, .. } => program,
        }
    }

    pub fn args(&self) -> &[String] {
        match self {
            Program::Just(_) => &[],
            Program::WithArgs { args, .. } => args,
        }
    }
}

impl From<Program> for Shell {
    fn from(value: Program) -> Self {
        match value {
            Program::Just(program) => Shell::new(program, Vec::new()),
            Program::WithArgs { program, args } => Shell::new(program, args),
        }
    }
}

impl SerdeReplace for Program {
    fn replace(&mut self, value: toml::Value) -> Result<(), Box<dyn Error>> {
        *self = Self::deserialize(value)?;

        Ok(())
    }
}

pub(crate) struct StringVisitor;
impl serde::de::Visitor<'_> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(s.to_lowercase())
    }
}

/// Buffer search configuration section.
#[derive(ConfigDeserialize, Default, Clone, Debug, PartialEq, Serialize)]
pub struct BufferSearchConfigSection {
    /// Whether search is case sensitive (default: false).
    pub case_sensitive: bool,
    
    /// Key binding to toggle buffer search mode (default: Ctrl+Shift+F).
    #[serde(default = "default_buffer_search_key")]
    pub toggle_key: String,
}

#[allow(dead_code)]  // Used by serde default attribute
fn default_buffer_search_key() -> String {
    "Ctrl+Shift+T".to_string()
}

impl From<&BufferSearchConfigSection> for BufferSearchConfig {
    fn from(config: &BufferSearchConfigSection) -> Self {
        BufferSearchConfig {
            case_sensitive: config.case_sensitive,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alacritty_terminal::term::test::mock_term;

    use crate::display::hint::visible_regex_match_iter;

    #[test]
    fn positive_url_parsing_regex_test() {
        for regular_url in [
            "ipfs:s0mEhAsh",
            "ipns:an0TherHash1234",
            "magnet:?xt=urn:btih:L0UDHA5H12",
            "mailto:example@example.org",
            "gemini://gemini.example.org/",
            "gopher://gopher.example.org",
            "https://www.example.org",
            "http://example.org",
            "news:some.news.portal",
            "file:///C:/Windows/",
            "file:/home/user/whatever",
            "git://github.com/user/repo.git",
            "ssh:git@github.com:user/repo.git",
            "ftp://ftp.example.org",
        ] {
            let term = mock_term(regular_url);
            let mut regex = RegexSearch::new(URL_REGEX).unwrap();
            let matches = visible_regex_match_iter(&term, &mut regex).collect::<Vec<_>>();
            assert_eq!(
                matches.len(),
                1,
                "Should have exactly one match url {regular_url}, but instead got: {matches:?}"
            )
        }
    }

    #[test]
    fn negative_url_parsing_regex_test() {
        for url_like in [
            "http::trace::on_request::log_parameters",
            "http//www.example.org",
            "/user:example.org",
            "mailto: example@example.org",
            "http://<script>alert('xss')</script>",
            "mailto:",
        ] {
            let term = mock_term(url_like);
            let mut regex = RegexSearch::new(URL_REGEX).unwrap();
            let matches = visible_regex_match_iter(&term, &mut regex).collect::<Vec<_>>();
            assert!(
                matches.is_empty(),
                "Should not match url in string {url_like}, but instead got: {matches:?}"
            )
        }
    }
}
