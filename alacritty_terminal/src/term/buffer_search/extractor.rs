//! Extract searchable content from terminal buffer.

use crate::grid::{Dimensions, Grid};
use crate::index::{Column, Line};
use crate::term::cell::Cell;

use super::matcher::SearchLine;

/// 从终端 Grid 提取可搜索内容（纯文本，用于显示）
pub struct BufferExtractor;

impl BufferExtractor {
    pub fn extract(grid: &Grid<Cell>) -> Vec<SearchLine> {
        let mut lines = Vec::new();
        let screen_lines = grid.screen_lines();
        let total_lines = grid.total_lines();
        
        // Calculate scrollback lines count.
        let scrollback_lines = total_lines.saturating_sub(screen_lines);

        // Iterate through all lines including scrollback.
        // Line indices: scrollback lines are negative, viewport lines are 0..screen_lines
        for line_idx in 0..total_lines {
            // Convert to Line coordinate: scrollback lines use negative indices
            let line_offset = line_idx as i32 - scrollback_lines as i32;
            let line = Line(line_offset);
            
            let mut content = String::new();

            // 遍历该行的所有列
            for col in 0..grid.columns() {
                let cell = &grid[line][Column(col)];
                
                // 跳过尾部的空字符，但保留中间的空格
                if cell.c != ' ' || !content.is_empty() {
                    content.push(cell.c);
                }
            }

            // 只添加非空行
            let trimmed = content.trim_end();
            if !trimmed.is_empty() {
                // Store the absolute line index (0-based from top of scrollback)
                lines.push(SearchLine {
                    line_number: line_idx,
                    content: trimmed.to_string(),
                });
            }
        }

        lines
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    // TC-EXTRACT-001: 基本文本提取
    #[test]
    fn test_basic_extraction() {
        // 创建一个简单的 Grid 用于测试
        let mut grid: Grid<Cell> = Grid::new(3, 10, 0); // 3 行 x10 列
        
        // 填充一些测试内容
        for (i, c) in "Hello".chars().enumerate() {
            grid[Line(0)][Column(i)].c = c;
        }
        for (i, c) in "World".chars().enumerate() {
            grid[Line(1)][Column(i)].c = c;
        }
        
        let lines = BufferExtractor::extract(&grid);
        
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].content, "Hello");
        assert_eq!(lines[1].content, "World");
    }

    // TC-EXTRACT-002: 空行过滤
    #[test]
    fn test_empty_line_filtering() {
        let mut grid: Grid<Cell> = Grid::new(3, 10, 0);
        
        // 第 0 行有内容
        for (i, c) in "Test".chars().enumerate() {
            grid[Line(0)][Column(i)].c = c;
        }
        // 第 1 行为空
        // 第 2 行有内容
        for (i, c) in "Data".chars().enumerate() {
            grid[Line(2)][Column(i)].c = c;
        }
        
        let lines = BufferExtractor::extract(&grid);
        
        // 空行应被过滤
        assert_eq!(lines.len(), 2);
    }

    // TC-EXTRACT-003: 尾部空格处理
    #[test]
    fn test_trailing_space_handling() {
        let mut grid: Grid<Cell> = Grid::new(1, 10, 0);
        
        // 填充内容加空格
        let text = "Test   "; // 尾部有空格
        for (i, c) in text.chars().enumerate() {
            grid[Line(0)][Column(i)].c = c;
        }
        
        let lines = BufferExtractor::extract(&grid);
        
        assert_eq!(lines.len(), 1);
        // 尾部空格应被 trim
        assert_eq!(lines[0].content, "Test");
    }

    // TC-EXTRACT-004: 中间空格保留
    #[test]
    fn test_middle_space_preservation() {
        let mut grid: Grid<Cell> = Grid::new(1, 20, 0);
        
        // 填充带中间空格的内容
        let text = "Hello World Test";
        for (i, c) in text.chars().enumerate() {
            grid[Line(0)][Column(i)].c = c;
        }
        
        let lines = BufferExtractor::extract(&grid);
        
        assert_eq!(lines.len(), 1);
        // 中间空格应保留
        assert_eq!(lines[0].content, "Hello World Test");
    }

    // TC-EXTRACT-005: 跨 scrollback 提取
    #[test]
    fn test_scrollback_extraction() {
        // 创建带 scrollback 的 Grid
        let mut grid: Grid<Cell> = Grid::new(5, 10, 10); // 5 行屏幕，10 行 scrollback
        
        // 填充多行内容
        for row in 0..5 {
            let text = format!("Line{}", row);
            for (col, c) in text.chars().enumerate() {
                grid[Line(row as i32)][Column(col)].c = c;
            }
        }
        
        let lines = BufferExtractor::extract(&grid);
        
        // 应能提取所有非空行
        assert!(lines.len() >= 5);
    }

    // TC-EXTRACT-006: 全空 Grid
    #[test]
    fn test_all_empty_grid() {
        let grid: Grid<Cell> = Grid::new(3, 10, 0);
        
        let lines = BufferExtractor::extract(&grid);
        
        // 全空 Grid 应返回空列表
        assert_eq!(lines.len(), 0);
    }

    // TC-EXTRACT-007: 特殊字符提取
    #[test]
    fn test_special_char_extraction() {
        let mut grid: Grid<Cell> = Grid::new(1, 20, 0);
        
        // 填充特殊字符
        let text = "test@example.com";
        for (i, c) in text.chars().enumerate() {
            grid[Line(0)][Column(i)].c = c;
        }
        
        let lines = BufferExtractor::extract(&grid);
        
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].content, "test@example.com");
    }

    // TC-EXTRACT-008: Unicode 字符提取
    #[test]
    fn test_unicode_extraction() {
        let mut grid: Grid<Cell> = Grid::new(1, 20, 0);
        
        // 填充 Unicode 字符
        let text = "中文测试";
        for (i, c) in text.chars().enumerate() {
            grid[Line(0)][Column(i)].c = c;
        }
        
        let lines = BufferExtractor::extract(&grid);
        
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].content, "中文测试");
    }

    // TC-EXTRACT-009: 行号索引正确性
    #[test]
    fn test_line_number_indexing() {
        let mut grid: Grid<Cell> = Grid::new(3, 10, 0);
        
        // 填充 3 行内容
        for row in 0..3 {
            let text = format!("R{}", row);
            for (col, c) in text.chars().enumerate() {
                grid[Line(row as i32)][Column(col)].c = c;
            }
        }
        
        let lines = BufferExtractor::extract(&grid);
        
        // 验证行号索引
        assert_eq!(lines.len(), 3);
        for (i, line) in lines.iter().enumerate() {
            assert_eq!(line.line_number, i);
        }
    }

    // TC-EXTRACT-010: 内容长度验证
    #[test]
    fn test_content_length() {
        let mut grid: Grid<Cell> = Grid::new(1, 10, 0);
        
        // 填充内容
        let text = "1234567890";
        for (i, c) in text.chars().enumerate() {
            grid[Line(0)][Column(i)].c = c;
        }
        
        let lines = BufferExtractor::extract(&grid);
        
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].content.len(), 10);
    }
}
