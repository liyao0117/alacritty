//! Buffer Search matcher using nucleo-matcher.
//! Uses nucleo-matcher's native pattern syntax:
//! - `foo` (default): fuzzy match
//! - `'foo`: substring match (contiguous)
//! - `^foo`: prefix match
//! - `foo$`: postfix/suffix match
//! - `^foo$`: exact match

use nucleo_matcher::{Config, Matcher, Utf32Str};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};

/// Search configuration from config file
#[derive(Debug, Clone, Copy)]
pub struct SearchConfig {
    pub case_sensitive: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            case_sensitive: false,
        }
    }
}

/// Configure nucleo matcher
pub fn create_matcher() -> Matcher {
    Matcher::new(Config::DEFAULT)
}

/// Execute search query using nucleo-matcher's native pattern parsing
/// The pattern syntax is automatically detected:
/// - `^prefix`: prefix match
/// - `suffix$`: postfix match
/// - `'substring`: substring match (contiguous)
/// - `^exact$`: exact match
/// - `default`: fuzzy match
pub fn match_query(
    matcher: &mut Matcher,
    query: &str,
    lines: &[SearchLine],
    config: &SearchConfig,
) -> Vec<MatchResult> {
    if query.is_empty() {
        return Vec::new();
    }

    let case_matching = if config.case_sensitive {
        CaseMatching::Respect
    } else {
        CaseMatching::Ignore
    };
    
    let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
    let mut results = Vec::new();

    for line in lines {
        let mut buf = Vec::new();
        let haystack = Utf32Str::new(&line.content, &mut buf);
        
        let mut indices = Vec::new();
        let score = pattern.indices(haystack, matcher, &mut indices);
        
        if !indices.is_empty() {
            results.push(MatchResult {
                line_number: line.line_number,
                content: line.content.clone(),
                score: score.unwrap_or(0),
                highlight_ranges: indices.into_iter().map(|i| i as usize).collect(),
            });
        }
    }

    // Sort by score (higher first)
    results.sort_by(|a, b| b.score.cmp(&a.score));

    results
}

/// Search line
#[derive(Debug, Clone)]
pub struct SearchLine {
    pub line_number: usize,
    pub content: String,
}

/// Match result
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub line_number: usize,
    pub content: String,
    pub score: u32,
    pub highlight_ranges: Vec<usize>,
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // TC-MATCHER-001: nucleo 模糊匹配集成
    #[test]
    fn test_nucleo_fuzzy_matching() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "fuzzy search test".to_string() },
            SearchLine { line_number: 1, content: "no match".to_string() },
        ];
        let config = SearchConfig::default();
        
        let results = match_query(&mut matcher, "fz", &lines, &config);
        
        // "fz" 应匹配 "fuzzy search test"
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 0);
    }

    // TC-MATCHER-002: 匹配得分排序
    #[test]
    fn test_match_score_ordering() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "fuzzy".to_string() },
            SearchLine { line_number: 1, content: "foo bar baz".to_string() },
            SearchLine { line_number: 2, content: "fzb".to_string() },
        ];
        let config = SearchConfig::default();
        
        let results = match_query(&mut matcher, "fb", &lines, &config);
        
        // 结果应按得分降序排列
        assert!(results.len() >= 2);
        for i in 1..results.len() {
            assert!(results[i - 1].score >= results[i].score);
        }
    }

    // TC-MATCHER-003: 特殊字符查询
    #[test]
    fn test_special_char_query() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "test@example.com".to_string() },
            SearchLine { line_number: 1, content: "user name".to_string() },
        ];
        let config = SearchConfig::default();
        
        // 特殊字符应作为普通字符处理
        let results = match_query(&mut matcher, "@", &lines, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 0);
    }

    // TC-MATCHER-004: Unicode 匹配
    #[test]
    fn test_unicode_matching() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "中文测试".to_string() },
            SearchLine { line_number: 1, content: "English text".to_string() },
            SearchLine { line_number: 2, content: "日本語テスト".to_string() },
        ];
        let config = SearchConfig::default();
        
        let results = match_query(&mut matcher, "中文", &lines, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 0);
    }

    // TC-MATCHER-005: 子串匹配模式
    #[test]
    fn test_substring_match_mode() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "fuzzy search".to_string() },
            SearchLine { line_number: 1, content: "fuzsearch".to_string() },
        ];
        let config = SearchConfig::default();
        
        // "'fuzz" 应只匹配包含连续 "fuzz" 的行
        let results = match_query(&mut matcher, "'fuzz", &lines, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 0);
    }

    // TC-MATCHER-006: 前缀匹配模式
    #[test]
    fn test_prefix_match_mode() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "git commit".to_string() },
            SearchLine { line_number: 1, content: "commit git".to_string() },
        ];
        let config = SearchConfig::default();
        
        // "^git" 应只匹配以 "git" 开头的行
        let results = match_query(&mut matcher, "^git", &lines, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 0);
    }

    // TC-MATCHER-007: 后缀匹配模式
    #[test]
    fn test_suffix_match_mode() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "main.rs".to_string() },
            SearchLine { line_number: 1, content: "rs.main".to_string() },
        ];
        let config = SearchConfig::default();
        
        // "rs$" 应只匹配以 "rs" 结尾的行
        let results = match_query(&mut matcher, "rs$", &lines, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 0);
    }

    // TC-MATCHER-008: 精确匹配模式
    #[test]
    fn test_exact_match_mode() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "foo".to_string() },
            SearchLine { line_number: 1, content: "foo bar".to_string() },
            SearchLine { line_number: 2, content: "bar foo".to_string() },
        ];
        let config = SearchConfig::default();
        
        // "^foo$" 应只匹配完全等于 "foo" 的行
        let results = match_query(&mut matcher, "^foo$", &lines, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 0);
    }

    // TC-MATCHER-009: 大小写敏感配置
    #[test]
    fn test_case_sensitive_config() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "Hello".to_string() },
            SearchLine { line_number: 1, content: "hello".to_string() },
        ];
        
        // 不区分大小写
        let config_insensitive = SearchConfig { case_sensitive: false };
        let results = match_query(&mut matcher, "hello", &lines, &config_insensitive);
        assert_eq!(results.len(), 2);
        
        // 区分大小写
        let config_sensitive = SearchConfig { case_sensitive: true };
        let results = match_query(&mut matcher, "hello", &lines, &config_sensitive);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 1);
    }

    // TC-MATCHER-010: 空查询处理
    #[test]
    fn test_empty_query() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "test".to_string() },
        ];
        let config = SearchConfig::default();
        
        let results = match_query(&mut matcher, "", &lines, &config);
        assert_eq!(results.len(), 0);
    }

    // TC-MATCHER-011: 无匹配结果
    #[test]
    fn test_no_match() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "hello world".to_string() },
        ];
        let config = SearchConfig::default();
        
        let results = match_query(&mut matcher, "xyz123", &lines, &config);
        assert_eq!(results.len(), 0);
    }

    // TC-MATCHER-012: 高亮范围计算
    #[test]
    fn test_highlight_ranges() {
        let mut matcher = create_matcher();
        let lines = vec![
            SearchLine { line_number: 0, content: "fuzzy".to_string() },
        ];
        let config = SearchConfig::default();
        
        let results = match_query(&mut matcher, "fz", &lines, &config);
        assert_eq!(results.len(), 1);
        assert!(!results[0].highlight_ranges.is_empty());
    }
}
