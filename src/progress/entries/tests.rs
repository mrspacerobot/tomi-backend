//! Tests of what the rules make of imperfect cook entries: abandoned cooks, entries from a newer
//! catalog, copies and odd orders. Ported from TomiCore's `ProgressLogTests`, plus the server's own
//! promises from issue #36.

use crate::progress::fixtures::{
    complete_per_level, cook, cooks, cooks_per_level, stopped_before_the_end, with_the_quiz_wrong, TestLog,
};
use crate::progress::{earned_xp, open_levels, CookCounts, CookEntry, Rank, SkillStatus};

#[test]
fn cooks_stopped_early_do_not_count() {
    let log = TestLog::new();
    let finished = cook("knife", 1);
    let stopped = stopped_before_the_end(&cook("knife", 1));
    let knife = log.progress_of("knife", &[finished, stopped]);
    assert_eq!(cooks_per_level(&knife), [1, 0]);
    assert_eq!(complete_per_level(&knife), [false, false]);
}

#[test]
fn entries_for_a_skill_the_catalog_does_not_know_are_ignored() {
    let log = TestLog::new();
    let only_knife = cooks(1, "knife", 1);
    let entries = [cooks(2, "sous-vide", 1), only_knife.clone()].concat();
    assert_eq!(log.progress(&entries), log.progress(&only_knife));
}

#[test]
fn entries_for_a_level_the_catalog_does_not_know_are_ignored() {
    let log = TestLog::new();
    let entries = [cooks(2, "knife", 3), cooks(2, "knife", 0)].concat();
    let knife = log.progress_of("knife", &entries);
    assert_eq!(cooks_per_level(&knife), [0, 0]);
    assert_eq!(knife.status(), SkillStatus::Served);
}

/// The app's entries name a lesson the library may not know; here an entry names no dish at all,
/// only its own skill and level, which is what counts. Two separate cooks of new dishes count twice.
#[test]
fn cooks_of_lessons_the_library_does_not_know_count_at_their_skill_and_level() {
    let log = TestLog::new();
    let knife = log.progress_of("knife", &[cook("knife", 1), cook("knife", 1)]);
    assert_eq!(cooks_per_level(&knife), [2, 0]);
    assert_eq!(knife.current_level(), 2);
}

#[test]
fn the_same_entry_twice_counts_once() {
    let entry = cook("knife", 1);
    let knife = TestLog::new().progress_of("knife", &[entry.clone(), entry]);
    assert_eq!(cooks_per_level(&knife), [1, 0]);
}

#[test]
fn merging_two_overlapping_logs_equals_one_log() {
    let log = TestLog::new();
    let shared = cooks(1, "knife", 1);
    let only_on_phone = [cooks(1, "knife", 1), cooks(1, "eggs", 1)].concat();
    let only_on_tablet = cooks(2, "roux", 1);
    let phone = [shared.clone(), only_on_phone.clone()].concat();
    let tablet = [shared.clone(), only_on_tablet.clone()].concat();
    let one_log = [shared, only_on_phone, only_on_tablet].concat();
    assert_eq!(log.progress(&[phone.clone(), tablet.clone()].concat()), log.progress(&one_log));
    assert_eq!(log.progress(&[tablet.clone(), phone.clone()].concat()), log.progress(&one_log));
    assert_eq!(cooks_per_level(&log.progress_of("knife", &[phone, tablet].concat())), [2, 0]);
}

#[test]
fn the_order_of_entries_does_not_matter() {
    let log = TestLog::new();
    let entries = log.cooks_completing_everything();
    let reversed: Vec<_> = entries.iter().rev().cloned().collect();
    assert_eq!(log.progress(&reversed), log.progress(&entries));
}

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

fn stored_row() -> crate::progress::StoredEntry {
    crate::progress::StoredEntry {
        id: uuid::Uuid::new_v4(),
        skill: "eggs".to_string(),
        level: 2,
        outcome: "finished".to_string(),
        weekly_goal: 4,
        cooked_at_microseconds: 1_792_521_720_000_000,
        time_zone: "Asia/Tokyo".to_string(),
    }
}

#[test]
fn a_stored_row_reads_back_as_the_cook_it_was_with_no_quiz_bonus_until_answers_are_recorded() {
    let row = stored_row();
    let entry = row.read().unwrap();
    assert_eq!((entry.id, entry.skill.as_str(), entry.level, entry.weekly_goal), (row.id, "eggs", 2, 4));
    assert_eq!(entry.outcome, crate::progress::Outcome::Finished { quiz_passed: false });
    assert_eq!(entry.cooked_at, jiff::Timestamp::from_second(1_792_521_720).unwrap());
    assert_eq!(entry.time_zone.iana_name(), Some("Asia/Tokyo"));
    let stopped = crate::progress::StoredEntry { outcome: "stoppedEarly".to_string(), ..stored_row() };
    assert_eq!(stopped.read().unwrap().outcome, crate::progress::Outcome::StoppedEarly);
}

#[test]
fn a_stored_row_the_rules_cannot_read_is_refused_never_left_out() {
    let unreadable = [
        crate::progress::StoredEntry { skill: "Not an id".to_string(), ..stored_row() },
        crate::progress::StoredEntry { level: 300, ..stored_row() },
        crate::progress::StoredEntry { outcome: "abandoned".to_string(), ..stored_row() },
        crate::progress::StoredEntry { weekly_goal: -1, ..stored_row() },
        crate::progress::StoredEntry { time_zone: "Mars/Olympus_Mons".to_string(), ..stored_row() },
        crate::progress::StoredEntry { time_zone: "+02:00".to_string(), ..stored_row() },
    ];
    for row in unreadable {
        assert_eq!(row.read(), Err(crate::progress::UnreadableStoredEntry), "{row:?}");
    }
}
