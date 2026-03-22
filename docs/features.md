# Features

This document gives an overview over Alacritty's features beyond its terminal
emulation capabilities. To get a list with supported control sequences take a
look at [Alacritty's escape sequence support](./escape_support.md).

## Vi Mode

The vi mode allows moving around Alacritty's viewport and scrollback using the
keyboard. It also serves as a jump-off point for other features like search and
opening URLs with the keyboard. By default you can launch it using
<kbd>Ctrl</kbd> <kbd>Shift</kbd> <kbd>Space</kbd>.

### Motion

The cursor motions are setup by default to mimic vi, however they are fully
configurable. If you don't like vi's bindings, take a look at the configuration
file to change the various movements.

### Selection

One useful feature of vi mode is the ability to make selections and copy text to
the clipboard. By default you can start a selection using <kbd>v</kbd> and copy
it using <kbd>y</kbd>. All selection modes that are available with the mouse can
be accessed from vi mode, including the semantic (<kbd>Alt</kbd> <kbd>v</kbd>),
line (<kbd>Shift</kbd> <kbd>v</kbd>) and block selection (<kbd>Ctrl</kbd>
<kbd>v</kbd>). You can also toggle between them while the selection is still
active.

## Search

Search allows you to find anything in Alacritty's scrollback buffer. You can
search forward using <kbd>Ctrl</kbd> <kbd>Shift</kbd> <kbd>f</kbd> (<kbd>Command</kbd> <kbd>f</kbd> on macOS) and
backward using <kbd>Ctrl</kbd> <kbd>Shift</kbd> <kbd>b</kbd> (<kbd>Command</kbd> <kbd>b</kbd> on macOS).

### Vi Search

In vi mode the search is bound to <kbd>/</kbd> for forward and <kbd>?</kbd> for
backward search. This allows you to move around quickly and help with selecting
content. The `SearchStart` and `SearchEnd` keybinding actions can be bound if
you're looking for a way to jump to the start or the end of a match.

### Normal Search

During normal search you don't have the opportunity to move around freely, but
you can still jump between matches using <kbd>Enter</kbd> and <kbd>Shift</kbd>
<kbd>Enter</kbd>. After leaving search with <kbd>Escape</kbd> your active match
stays selected, allowing you to easily copy it.

## Hints

Terminal hints allow easily interacting with visible text without having to
start vi mode. They consist of a regex that detects these text elements and then
either feeds them to an external application or triggers one of Alacritty's
built-in actions.

Hints can also be triggered using the mouse or vi mode cursor. If a hint is
enabled for mouse interaction and recognized as such, it will be underlined when
the mouse or vi mode cursor is on top of it. Using the left mouse button or
<kbd>Enter</kbd> key in vi mode will then trigger the hint.

Hints can be configured in the `hints` and `colors.hints` sections in the
Alacritty configuration file.

## Selection expansion

After making a selection, you can use the right mouse button to expand it.
Double-clicking will expand the selection semantically, while triple-clicking
will perform line selection. If you hold <kbd>Ctrl</kbd> while expanding the
selection, it will switch to the block selection mode.

## Opening URLs with the mouse

You can open URLs with your mouse by clicking on them. The modifiers required to
be held and program which should open the URL can be setup in the configuration
file. If an application captures your mouse clicks, which is indicated by a
change in mouse cursor shape, you're required to hold <kbd>Shift</kbd> to bypass
that.

## Multi-Window

Alacritty supports running multiple terminal emulators from the same Alacritty
instance. New windows can be created either by using the `CreateNewWindow`
keybinding action, or by executing the `alacritty msg create-window` subcommand.

## Buffer Fuzzy Search

Buffer Fuzzy Search provides powerful, fuzzy-matching-based search for your terminal's scrollback buffer. It allows you to quickly find and filter lines using intelligent pattern matching, with support for multiple match modes, multi-selection, and column filtering.

### Activating Buffer Fuzzy Search

**Default Keybinding:** <kbd>Ctrl</kbd> <kbd>Shift</kbd> <kbd>T</kbd>

Press the keybinding to toggle Buffer Fuzzy Search mode. When active, you'll see a prompt at the bottom showing `Fuzzy: <query>` with match count.

**Exit:** Press <kbd>Escape</kbd> or press the toggle keybinding again to exit.

### Basic Usage

1. **Activate:** Press <kbd>Ctrl</kbd> <kbd>Shift</kbd> <kbd>T</kbd>
2. **Type Query:** Start typing your search query
3. **Navigate Results:**
   - <kbd>↑</kbd> / <kbd>↓</kbd> or <kbd>Ctrl</kbd> <kbd>P</kbd> / <kbd>Ctrl</kbd> <kbd>N</kbd>: Move selection up/down
   - <kbd>Page Up</kbd> / <kbd>Page Down</kbd>: Scroll by page
   - <kbd>Home</kbd> / <kbd>End</kbd>: Jump to first/last result
4. **Jump to Line:** Press <kbd>Enter</kbd> to jump to the selected line in the terminal
5. **Copy Content:** Press <kbd>Ctrl</kbd> <kbd>C</kbd> to copy selected line(s) to clipboard

### Match Modes

Buffer Fuzzy Search uses nucleo-matcher's native pattern syntax for different match modes:

| Pattern | Mode | Description | Example |
|---------|------|-------------|---------|
| `foo` | Fuzzy (default) | Matches characters in order, not necessarily contiguous | `fz` matches "fuzzy search" |
| `'foo` | Substring | Matches exact contiguous substring | `'error` matches "error" but not "err_or" |
| `^foo` | Prefix | Matches lines starting with foo | `^git` matches "git commit" |
| `foo$` | Postfix | Matches lines ending with foo | `rs$` matches "main.rs" |
| `^foo$` | Exact | Matches lines exactly equal to foo | `^foo$` matches only "foo" |

### Case Sensitivity

Toggle case-sensitive matching:

- **Default:** Case-insensitive (matches "Hello", "hello", "HELLO")
- **Toggle:** Press <kbd>Ctrl</kbd> <kbd>Shift</kbd> <kbd>C</kbd> to toggle case sensitivity
- **Indicator:** `Aa` (case-sensitive) or `aa` (case-insensitive) shown in prompt

### Multi-Select Mode (Phase 4)

Select and copy multiple lines at once:

- **Toggle Selection:** <kbd>Tab</kbd> - Toggle selection for current line
- **Select All:** <kbd>Ctrl</kbd> <kbd>A</kbd> - Select/deselect all visible results
- **Copy Selected:** <kbd>Ctrl</kbd> <kbd>C</kbd> - Copy all selected lines to clipboard
- **Visual Indicator:** Selected lines are highlighted with a different background color

### Column Filtering (Phase 4.3)

Filter search by specific columns when output has structured data:

Configure in `alacritty.toml`:
```toml
[buffer_search]
# Column delimiter (e.g., ":", " ", "\t")
delimiter = ":"

# Search specific columns (1-based indices)
# Empty means search entire line
nth = [1, 3]  # Search only columns 1 and 3
```

### Configuration Options

Add to your `alacritty.toml`:

```toml
# Buffer Search configuration
[buffer_search]

# Toggle case sensitivity (default: false)
case_sensitive = false

# Custom toggle keybinding (default: "Ctrl+Shift+T")
# Format: "Modifiers+Key"
# Modifiers: Ctrl, Shift, Alt, Super (Win/Command)
toggle_key = "Ctrl+Shift+T"

# Column filtering
# Delimiter for splitting columns
delimiter = ":"

# Column indices to search (1-based, empty = search all)
nth = [1, 2]
```

### Custom Keybindings

You can also configure the toggle keybinding using the traditional `keyboard.bindings` approach:

```toml
[[keyboard.bindings]]
key = "T"
mods = "Control|Shift"
action = "StartBufferFuzzySearch"
```

### Example Workflows

#### Find Error Messages
```
1. Press Ctrl+Shift+T
2. Type: error
3. Navigate with ↑/↓
4. Press Enter to jump to the line
```

#### Find Git Commands (Prefix Match)
```
1. Press Ctrl+Shift+T
2. Type: ^git
3. Only lines starting with "git" are shown
```

#### Find Rust Files (Postfix Match)
```
1. Press Ctrl+Shift+T
2. Type: .rs$
3. Only lines ending with ".rs" are shown
```

#### Copy Multiple Log Lines
```
1. Press Ctrl+Shift+T
2. Type: ERROR
3. Press Tab on each error line to select
4. Press Ctrl+C to copy all selected lines
```

#### Exact Command Search
```
1. Press Ctrl+Shift+T
2. Type: ^cargo build$
3. Only exact "cargo build" commands are shown
```

### Tips

- **Empty Query:** When query is empty, all non-empty lines are shown
- **Wrap-Around Navigation:** Navigation wraps around (last → first, first → last)
- **IME Support:** IME input is supported for non-English queries
- **Performance:** Results are sorted by match score (best matches first)
- **Scrollback:** Searches entire scrollback buffer, not just visible content

### Troubleshooting

**Keybinding not working?**
- Check for conflicts with other applications or system shortcuts
- Verify configuration file syntax
- Try restarting Alacritty

**No matches found?**
- Try a shorter or more general query
- Check if case sensitivity is enabled (toggle with Ctrl+Shift+C)
- Verify you're searching the correct buffer content

**Slow performance with large buffers?**
- Use more specific queries to reduce result count
- Consider using prefix/substring modes for faster matching

---

Buffer Fuzzy Search is powered by [nucleo-matcher](https://github.com/helix-editor/nucleo), providing fast and intelligent fuzzy matching optimized for terminal workflows.
.
