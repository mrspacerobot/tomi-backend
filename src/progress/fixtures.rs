//! Builds cook entries and catalogs for the progress tests, as TomiCore's `ProgressTestLog` and
//! fixtures. The catalog is the fixture catalog unless a test brings its own: knife and eggs in
//! "basics" first, roux in "sauces" needing knife, two levels each.

use crate::kernel::ids::SavedId;
use crate::library::Catalog;
use crate::profile::Experience;
use crate::progress::game::COOKS_TO_COMPLETE_LEVEL;
use crate::progress::{CookEntry, Outcome, ProgressRules, ProgressSnapshot, SkillProgress};
use serde_json::{json, Value};
use uuid::Uuid;

pub fn id(raw: &str) -> SavedId {
    SavedId::parse(raw).unwrap()
}

pub fn ids(raw: &[&str]) -> Vec<SavedId> {
    raw.iter().map(|item| id(item)).collect()
}

/// A finished cook with its own new id and the quiz passed, as TomiCore's `CookEntry.fixture`.
pub fn cook(skill: &str, level: u8) -> CookEntry {
    CookEntry { id: Uuid::new_v4(), skill: id(skill), level, outcome: Outcome::Finished { quiz_passed: true } }
}

/// As many separate finished cooks of one skill at one level as asked for.
pub fn cooks(count: u32, skill: &str, level: u8) -> Vec<CookEntry> {
    (0..count).map(|_| cook(skill, level)).collect()
}

/// The same cook, abandoned before the last step.
pub fn stopped_before_the_end(entry: &CookEntry) -> CookEntry {
    CookEntry { outcome: Outcome::StoppedEarly, ..entry.clone() }
}

/// The same cook, with the quiz at the end not passed.
pub fn with_the_quiz_wrong(entry: &CookEntry) -> CookEntry {
    CookEntry { outcome: Outcome::Finished { quiz_passed: false }, ..entry.clone() }
}

/// One skill of a test catalog, with levels 1 and 2 and no prerequisites unless the test says.
pub struct SkillFixture {
    id: &'static str,
    course: &'static str,
    needs: Vec<&'static str>,
    levels: Vec<u8>,
}

pub fn skill(id: &'static str, course: &'static str) -> SkillFixture {
    SkillFixture { id, course, needs: Vec::new(), levels: vec![1, 2] }
}

impl SkillFixture {
    pub fn needs(self, needs: &[&'static str]) -> Self {
        SkillFixture { needs: needs.to_vec(), ..self }
    }

    pub fn levels(self, levels: &[u8]) -> Self {
        SkillFixture { levels: levels.to_vec(), ..self }
    }

    fn json(&self) -> Value {
        let levels: Vec<Value> =
            self.levels.iter().map(|number| json!({"number": number, "name": {"en": "A level"}})).collect();
        let name = format!("Skill {}", self.id);
        json!({
            "id": self.id, "course": self.course, "name": {"en": name}, "summary": {"en": "What it teaches"},
            "prerequisites": self.needs, "levels": levels,
        })
    }
}

/// A catalog of these courses and skills, in this order.
pub fn catalog(courses: &[&str], skills: &[SkillFixture]) -> Catalog {
    let courses: Vec<Value> = courses.iter().map(|course| json!({"id": course, "title": {"en": "A course"}})).collect();
    let skills: Vec<Value> = skills.iter().map(SkillFixture::json).collect();
    let file = json!({"schemaVersion": 1, "libraryVersion": 1, "courses": courses, "skills": skills});
    Catalog::parse(&file.to_string()).unwrap()
}

pub fn fixture_skills() -> Vec<SkillFixture> {
    vec![skill("knife", "basics"), skill("eggs", "basics"), skill("roux", "sauces").needs(&["knife"])]
}

pub fn fixture_catalog() -> Catalog {
    catalog(&["basics", "sauces"], &fixture_skills())
}

pub struct TestLog {
    pub catalog: Catalog,
    /// What the cook answered on the experience screen. The basics, the screen's own default, unless a test says.
    pub experience: Experience,
}

impl TestLog {
    pub fn new() -> Self {
        TestLog { catalog: fixture_catalog(), experience: Experience::Basics }
    }

    pub fn with_experience(experience: Experience) -> Self {
        TestLog { catalog: fixture_catalog(), experience }
    }

    pub fn with_catalog(catalog: Catalog) -> Self {
        TestLog { catalog, experience: Experience::Basics }
    }

    /// Enough finished cooks to complete every level of every skill of the catalog.
    pub fn cooks_completing_everything(&self) -> Vec<CookEntry> {
        let mut entries = Vec::new();
        for skill in &self.catalog.skills {
            for level in &skill.levels {
                entries.extend(cooks(COOKS_TO_COMPLETE_LEVEL, &skill.id.to_string(), level.number));
            }
        }
        entries
    }

    pub fn progress(&self, entries: &[CookEntry]) -> ProgressSnapshot {
        ProgressRules::new(&self.catalog, self.experience).progress(entries)
    }

    pub fn progress_of(&self, skill: &str, entries: &[CookEntry]) -> SkillProgress {
        self.progress(entries).skill(&id(skill)).unwrap().clone()
    }
}

/// Finished cooks per level, in catalog order.
pub fn cooks_per_level(skill: &SkillProgress) -> Vec<u32> {
    skill.levels.iter().map(|level| level.cooks).collect()
}

pub fn complete_per_level(skill: &SkillProgress) -> Vec<bool> {
    skill.levels.iter().map(|level| level.is_complete()).collect()
}

pub fn open_per_level(skill: &SkillProgress) -> Vec<bool> {
    skill.levels.iter().map(|level| level.is_open).collect()
}

pub fn granted_per_level(skill: &SkillProgress) -> Vec<bool> {
    skill.levels.iter().map(|level| level.is_granted).collect()
}
