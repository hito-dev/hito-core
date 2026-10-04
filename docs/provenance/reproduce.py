#!/usr/bin/env python3
"""Reproduce the filtered import in a fresh destination (git-filter-repo 2.47.0)."""
import json
import pathlib
import subprocess
import sys

manifest = json.loads(pathlib.Path(__file__).with_name('extraction.json').read_text())
source, destination = sys.argv[1:]
def git(*args):
    return subprocess.check_output(['git', '-C', destination, *args])
subprocess.run(['git', 'clone', '--no-local', '--single-branch', '--branch', manifest['source_branch'], source, destination], check=True)
git('reset', '--hard', manifest['source_revision'])
git('update-ref', 'refs/remotes/origin/' + manifest['source_branch'], manifest['source_revision'])
args = ['git', 'filter-repo', '--preserve-commit-hashes']
for path in manifest['paths']:
    args += ['--path', path]
if manifest['legacy_cutoff']:
    ids = git('rev-list', manifest['legacy_cutoff'] + '..' + manifest['source_revision']).splitlines()
    args += ['--commit-callback', 'if commit.original_id not in ' + repr(set(ids)) + ':\n    commit.file_changes = []', '--prune-empty', 'always', '--prune-degenerate', 'always']
else:
    args += ['--path-rename', 'crates/hito_core/:']
subprocess.run(args, cwd=destination, check=True)
assert git('rev-parse', 'HEAD').decode().strip() == manifest['imported_tip'], 'Unexpected imported history; verify git-filter-repo version'
git('branch', '-m', 'imported-history')
print('Verified imported tip:', manifest['imported_tip'])
