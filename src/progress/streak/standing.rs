//! Where the cook stands in their weekly streak at one moment. Computed from the cook log, never stored.

/// The weekly streak right now: met weeks in a row before this one, and how this week is going.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Streak {
    /// Consecutive weeks with the goal met, up to the end of last week.
    pub weeks_before_this_week: u32,
    /// Days this week with at least one finished cook.
    pub days_this_week: u32,
    /// This week's goal in cooking days.
    pub goal: u8,
    /// Days left in this week, today included.
    pub days_left_in_week: u32,
}

impl Streak {
    /// Consecutive weeks with the goal met. The week in progress adds one once it is met, and never
    /// breaks the streak before it ends.
    pub fn weeks(&self) -> u32 {
        if self.is_this_week_met() {
            return self.weeks_before_this_week + 1;
        }
        self.weeks_before_this_week
    }

    pub fn is_this_week_met(&self) -> bool {
        self.days_this_week >= u32::from(self.goal)
    }

    /// Still possible, but only by cooking on every day left.
    pub fn is_at_risk(&self) -> bool {
        !self.is_this_week_met() && self.days_still_needed() == self.days_left_in_week
    }

    /// Too few days left to meet the goal. The streak ends when the week does.
    pub fn is_lost(&self) -> bool {
        self.days_still_needed() > self.days_left_in_week
    }

    fn days_still_needed(&self) -> u32 {
        u32::from(self.goal).saturating_sub(self.days_this_week)
    }
}
