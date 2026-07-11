use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCell {
    pub text: String,
    pub bold: bool,
    pub underline: bool,
    pub inverse: bool,
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
                    rects.push(DirtyRect {
                        x: left,
                        y: row,
                        width: column - left,
                        height: 1,
                    });
                }
            }
            if let Some(left) = start {
                rects.push(DirtyRect {
                    x: left,
                    y: row,
                    width: next.columns - left,
                    height: 1,
                });
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefreshPriority {
    Low,
    Normal,
    High,
}

#[derive(Debug)]
pub struct RefreshScheduler {
    min_interval: Duration,
    last_refresh: Option<Instant>,
    pending_priority: Option<RefreshPriority>,
}

impl RefreshScheduler {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            min_interval,
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
            (RefreshPriority::High, _) => true,
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
}
