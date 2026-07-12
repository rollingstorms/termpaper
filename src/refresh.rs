use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCell {
    pub text: String,
    pub bold: bool,
    pub underline: bool,
    pub inverse: bool,
    pub foreground: TerminalColor,
    pub background: TerminalColor,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TerminalColor {
    #[default]
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSnapshot {
    pub rows: u16,
    pub columns: u16,
    pub cells: Vec<RenderCell>,
    pub cursor_row: u16,
    pub cursor_column: u16,
    pub cursor_visible: bool,
    pub alternate_screen: bool,
}

impl RenderSnapshot {
    pub fn diff(&self, next: &RenderSnapshot) -> Vec<DirtyRect> {
        if self.rows != next.rows || self.columns != next.columns {
            return vec![DirtyRect::full(next.columns, next.rows)];
        }

        let mut rects = Vec::new();
        for row in 0..next.rows {
            let mut start = None;
            for column in 0..next.columns {
                let idx = (row as usize * next.columns as usize) + column as usize;
                if self.cells[idx] != next.cells[idx] {
                    start.get_or_insert(column);
                } else if let Some(left) = start.take() {
                    append_or_extend_rect(
                        &mut rects,
                        DirtyRect {
                            x: left,
                            y: row,
                            width: column - left,
                            height: 1,
                        },
                    );
                }
            }
            if let Some(left) = start {
                append_or_extend_rect(
                    &mut rects,
                    DirtyRect {
                        x: left,
                        y: row,
                        width: next.columns - left,
                        height: 1,
                    },
                );
            }
        }

        rects
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl DirtyRect {
    pub fn full(columns: u16, rows: u16) -> Self {
        Self {
            x: 0,
            y: 0,
            width: columns,
            height: rows,
        }
    }
}

pub fn coalesce_dirty_rects(rects: &mut Vec<DirtyRect>) {
    rects.retain(|rect| rect.width > 0 && rect.height > 0);
    rects.sort_by_key(|rect| (rect.x, rect.width, rect.y, rect.height));

    let mut merged = Vec::with_capacity(rects.len());
    for rect in rects.drain(..) {
        append_or_extend_rect(&mut merged, rect);
    }
    *rects = merged;
}

fn append_or_extend_rect(rects: &mut Vec<DirtyRect>, rect: DirtyRect) {
    if let Some(last) = rects.last_mut() {
        if last.x == rect.x
            && last.width == rect.width
            && last.y + last.height == rect.y
            && rect.height == 1
        {
            last.height += 1;
            return;
        }
    }

    rects.push(rect);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefreshPriority {
    Low,
    Normal,
    High,
}

#[derive(Debug)]
pub struct RefreshScheduler {
    min_interval: Duration,
    high_priority_bypasses_rate_limit: bool,
    last_refresh: Option<Instant>,
    pending_priority: Option<RefreshPriority>,
}

impl RefreshScheduler {
    pub fn new(min_interval: Duration) -> Self {
        Self::with_policy(min_interval, true)
    }

    pub fn with_policy(min_interval: Duration, high_priority_bypasses_rate_limit: bool) -> Self {
        Self {
            min_interval,
            high_priority_bypasses_rate_limit,
            last_refresh: None,
            pending_priority: None,
        }
    }

    pub fn record_mutation(&mut self, priority: RefreshPriority) {
        self.pending_priority = Some(self.pending_priority.map_or(priority, |p| p.max(priority)));
    }

    pub fn should_refresh(&mut self, now: Instant) -> bool {
        let Some(priority) = self.pending_priority else {
            return false;
        };

        let due = match (priority, self.last_refresh) {
            (RefreshPriority::High, _) if self.high_priority_bypasses_rate_limit => true,
            (_, None) => true,
            (_, Some(last)) => now.duration_since(last) >= self.min_interval,
        };

        if due {
            self.last_refresh = Some(now);
            self.pending_priority = None;
        }

        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(text: &str) -> RenderCell {
        RenderCell {
            text: text.to_string(),
            bold: false,
            underline: false,
            inverse: false,
            foreground: TerminalColor::Default,
            background: TerminalColor::Default,
        }
    }

    #[test]
    fn diff_merges_neighboring_dirty_cells_per_row() {
        let previous = RenderSnapshot {
            rows: 1,
            columns: 5,
            cells: vec![cell("a"), cell("b"), cell("c"), cell("d"), cell("e")],
            cursor_row: 0,
            cursor_column: 0,
            cursor_visible: true,
            alternate_screen: false,
        };
        let next = RenderSnapshot {
            rows: 1,
            columns: 5,
            cells: vec![cell("a"), cell("x"), cell("y"), cell("d"), cell("z")],
            ..previous.clone()
        };

        assert_eq!(
            previous.diff(&next),
            vec![
                DirtyRect {
                    x: 1,
                    y: 0,
                    width: 2,
                    height: 1
                },
                DirtyRect {
                    x: 4,
                    y: 0,
                    width: 1,
                    height: 1
                }
            ]
        );
    }

    #[test]
    fn diff_merges_identical_spans_across_adjacent_rows() {
        let previous = RenderSnapshot {
            rows: 3,
            columns: 4,
            cells: vec![cell("a"); 12],
            cursor_row: 0,
            cursor_column: 0,
            cursor_visible: true,
            alternate_screen: false,
        };
        let mut cells = previous.cells.clone();
        cells[1] = cell("x");
        cells[2] = cell("x");
        cells[5] = cell("x");
        cells[6] = cell("x");
        let next = RenderSnapshot {
            cells,
            ..previous.clone()
        };

        assert_eq!(
            previous.diff(&next),
            vec![DirtyRect {
                x: 1,
                y: 0,
                width: 2,
                height: 2
            }]
        );
    }

    #[test]
    fn coalesces_pending_rects_from_multiple_mutations() {
        let mut rects = vec![
            DirtyRect {
                x: 0,
                y: 1,
                width: 3,
                height: 1,
            },
            DirtyRect {
                x: 0,
                y: 0,
                width: 3,
                height: 1,
            },
            DirtyRect {
                x: 5,
                y: 0,
                width: 1,
                height: 1,
            },
        ];

        coalesce_dirty_rects(&mut rects);

        assert_eq!(
            rects,
            vec![
                DirtyRect {
                    x: 0,
                    y: 0,
                    width: 3,
                    height: 2,
                },
                DirtyRect {
                    x: 5,
                    y: 0,
                    width: 1,
                    height: 1,
                }
            ]
        );
    }

    #[test]
    fn high_priority_refresh_bypasses_rate_limit() {
        let mut scheduler = RefreshScheduler::new(Duration::from_secs(10));
        let now = Instant::now();
        scheduler.record_mutation(RefreshPriority::Normal);
        assert!(scheduler.should_refresh(now));
        scheduler.record_mutation(RefreshPriority::Normal);
        assert!(!scheduler.should_refresh(now + Duration::from_secs(1)));
        scheduler.record_mutation(RefreshPriority::High);
        assert!(scheduler.should_refresh(now + Duration::from_secs(1)));
    }

    #[test]
    fn high_priority_can_respect_rate_limit_for_slow_displays() {
        let mut scheduler = RefreshScheduler::with_policy(Duration::from_secs(10), false);
        let now = Instant::now();
        scheduler.record_mutation(RefreshPriority::Normal);
        assert!(scheduler.should_refresh(now));
        scheduler.record_mutation(RefreshPriority::High);
        assert!(!scheduler.should_refresh(now + Duration::from_secs(1)));
        assert!(scheduler.should_refresh(now + Duration::from_secs(10)));
    }
}
