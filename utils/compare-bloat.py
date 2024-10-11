import argparse
import json

from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Added:
    name: str
    size: int


@dataclass(frozen=True)
class Removed:
    name: str
    size: int


@dataclass(frozen=True)
class Changed:
    name: str
    old_size: int
    new_size: int

    @property
    def size_delta(self) -> int:
        return self.new_size - self.old_size


@dataclass(frozen=True)
class Delta:
    added: Sequence[Added]
    removed: Sequence[Removed]
    changed: Sequence[Changed]


@dataclass(frozen=True)
class File:
    total_size: int
    functions: Mapping[str, int]

    def compare(self, other: "File", rename: list[tuple[str, str]] = []) -> Delta:
        added = []
        removed = []
        changed = []

        known_names = []

        for name, old_size in self.functions.items():
            if name not in other.functions:
                for old, new in rename:
                    new_name = name.replace(old, new)
                    if new_name in other.functions:
                        name = new_name
                        break

            known_names.append(name)
        
            if name in other.functions:
                new_size = other.functions[name]
                if old_size != new_size:
                    changed.append(Changed(name=name, old_size=old_size, new_size=new_size))
            else:
                removed.append(Removed(name=name, size=old_size))

        for name, size in other.functions.items():
            if name not in known_names:
                added.append(Added(name=name, size=size))

        added.sort(key=lambda a: a.size, reverse=True)
        removed.sort(key=lambda r: r.size, reverse=True)
        changed.sort(key=lambda c: c.size_delta, reverse=True)

        return Delta(added=added, removed=removed, changed=changed)
        

    @staticmethod
    def load(path: Path) -> "File":
        data = json.loads(path.read_text())
        functions = {}
        for function in data["functions"]:
            functions[function["name"]] = function["size"]
        return File(functions=functions, total_size=data["text-section-size"])


def compare_files(path1: Path, path2: Path, rename: list[tuple[str, str]]) -> None:
    file1 = File.load(path1)
    file2 = File.load(path2)
    delta = file1.compare(file2)

    print("Summary:")
    print(f"  old:      {path1}")
    print(f"  new:      {path2}")
    print("  total size:")
    print(f"    delta: {file2.total_size - file1.total_size}")
    print(f"    old:   {file1.total_size}")
    print(f"    new:   {file2.total_size}")
    print(f"  added:   {len(delta.added)}")
    print(f"  removed: {len(delta.removed)}")
    print(f"  changed: {len(delta.changed)}")

    print()

    print(f"Added ({len(delta.added)}):")
    print("size\tname")
    for a in delta.added:
        print(f"{a.size}\t{a.name}")

    print()

    print(f"Removed ({len(delta.removed)}):")
    print("size\tname")
    for r in delta.removed:
        print(f"{r.size}\t{r.name}")

    print()

    print(f"Changed ({len(delta.changed)}):")
    print("delta\told\tnew\tname")
    for c in delta.changed:
        print(f"{c.size_delta}\t{c.old_size}\t{c.new_size}\t{c.name}")

def run() -> None:
    parser = argparse.ArgumentParser(
        description="Compare two JSON dumps produced by cargo bloat",
    )
    parser.add_argument("file1", type=Path)
    parser.add_argument("file2", type=Path)
    parser.add_argument(
        "--rename",
        help="a list of renames to consider, e. g. old_crate=new_crate,OldStruct=NewStruct",
    )

    args = parser.parse_args()
    rename = []
    if args.rename:
        for s in args.rename.split(","):
            parts = s.split("=")
            assert len(parts) == 2
            rename.append(tuple(parts))
    compare_files(args.file1, args.file2, rename)


if __name__ == "__main__":
    run()
