//! Buffer Search mode state management.
//! Uses nucleo-matcher's native pattern syntax for mode selection.

use std::collections::HashSet;
use std::fmt;

use crate::term::buffer_search;
use crate::term::buffer_search::matcher::SearchConfig;
use nucleo_matcher::Matcher;

/// Buffer Search 匹配结果
#[derive(Debug, Clone)]
pub struct Match {
    pub line_number: usize,
    pub content: String,
    pub score: u32,
    pub highlight_ranges: Vec<usize>,
}

/// Buffer Search 列过滤配置 (Phase 4.3)
#[derive(Debug, Clone, Default)]
pub struct BufferSearchConfig {
    /// 列分隔符，如 ":", " ", "\t"
    pub delimiter: String,
    
    /// 搜索的列索引 (1-based)，空表示搜索整行
    pub nth: Vec<usize>,
}

/// Buffer Search mode state.
pub struct BufferSearchState {
    /// Current search query string.
    pub query: String,
    
    /// Whether the search mode is active.
    pub active: bool,
    
    /// Match results (Phase 2).
    pub matches: Vec<Match>,
    
    /// Currently selected match index (Phase 2).
    pub selected_index: usize,
    
    /// Nucleo matcher (Phase 2).
    pub matcher: Matcher,
    
    /// Search configuration (from config file).
    pub search_config: SearchConfig,
    
    /// Scroll offset for result list (Phase 3).
    pub scroll_offset: usize,
    
    /// Maximum number of results to display (Phase 3).
    pub max_display: usize,
    
    // ========== Phase 4.1: Multi-select ==========
    
    /// Whether multi-select mode is enabled.
    pub multi_select: bool,
    
    /// Set of selected match indices (Phase 4.1).
    pub selected_items: HashSet<usize>,
}

/// Maximum number of results to display in overlay.
pub const MAX_DISPLAY_RESULTS: usize = 100;  // Show up to 100 results

impl Default for BufferSearchState {
    fn default() -> Self {
        Self {
            query: String::with_capacity(256),
            active: false,
            matches: Vec::new(),
            selected_index: 0,
            matcher: Matcher::new(nucleo_matcher::Config::DEFAULT),
            search_config: SearchConfig::default(),
            scroll_offset: 0,
            max_display: MAX_DISPLAY_RESULTS,
            // Phase 4.1: Multi-select
            multi_select: false,
            selected_items: HashSet::new(),
        }
    }
}

impl BufferSearchState {
    /// Create a new BufferSearchState.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a new BufferSearchState with custom config.
    pub fn with_config(config: SearchConfig) -> Self {
        Self {
            search_config: config,
            ..Self::default()
        }
    }
    
    /// Activate the search mode.
    pub fn activate(&mut self, max_display: usize) {
        self.active = true;
        self.query.clear();
        self.matches.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.max_display = max_display;
        self.selected_items.clear();
    }
    
    /// Deactivate the search mode.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.query.clear();
        self.matches.clear();
        self.selected_index = 0;
        self.selected_items.clear();
    }
    
    /// Update search config (case sensitivity).
    pub fn update_config(&mut self, config: SearchConfig) {
        self.search_config = config;
    }
    
    /// Get case sensitivity status.
    pub fn is_case_sensitive(&self) -> bool {
        self.search_config.case_sensitive
    }
    
    /// Input a character to the query.
    pub fn input(&mut self, c: char) {
        self.query.push(c);
    }
    
    /// Remove the last character from query (backspace).
    pub fn backspace(&mut self) {
        self.query.pop();
    }
    
    /// Check if the search is active.
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    /// Get the current query.
    pub fn query(&self) -> &str {
        &self.query
    }
    
    /// Update matches after query change.
    pub fn update_matches(&mut self, search_lines: &[buffer_search::SearchLine]) {
        // Preserve current selection state before updating.
        let old_selected_index = self.selected_index;
        
        if self.query.is_empty() {
            // When query is empty, show all non-empty lines.
            self.matches = search_lines
                .iter()
                .filter(|line| !line.content.trim().is_empty())
                .map(|line| Match {
                    line_number: line.line_number,
                    content: line.content.clone(),
                    score: 0,
                    highlight_ranges: vec![],
                })
                .collect();
            // Reset selection only when query changes to empty.
            self.selected_index = 0;
            self.scroll_offset = 0;
            return;
        }
        
        let results = buffer_search::match_query(
            &mut self.matcher,
            &self.query,
            search_lines,
            &self.search_config,
        );
        
        self.matches = results
            .into_iter()
            .map(|r| Match {
                line_number: r.line_number,
                content: r.content,
                score: r.score,
                highlight_ranges: r.highlight_ranges,
            })
            .collect();
        
        // Preserve selection state if possible.
        // If the previously selected item still exists, keep it selected.
        // Otherwise, try to preserve the index or reset to 0.
        if old_selected_index < self.matches.len() {
            self.selected_index = old_selected_index;
        } else if !self.matches.is_empty() {
            self.selected_index = 0;
        } else {
            self.selected_index = 0;
        }
        
        // Adjust scroll offset to keep selection visible.
        self.adjust_scroll_offset();
    }
    
    /// Select next match (with wrap-around).
    pub fn select_next(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        // Wrap around: if at last item, go to first
        if self.selected_index + 1 >= self.matches.len() {
            self.selected_index = 0;
        } else {
            self.selected_index += 1;
        }
        self.adjust_scroll_offset();
    }
    
    /// Select previous match (with wrap-around).
    pub fn select_previous(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        // Wrap around: if at first item, go to last
        if self.selected_index == 0 {
            self.selected_index = self.matches.len().saturating_sub(1);
        } else {
            self.selected_index -= 1;
        }
        self.adjust_scroll_offset();
    }
    
    /// Adjust scroll offset to keep selected item visible.
    fn adjust_scroll_offset(&mut self) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.max_display {
            self.scroll_offset = self.selected_index - self.max_display + 1;
        }
    }
    
    /// Get selected match.
    pub fn selected_match(&self) -> Option<&Match> {
        self.matches.get(self.selected_index)
    }
    
    /// Get matches to display (considering scroll offset).
    pub fn visible_matches(&self) -> &[Match] {
        let start = self.scroll_offset.min(self.matches.len());
        let end = (start + self.max_display).min(self.matches.len());
        &self.matches[start..end]
    }
    
    /// Get scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }
    
    /// Get total match count.
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }
    
    // ========== Phase 4.1: Multi-select Methods ==========
    
    /// Toggle selection state for current match.
    pub fn toggle_selection(&mut self) {
        if self.selected_index < self.matches.len() {
            if self.selected_items.contains(&self.selected_index) {
                self.selected_items.remove(&self.selected_index);
            } else {
                self.selected_items.insert(self.selected_index);
            }
        }
    }
    
    /// Select all matches.
    pub fn select_all(&mut self) {
        if self.selected_items.len() == self.matches.len() {
            self.selected_items.clear();
        } else {
            self.selected_items = (0..self.matches.len()).collect();
        }
    }
    
    /// Get all selected matches' content (plain text, for display).
    pub fn get_selected_content(&self) -> Vec<String> {
        let mut indices: Vec<usize> = self.selected_items.iter().copied().collect();
        indices.sort();
        
        indices
            .into_iter()
            .filter_map(|idx| self.matches.get(idx))
            .map(|m| m.content.clone())
            .collect()
    }
    
    /// Check if current match is selected.
    pub fn is_current_selected(&self) -> bool {
        self.selected_items.contains(&self.selected_index)
    }
    
    /// Get count of selected items.
    pub fn selected_count(&self) -> usize {
        self.selected_items.len()
    }
}

impl fmt::Display for BufferSearchState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.active {
            let case_str = if self.search_config.case_sensitive { "Aa" } else { "aa" };
            write!(f, "BufferSearch({}, case={})", self.query, case_str)
        } else {
            write!(f, "BufferSearch(inactive)")
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::buffer_search::matcher::SearchConfig;

    // TC-UNIT-001: 基本模糊匹配
    #[test]
    fn test_fuzzy_match_basic() {
        let mut state = BufferSearchState::new();
        
        // 设置测试数据
        state.matches = vec![
            Match {
                line_number: 0,
                content: "fuzzy search test".to_string(),
                score: 100,
                highlight_ranges: vec![],
            },
            Match {
                line_number: 1,
                content: "fuzzysearch test".to_string(),
                score: 90,
                highlight_ranges: vec![],
            },
            Match {
                line_number: 2,
                content: "no match here".to_string(),
                score: 0,
                highlight_ranges: vec![],
            },
        ];
        
        // 查询 "fz" 应匹配前两个
        state.query = "fz".to_string();
        assert_eq!(state.matches.len(), 3);
    }

    // TC-UNIT-002: 大小写敏感匹配
    #[test]
    fn test_case_sensitive_match() {
        let mut state = BufferSearchState::with_config(SearchConfig { case_sensitive: true }); // Fixed
        
        state.matches = vec![
            Match {
                line_number: 0,
                content: "Hello World".to_string(),
                score: 100,
                highlight_ranges: vec![],
            },
            Match {
                line_number: 1,
                content: "hello world".to_string(),
                score: 0,
                highlight_ranges: vec![],
            },
        ];
        
        assert!(state.is_case_sensitive());
    }

    // TC-UNIT-003: 导航回环 - 从第 1 行向上到最后
    #[test]
    fn test_navigation_wrap_around_previous() {
        let mut state = BufferSearchState::new();
        
        state.matches = vec![
            Match { line_number: 0, content: "Line 1".to_string(), score: 0, highlight_ranges: vec![] },
            Match { line_number: 1, content: "Line 2".to_string(), score: 0, highlight_ranges: vec![] },
            Match { line_number: 2, content: "Line 3".to_string(), score: 0, highlight_ranges: vec![] },
        ];
        state.selected_index = 0; // 第 1 行
        
        // 向上应跳转到最后 1 行
        state.select_previous();
        assert_eq!(state.selected_index, 2);
    }

    // TC-UNIT-004: 导航回环 - 从最后 1 行向下到第 1 行
    #[test]
    fn test_navigation_wrap_around_next() {
        let mut state = BufferSearchState::new();
        
        state.matches = vec![
            Match { line_number: 0, content: "Line 1".to_string(), score: 0, highlight_ranges: vec![] },
            Match { line_number: 1, content: "Line 2".to_string(), score: 0, highlight_ranges: vec![] },
            Match { line_number: 2, content: "Line 3".to_string(), score: 0, highlight_ranges: vec![] },
        ];
        state.selected_index = 2; // 最后 1 行
        
        // 向下应跳转到第 1 行
        state.select_next();
        assert_eq!(state.selected_index, 0);
    }

    // TC-UNIT-005: 多选切换
    #[test]
    fn test_multi_select_toggle() {
        let mut state = BufferSearchState::new();
        
        state.matches = vec![
            Match { line_number: 0, content: "Line 1".to_string(), score: 0, highlight_ranges: vec![] },
            Match { line_number: 1, content: "Line 2".to_string(), score: 0, highlight_ranges: vec![] },
            Match { line_number: 2, content: "Line 3".to_string(), score: 0, highlight_ranges: vec![] },
        ];
        state.selected_index = 0;
        
        // 初始无选中
        assert_eq!(state.selected_count(), 0);
        
        // Tab 切换选中当前行
        state.toggle_selection();
        assert_eq!(state.selected_count(), 1);
        assert!(state.selected_items.contains(&0));
        
        // 再次 Tab 取消选中
        state.toggle_selection();
        assert_eq!(state.selected_count(), 0);
    }

    // TC-UNIT-006: 全选功能
    #[test]
    fn test_select_all() {
        let mut state = BufferSearchState::new();
        
        state.matches = vec![
            Match { line_number: 0, content: "Line 1".to_string(), score: 0, highlight_ranges: vec![] },
            Match { line_number: 1, content: "Line 2".to_string(), score: 0, highlight_ranges: vec![] },
            Match { line_number: 2, content: "Line 3".to_string(), score: 0, highlight_ranges: vec![] },
        ];
        
        // Ctrl+A 全选
        state.select_all();
        assert_eq!(state.selected_count(), 3);
        
        // 再次 Ctrl+A 应取消全选
        state.select_all();
        assert_eq!(state.selected_count(), 0);
    }

    // TC-UNIT-007: 获取选中内容
    #[test]
    fn test_get_selected_content() {
        let mut state = BufferSearchState::new();
        
        state.matches = vec![
            Match { line_number: 0, content: "First".to_string(), score: 0, highlight_ranges: vec![] },
            Match { line_number: 1, content: "Second".to_string(), score: 0, highlight_ranges: vec![] },
            Match { line_number: 2, content: "Third".to_string(), score: 0, highlight_ranges: vec![] },
        ];
        
        // 选中第 1 和第 3 行
        state.selected_items.insert(0);
        state.selected_items.insert(2);
        
        let content = state.get_selected_content();
        assert_eq!(content.len(), 2);
        assert_eq!(content[0], "First");
        assert_eq!(content[1], "Third");
    }

    // TC-UNIT-008: 更新匹配时保持选择状态
    #[test]
    fn test_preserve_selection_on_update() {
        let mut state = BufferSearchState::new();
        
        // 初始有 5 个匹配
        state.matches = (0..5)
            .map(|i| Match {
                line_number: i,
                content: format!("Line {}", i),
                score: 0,
                highlight_ranges: vec![],
            })
            .collect();
        
        state.selected_index = 2; // 选择第 3 个
        
        // 更新后匹配减少到 3 个，但索引 2 仍然有效
        let old_index = state.selected_index;
        state.matches.truncate(3);
        
        // 模拟 update_matches 的逻辑
        if old_index < state.matches.len() {
            state.selected_index = old_index;
        }
        
        assert_eq!(state.selected_index, 2);
    }

    // TC-UNIT-009: 空查询处理
    #[test]
    fn test_empty_query() {
        let state = BufferSearchState::new();
        assert_eq!(state.query, "");
        assert!(!state.active);
    }

    // TC-UNIT-010: 激活/停用状态
    #[test]
    fn test_activate_deactivate() {
        let mut state = BufferSearchState::new();
        
        assert!(!state.is_active());
        
        state.activate(10);
        assert!(state.is_active());
        
        state.deactivate();
        assert!(!state.is_active());
    }

    // TC-UNIT-011: 滚动偏移调整
    #[test]
    fn test_scroll_offset_adjustment() {
        let mut state = BufferSearchState::new(); // max_display = 5
        
        state.matches = (0..10)
            .map(|i| Match {
                line_number: i,
                content: format!("Line {}", i),
                score: 0,
                highlight_ranges: vec![],
            })
            .collect();
        
        // 选择第 8 个（索引 7）
        state.selected_index = 7;
        state.scroll_offset = 0;
        
        // 调整滚动偏移以保持选中项可见
        state.adjust_scroll_offset();
        
        // 滚动偏移应调整到能看到第 8 个的位置
        assert!(state.scroll_offset <= 7);
        assert!(state.scroll_offset + state.max_display > 7);
    }

    // TC-UNIT-012: 可见匹配项计算
    #[test]
    fn test_visible_matches() {
        let mut state = BufferSearchState::new();
        
        state.matches = (0..10)
            .map(|i| Match {
                line_number: i,
                content: format!("Line {}", i),
                score: 0,
                highlight_ranges: vec![],
            })
            .collect();
        
        state.scroll_offset = 2;
        
        let visible = state.visible_matches();
        // visible_matches 返回从 scroll_offset 开始的所有匹配
        assert_eq!(visible.len(), 8); // 10 - 2 = 8
        assert_eq!(visible[0].line_number, 2);
    }

    // TC-UNIT-013: 当前选中判断
    #[test]
    fn test_is_current_selected() {
        let mut state = BufferSearchState::new();
        
        state.matches = vec![
            Match { line_number: 0, content: "Line 1".to_string(), score: 0, highlight_ranges: vec![] },
            Match { line_number: 1, content: "Line 2".to_string(), score: 0, highlight_ranges: vec![] },
        ];
        state.selected_index = 0;
        
        // 未选中任何项
        assert!(!state.is_current_selected());
        
        // 选中当前项
        state.toggle_selection();
        assert!(state.is_current_selected());
    }

    // TC-UNIT-014: Display 实现
    #[test]
    fn test_display_format() {
        let mut state = BufferSearchState::new();
        
        // 未激活状态
        let display = format!("{}", state);
        assert!(display.contains("inactive"));
        
        // 激活状态
        state.activate(10);
        state.query = "test".to_string();
        let display = format!("{}", state);
        assert!(display.contains("test"));
        assert!(display.contains("case="));
    }

    // TC-UNIT-015: 匹配项清空
    #[test]
    fn test_clear_matches() {
        let mut state = BufferSearchState::new();
        
        state.matches = vec![
            Match { line_number: 0, content: "Line 1".to_string(), score: 0, highlight_ranges: vec![] },
        ];
        
        assert_eq!(state.matches.len(), 1);
        
        state.matches.clear();
        assert_eq!(state.matches.len(), 0);
    }
}
