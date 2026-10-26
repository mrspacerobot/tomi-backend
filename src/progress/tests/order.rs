//! The server's own promises about a log: arrival order never matters, an unknown skill or level
//! earns XP and opens nothing, and a cook stopped early is worth nothing anywhere.

use super::test_log::{cook, cooks, stopped_before_the_end, with_the_quiz_wrong, TestLog};
use crate::progress::{earned_xp, open_levels, CookCounts, CookEntry, Rank};

/// A log with a bit of everything: copies, a stopped cook, a failed quiz and an unknown skill.
fn mixed_log() -> Vec<CookEntry> {
    let copied = cook("knife", 1);
    let quiz_wrong = with_the_quiz_wrong(&cook("eggs", 1));
    let stopped = stopped_before_the_end(&cook("knife", 1));
    let unknown = cook("sous-vide", 9);
    let more = [cooks(2, "knife", 2), cooks(1, "roux", 1)].concat();
    [vec![copied.clone(), quiz_wrong, stopped, copied, unknown], more].concat()
}

#[test]
fn the_same_log_in_two_orders_gives_the_same_xp_rank_and_skill_progress() {
    let log = TestLog::new();
    let entries = mixed_log();
    let reversed: Vec<CookEntry> = entries.iter().rev().cloned().collect();
    assert_eq!(earned_xp(&reversed), earned_xp(&entries));
    assert_eq!(Rank::for_xp(earned_xp(&reversed)), Rank::for_xp(earned_xp(&entries)));
    assert_eq!(log.progress(&reversed), log.progress(&entries));
    let mut rotated = entries.clone();
    rotated.rotate_left(3);
    assert_eq!(log.progress(&rotated), log.progress(&entries));
    assert_eq!(earned_xp(&rotated), earned_xp(&entries));
}

#[test]
fn an_unknown_skill_or_level_earns_xp_and_opens_nothing() {
    let log = TestLog::new();
    let unknown = [cooks(2, "sous-vide", 1), cooks(2, "knife", 9)].concat();
    assert_eq!(earned_xp(&unknown), 4 * 60);
    assert_eq!(log.progress(&unknown), log.progress(&[]));
    let counts = CookCounts::from_log(&unknown);
    assert_eq!(
        open_levels(&log.catalog, log.experience, &counts),
        open_levels(&log.catalog, log.experience, &CookCounts::default())
    );
}

#[test]
fn a_stopped_early_cook_earns_nothing_and_counts_for_no_level() {
    let log = TestLog::new();
    let stopped: Vec<CookEntry> = cooks(2, "knife", 1).iter().map(stopped_before_the_end).collect();
    assert_eq!(earned_xp(&stopped), 0);
    assert_eq!(log.progress(&stopped), log.progress(&[]));
    assert_eq!(CookCounts::from_log(&stopped), CookCounts::default());
}

#[test]
fn a_finished_cook_earns_fifty_or_sixty_with_the_quiz_passed() {
    assert_eq!(earned_xp(&[with_the_quiz_wrong(&cook("knife", 1))]), 50);
    assert_eq!(earned_xp(&[cook("knife", 1)]), 60);
}
