import json
import unittest
from pathlib import Path

from scripts.validate_catalog import validate_catalog


ROOT = Path(__file__).resolve().parents[1]


class CatalogValidationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = json.loads((ROOT / "catalog.json").read_text(encoding="utf-8"))

    def test_repository_catalog_is_valid(self):
        self.assertEqual(validate_catalog(self.catalog, ROOT), [])

    def test_catalog_declares_only_supported_extension_hosts(self):
        supported = {
            "photocraft", "effectcraft", "vectorcraft", "filmcraft", "soundcraft",
            "pdfcraft", "designcraft", "lightcraft", "cadcraft",
        }
        self.assertEqual({app["id"] for app in self.catalog["apps"]}, supported)
        self.assertTrue(all(plugin["app"] in supported for plugin in self.catalog["plugins"]))
        chromatic_fringe = next(plugin for plugin in self.catalog["plugins"] if plugin["id"] == "org.effectcraft.trokute.chromatic-fringe")
        self.assertEqual(chromatic_fringe["app"], "effectcraft")
        self.assertEqual(chromatic_fringe["artifact"], "chromatic-fringe.wat")

    def test_unsupported_app_listing_is_rejected(self):
        catalog = json.loads(json.dumps(self.catalog))
        catalog["plugins"][0]["app"] = "unsupportedcraft"
        errors = validate_catalog(catalog, ROOT)
        self.assertTrue(any("not a supported app id" in error for error in errors))

    def test_duplicate_ids_are_rejected(self):
        catalog = json.loads(json.dumps(self.catalog))
        catalog["plugins"].append(dict(catalog["plugins"][0]))
        errors = validate_catalog(catalog, ROOT)
        self.assertTrue(any("duplicate plugin id" in error for error in errors))

    def test_unknown_app_is_rejected(self):
        catalog = json.loads(json.dumps(self.catalog))
        catalog["plugins"][0]["app"] = "unknowncraft"
        errors = validate_catalog(catalog, ROOT)
        self.assertTrue(any("not a supported app id" in error for error in errors))

    def test_source_path_cannot_escape_repository(self):
        catalog = json.loads(json.dumps(self.catalog))
        catalog["plugins"][0]["source"] = "../outside"
        errors = validate_catalog(catalog, ROOT)
        self.assertTrue(any("must stay inside the repository" in error for error in errors))

    def test_download_url_must_be_http_or_https(self):
        catalog = json.loads(json.dumps(self.catalog))
        catalog["plugins"][0]["downloadUrl"] = "javascript:alert(1)"
        errors = validate_catalog(catalog, ROOT)
        self.assertTrue(any("downloadUrl must be an absolute HTTP(S) URL" in error for error in errors))

    def test_artifact_must_be_a_filename(self):
        catalog = json.loads(json.dumps(self.catalog))
        catalog["plugins"][0]["artifact"] = "../payload.wasm"
        errors = validate_catalog(catalog, ROOT)
        self.assertTrue(any("artifact must be a plain artifact filename" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
