//! A cook's quiz result is derived from its recorded first answers: every question answered, every first answer
//! right, for the entry's own recipe, on a finished cook. Then the cook earns the quiz bonus.

use crate::progress::{earned_xp, CookEntry, QuizAnswers, StoredEntry};
use crate::quiz::RecordedAnswer;
use uuid::Uuid;

const RECIPE: &str = "recipe/soft-scrambled-eggs";
const QUESTIONS: [&str; 3] = ["quiz/purpose", "quiz/method", "quiz/observation"];

/// A finished cook of `RECIPE`, as the cook log stores it.
fn finished_cook() -> StoredEntry {
    StoredEntry {
        id: Uuid::new_v4(),
        skill: "eggs".to_string(),
        level: 1,
        recipe_id: Some(RECIPE.to_string()),
        outcome: "finished".to_string(),
        weekly_goal: 3,
        cooked_at_microseconds: 1_792_521_720_000_000,
        time_zone: "Europe/Amsterdam".to_string(),
    }
}

/// The first answer to a question of the three-question quiz of `RECIPE`, for this cook entry.
fn first_answer(entry: &StoredEntry, question: &str, correct: bool) -> RecordedAnswer {
    let quiz_question_ids = QUESTIONS.map(String::from).to_vec();
    let (entry_id, question_id, recipe_id) = (entry.id, question.to_string(), RECIPE.to_string());
    RecordedAnswer { entry_id, question_id, recipe_id, correct, quiz_question_ids }
}

/// A right first answer to every question of the quiz.
fn all_right(entry: &StoredEntry) -> Vec<RecordedAnswer> {
    QUESTIONS.iter().map(|question| first_answer(entry, question, true)).collect()
}

/// The XP of these stored cooks, read with these recorded first answers.
fn xp(entries: &[StoredEntry], answers: Vec<RecordedAnswer>) -> u32 {
    let answers = QuizAnswers::new(answers);
    let log: Vec<CookEntry> = entries.iter().map(|entry| entry.read(&answers).unwrap()).collect();
    earned_xp(&log)
}

#[test]
fn every_first_answer_right_on_a_finished_cook_earns_sixty() {
    let cook = finished_cook();
    assert_eq!(xp(std::slice::from_ref(&cook), all_right(&cook)), 60);
}

#[test]
fn one_wrong_first_answer_earns_fifty() {
    let cook = finished_cook();
    let mut answers = all_right(&cook);
    answers[1] = first_answer(&cook, QUESTIONS[1], false);
    assert_eq!(xp(std::slice::from_ref(&cook), answers), 50);
}

/// A right retry is never recorded (`POST /v1/quiz-answers` keeps only the first answer), so the only trace a
/// retry could leave is a second answer to the same question. Even that would not make the wrong first one right.
#[test]
fn a_right_answer_beside_a_wrong_first_answer_does_not_earn_the_bonus() {
    let cook = finished_cook();
    let mut answers = all_right(&cook);
    answers.push(first_answer(&cook, QUESTIONS[0], false));
    assert_eq!(xp(std::slice::from_ref(&cook), answers), 50);
}

#[test]
fn a_question_left_unanswered_earns_fifty() {
    let cook = finished_cook();
    let mut answers = all_right(&cook);
    answers.pop();
    assert_eq!(xp(std::slice::from_ref(&cook), answers), 50);
    assert_eq!(xp(std::slice::from_ref(&cook), Vec::new()), 50, "no answer at all");
}

#[test]
fn a_stopped_early_cook_earns_nothing_whatever_its_answers() {
    let stopped = StoredEntry { outcome: "stoppedEarly".to_string(), ..finished_cook() };
    assert_eq!(xp(std::slice::from_ref(&stopped), all_right(&stopped)), 0);
}

#[test]
fn answers_for_another_recipe_than_the_entrys_earn_no_bonus() {
    let other_recipe = StoredEntry { recipe_id: Some("recipe/french-omelette".to_string()), ..finished_cook() };
    assert_eq!(xp(std::slice::from_ref(&other_recipe), all_right(&other_recipe)), 50);
    let lesson = StoredEntry { recipe_id: None, ..finished_cook() };
    assert_eq!(xp(std::slice::from_ref(&lesson), all_right(&lesson)), 50, "a lesson entry names no recipe");
    let cook = finished_cook();
    let mut answers = all_right(&cook);
    answers[2].recipe_id = "recipe/french-omelette".to_string();
    assert_eq!(xp(std::slice::from_ref(&cook), answers), 50, "one answer of another recipe");
}

#[test]
fn answers_with_no_entry_earn_nothing() {
    let never_sent = finished_cook();
    assert_eq!(xp(&[], all_right(&never_sent)), 0);
    let cook = finished_cook();
    assert_eq!(xp(std::slice::from_ref(&cook), all_right(&never_sent)), 50, "another entry's answers");
}

#[test]
fn each_cook_is_judged_by_its_own_answers() {
    let (passed, failed) = (finished_cook(), finished_cook());
    let mut answers = all_right(&passed);
    answers.extend([first_answer(&failed, QUESTIONS[0], false)]);
    assert_eq!(xp(&[passed, failed], answers), 60 + 50);
}

#[test]
fn the_order_the_answers_were_recorded_in_does_not_matter() {
    let cook = finished_cook();
    let mut answers = all_right(&cook);
    answers[0] = first_answer(&cook, QUESTIONS[0], false);
    let reversed: Vec<RecordedAnswer> = answers.iter().rev().cloned().collect();
    assert_eq!(xp(std::slice::from_ref(&cook), reversed), xp(std::slice::from_ref(&cook), answers));
    let right = all_right(&cook);
    let right_reversed: Vec<RecordedAnswer> = right.iter().rev().cloned().collect();
    assert_eq!(xp(std::slice::from_ref(&cook), right_reversed), 60);
}
