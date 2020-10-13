---
title: "Git: rebase and sync with upstream"
date: 2020-03-19T01:04:35+08:00
---

I have been asked to do rebase on git branch a lot, so I create some scripts
to help me so.

First of all, assuming your git repo is a clone of upstream. Please add
upstream git link to your current git repo:

```bash
git remote add upstream <link_to_upstream.git>
```

Then place this script in your `$PATH`(for example: `$HOME/bin`) as
`git_sync_upstream` which will sync `origin` with `upstream`.

```bash
#!/bin/bash -x
#!/bin/bash -x
if [ "CHK$(git branch|grep ' master' )" != "CHK" ];then
    BASE_BRANCH="master"
elif [ "CHK$(git branch|grep ' main' )" != "CHK" ];then
    BASE_BRANCH="main"
else
    BASE_BRANCH="base"
fi
git commit  -m'wip' -a
git fetch || exit 1
git fetch upstream || exit 1
git checkout $BASE_BRANCH || exit 1
git reset --hard upstream/$BASE_BRANCH
git push origin +$BASE_BRANCH
```

Now create script as `rebase`:

```bash
#!/bin/bash -e

CUR_BRANCH=`git rev-parse --abbrev-ref HEAD`
if [ "CHK$(git branch|grep ' master' )" != "CHK" ];then
    BASE_BRANCH="master"
elif [ "CHK$(git branch|grep ' main' )" != "CHK" ];then
    BASE_BRANCH="main"
else
    BASE_BRANCH="base"
fi

if [ "CHK$CUR_BRANCH" == "CHK" ];then
    echo "Failed to get current branch name"
    exit 1
fi

git_sync_upstream
git checkout $CUR_BRANCH
git rebase $BASE_BRANCH
git push --force
```
