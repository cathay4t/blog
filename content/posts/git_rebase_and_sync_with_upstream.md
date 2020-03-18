---
title: "Git: rebase and sync with upstream"
date: 2020-03-19T01:04:35+08:00
---
## Git: rebase and sync with upstream

I have been asked to do rebase on git branch a lot, so I create some scripts
to help me so.

First of all, assuming your git repo is a clone of upstream. Please add
upstream git link to your current git repo:

```bash
git remote add upstream <link_to_upstream.git>
```

Then place this script in your `$PATH`(for example: `$HOME/bin`) as
`git_sync_upstream` which will sync `origin/master` with `upstream/master`.

```bash
#!/bin/bash -x
git commit  -m'wip' -a      # In case have uncommit work in current branch.
git fetch || exit 1
git fetch upstream || exit 1
git checkout master || exit 1
git reset --hard upstream/master
git push origin +master
```

Now create save script as `rebase`:

```bash
#!/bin/bash -xe

CUR_BRANCH=`git rev-parse --abbrev-ref HEAD`

if [ "CHK$CUR_BRANCH" == "CHK" ];then
    echo "Failed to get current branch name"
    exit 1
fi

git_sync_upstream
git checkout $CUR_BRANCH
git rebase upstream/master
git push --force
```
