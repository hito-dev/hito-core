# Development history

The extraction manifest records the source revision and selected paths. `commit-map.txt` maps original firmware commits to extracted commits; zero hashes mean the commit did not change a selected path. Filtering preserves author and committer identities, dates, and messages, but changes hashes because the file tree and parents change.

The imported-history branch contains the filtered development history. It was merged into the GitHub repository’s initial main branch, preserving its existing license commits. Packaging follows in separate commits.

To reproduce the import, install git-filter-repo 2.47.0, clone the source into a fresh temporary repository, and set its sole branch to the manifest’s source revision. Filter with `--preserve-commit-hashes` and one `--path` for each manifest path. For Core, also use `--path-rename crates/hito_core/:`. For UI, clear `commit.file_changes` for every original commit outside `legacy_cutoff..source_revision`, and use `--prune-empty always --prune-degenerate always`; this excludes the old Slint UI that once used the same directory name. Do not filter or force-push the original firmware repository.

Before merging, compare every extracted file’s mode and blob hash with the selected source tree. For each nonzero mapping, compare author, author email/date, committer, committer email/date, and full message. These comparisons passed for this import.
