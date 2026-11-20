"""Regression tests for importing decisions made on the dish photo review page."""

from pathlib import Path
import runpy
import unittest
from unittest.mock import Mock, patch

PHOTOS = runpy.run_path(str(Path(__file__).resolve().parents[1] / "photos"))


def photo(recipe):
    return dict(recipe=recipe, file=f"{recipe}.webp", recipeFingerprint="fresh",
                shows=["ingredient/salt"], status="generated")


def decision(record, verdict="approve"):
    return {**{key: record[key] for key in ("recipe", "file", "recipeFingerprint", "shows")},
            "decision": verdict}


class PhotoReviewTests(unittest.TestCase):
    def setUp(self):
        self.first = photo("first")
        self.second = photo("second")
        self.work = {"records": {"first": self.first, "second": self.second}, "stale": [], "orphaned": []}
        self.save = Mock()
        self.verify = Mock()
        self.validate = Mock()
        self.context = patch.dict(PHOTOS["review"].__globals__,
                                  save=self.save, verified_local=self.verify, inventory=self.validate)
        self.context.start()
        self.addCleanup(self.context.stop)

    def apply(self, *decisions):
        PHOTOS["apply_review"](self.work, {"reviewer": "Reviewer", "decisions": list(decisions)})

    def test_approval_and_rejection_are_saved_together(self):
        self.apply(decision(self.first), decision(self.second, "reject"))
        records = self.save.call_args.args[0]
        self.assertEqual(records["first"]["status"], "approved")
        self.assertEqual(records["first"]["approvedBy"], "Reviewer")
        self.assertEqual(records["second"]["status"], "rejected")
        self.verify.assert_called_once()
        self.validate.assert_called_once_with("validate")

    def test_changed_foods_or_image_refuse_the_whole_batch(self):
        for change in ({"shows": ["ingredient/pepper"]}, {"file": "other.webp"}, {"recipeFingerprint": "old"}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.apply(decision(self.first), {**decision(self.second), **change})
        self.assertEqual(self.first["status"], "generated")
        self.save.assert_not_called()

    def test_stale_rejected_or_duplicate_decisions_cannot_override_a_review(self):
        with self.assertRaises(ValueError):
            self.apply(decision(self.first), decision(self.first, "reject"))
        self.work["stale"] = [{"recipe": "first"}]
        with self.assertRaises(ValueError):
            self.apply(decision(self.first))
        self.second["status"] = "rejected"
        with self.assertRaises(ValueError):
            self.apply(decision(self.second))
        self.save.assert_not_called()

    def test_importing_the_same_decisions_again_is_safe(self):
        self.apply(decision(self.first))
        self.work["records"] = self.save.call_args.args[0]
        self.save.reset_mock()
        self.apply(decision(self.first))
        self.save.assert_not_called()


if __name__ == "__main__":
    unittest.main()
