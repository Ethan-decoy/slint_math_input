"""Behavior checks for the commit policy used by maintainers and CI."""
import unittest

from check_commits import validate


class CommitPolicyTests(unittest.TestCase):
    def test_valid_types_scopes_languages_and_breaking_changes(self):
        for message in [
            "feat(editor): add centered limits",
            "fix: preserve an unfinished denominator",
            "docs: 补充宿主接入示例",
            "refactor(api)!: replace the draft format\n\nBREAKING CHANGE: migrate saved drafts before loading",
            "revert: restore the former selection policy\n\nRefs: abc1234",
        ]:
            with self.subTest(message=message):
                self.assertEqual(validate(message), [])

    def test_rejects_unfinished_or_unstructured_headers(self):
        for message in ["", "update docs", "fixup! feat: add limits", "Feat: add limits", "fix(): retain selection", "fix: "]:
            with self.subTest(message=message):
                self.assertTrue(validate(message))

    def test_ci_cannot_be_skipped_in_header_or_body(self):
        for directive in ["[skip ci]", "[CI SKIP]", "[no ci]", "[skip actions]", "[actions skip]", "skip-checks: true"]:
            with self.subTest(directive=directive):
                self.assertTrue(validate(f"ci: update validation\n\n{directive}"))

    def test_mentions_of_ci_are_allowed(self):
        self.assertEqual(validate("ci: check commit conventions\n\nRemove the former CI-skip marker."), [])

    def test_body_requires_a_blank_separator(self):
        self.assertTrue(validate("fix: retain cursor\nExplain the change"))
        self.assertEqual(validate("fix: retain cursor\n\nExplain the change\n\nCloses #12"), [])

    def test_header_limit_and_final_punctuation(self):
        self.assertEqual(validate("docs: " + "a" * 66), [])
        self.assertTrue(validate("docs: " + "a" * 67))
        self.assertTrue(validate("docs: improve the guide."))

    def test_crlf_messages_are_supported(self):
        self.assertEqual(validate("fix(input): retain focus\r\n\r\nExplain the behavior\r\n"), [])


if __name__ == "__main__":
    unittest.main()
