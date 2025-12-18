#[derive(Debug, Default)]
pub struct SelectionState {
    index: Option<usize>,
    count: usize,
}

impl SelectionState {
    pub fn update_count(&mut self, count: usize) {
        self.count = count;
        if count == 0 {
            self.index = None;
        } else if let Some(idx) = self.index {
            if idx >= count {
                self.index = Some(count.saturating_sub(1));
            }
        }
    }

    pub fn select_first(&mut self) {
        if self.count > 0 {
            self.index = Some(0);
        }
    }

    pub fn move_up(&mut self) {
        if self.count == 0 {
            return;
        }

        self.index = Some(match self.index {
            Some(0) => self.count.saturating_sub(1),
            Some(idx) => idx.saturating_sub(1),
            None => self.count.saturating_sub(1),
        });
    }

    pub fn move_down(&mut self) {
        if self.count == 0 {
            return;
        }

        self.index = Some(match self.index {
            Some(idx) if idx >= self.count.saturating_sub(1) => 0,
            Some(idx) => idx + 1,
            None => 0,
        });
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn clear(&mut self) {
        self.index = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_up_wraps_around() {
        let mut state = SelectionState::default();
        state.update_count(3);
        state.index = Some(0);
        state.move_up();
        assert_eq!(state.index, Some(2));
    }

    #[test]
    fn move_down_wraps_around() {
        let mut state = SelectionState::default();
        state.update_count(3);
        state.index = Some(2);
        state.move_down();
        assert_eq!(state.index, Some(0));
    }

    #[test]
    fn update_count_clamps_index() {
        let mut state = SelectionState::default();
        state.update_count(5);
        state.index = Some(4);
        state.update_count(2);
        assert_eq!(state.index, Some(1));
    }
}
