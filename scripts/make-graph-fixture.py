#!/usr/bin/env python3
"""Writes tests/fixtures/graph/: a small excerpt of a real graph release, for tests that need a graph.

    scripts/make-graph-fixture.py ../cooking-graph/out/release

It keeps the recipes the shipped lessons map to, the whole program, every fact, and the ingredients those recipes and
their swaps name. Everything is copied unchanged from the release; only the selection is ours. Run it again when the
graph's artifact schema changes, and commit the result.
"""
import hashlib
import json
import os
import sys

RECIPES = ["recipe/spinach-and-cheese-omelette", "recipe/seared-chicken-thighs-with-lemon-butter-pan-sauce"]
OUT = os.path.join(os.path.dirname(__file__), "..", "tests", "fixtures", "graph")


def encode(document):
    return (json.dumps(document, sort_keys=True, ensure_ascii=False, separators=(",", ":")) + "\n").encode("utf-8")


def ingredients_named(serving):
    found = set()
    for recipe in serving["recipes"].values():
        for line in recipe["lines"]:
            found.add(line["thing"])
            found.update(swap["thing"] for swap in line["swaps"])
    return found


def excerpt(release):
    artifact = json.load(open(os.path.join(release, "graph-artifact.json")))
    serving = {"facts": artifact["serving"]["facts"],
               "recipes": {one: artifact["serving"]["recipes"][one] for one in RECIPES}}
    wanted = {one for one in ingredients_named(serving) if one.startswith("ingredient/")}
    things = {key: value for key, value in artifact["things"].items() if key in wanted or key.startswith("program/")}
    return {"artifactSchemaVersion": artifact["artifactSchemaVersion"], "channel": "draft", "things": things,
            "serving": serving}


def main(release):
    body = encode(excerpt(release))
    digest = hashlib.sha256(body).hexdigest()
    manifest = {"artifactSchemaVersion": json.loads(body)["artifactSchemaVersion"], "graphVersion": "fixture",
                "sourceCommit": "fixture", "channel": "draft", "contentSha256": digest}
    os.makedirs(OUT, exist_ok=True)
    open(os.path.join(OUT, "graph-artifact.json"), "wb").write(body)
    open(os.path.join(OUT, "manifest.json"), "wb").write(encode(manifest))
    open(os.path.join(OUT, "graph.lock"), "w").write("version fixture\nsha256 %s\n" % digest)
    print("fixture:", len(body), "bytes,", len(json.loads(body)["things"]), "things")


if __name__ == "__main__":
    main(sys.argv[1])
