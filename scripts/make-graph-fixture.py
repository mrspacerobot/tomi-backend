#!/usr/bin/env python3
"""Writes tests/fixtures/graph/: an excerpt of a graph release with recipes and source instructions.

    scripts/make-graph-fixture.py ../cooking-graph/out/release

It keeps sample recipes, the whole program, every fact (and its thing), the ingredients those recipes and
their swaps name, and the cupboard staples. Everything is copied unchanged from the release; only the selection is ours. Run it again when the
graph's artifact schema changes, and commit the result.
"""
import hashlib
import json
import os
import sys

RECIPES = ["recipe/spinach-and-cheese-omelette", "recipe/seared-chicken-thighs-with-lemon-butter-pan-sauce",
           "recipe/soft-scrambled-eggs"]
OUT = os.path.join(os.path.dirname(__file__), "..", "tests", "fixtures", "graph")


def encode(document):
    return (json.dumps(document, sort_keys=True, ensure_ascii=False, separators=(",", ":")) + "\n").encode("utf-8")


def recipe_things_named(serving):
    found = set()
    for recipe in serving["recipes"].values():
        for line in recipe["lines"]:
            found.add(line["thing"])
            found.update(swap["thing"] for swap in line["swaps"])
        found.update(recipe["equipment"])
        found.update(recipe["techniques"])
        found.add(recipe["makes"])
    return found


def staples():
    path = os.path.join(os.path.dirname(__file__), "..", "library", "staples.json")
    return {"ingredient/" + one["ingredient"] for one in json.load(open(path))["staples"]}


def excerpt(release):
    artifact = json.load(open(os.path.join(release, "graph-artifact.json")))
    serving = {"facts": artifact["serving"]["facts"],
               "recipes": {one: artifact["serving"]["recipes"][one] for one in RECIPES}}
    wanted = recipe_things_named(serving) | staples()
    dish_ids = {recipe["makes"] for recipe in serving["recipes"].values()}
    for dish_id in dish_ids:
        wanted.update(link["thing"] for link in artifact["things"][dish_id].get("belongs_to", []))
    things = {key: value for key, value in artifact["things"].items()
              if key in wanted or key.startswith(("program/", "fact/", "technique/"))}
    for recipe_id in RECIPES:
        things[recipe_id] = artifact["things"][recipe_id]
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
