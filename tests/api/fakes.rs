//! Stand-ins for the model and the photo store, so tests never call out.

use std::collections::BTreeMap;
use std::sync::Mutex;
use tomi_api::ai::{Ai, AiFuture, FridgeQuestion};
use tomi_api::scans::fridge::ModelItem;
use tomi_api::storage::{PhotoStore, StoreFuture, StoredFile};

/// Sees what it is told to see, and ranks as it is told. A ranking of
/// `None` stands for a model that cannot be reached.
pub struct FakeAi {
    pub seen: Vec<ModelItem>,
}

impl Default for FakeAi {
    fn default() -> Self {
        let item = |label: &str, ingredient: Option<&str>| ModelItem {
            label: label.into(),
            ingredient: ingredient.map(Into::into),
            confidence: 0.9,
        };
        let seen = vec![item("eggs", Some("egg")), item("a jar of pesto", Some("pesto"))];
        FakeAi { seen }
    }
}

impl Ai for FakeAi {
    fn model_name(&self) -> &str {
        "fake-model"
    }

    fn read_fridge<'a>(&'a self, _question: FridgeQuestion<'a>) -> AiFuture<'a, Vec<ModelItem>> {
        Box::pin(async { Ok(self.seen.clone()) })
    }
}

/// Keeps files in memory, by path.
#[derive(Default)]
pub struct FakeStore {
    pub files: Mutex<BTreeMap<String, Vec<u8>>>,
}

impl PhotoStore for FakeStore {
    fn is_configured(&self) -> bool {
        true
    }

    fn put<'a>(&'a self, file: StoredFile<'a>) -> StoreFuture<'a> {
        self.files.lock().unwrap().insert(file.path.to_string(), file.bytes);
        Box::pin(async { Ok(()) })
    }

    fn delete<'a>(&'a self, paths: &'a [String]) -> StoreFuture<'a> {
        let mut files = self.files.lock().unwrap();
        for path in paths {
            files.remove(path);
        }
        Box::pin(async { Ok(()) })
    }
}
