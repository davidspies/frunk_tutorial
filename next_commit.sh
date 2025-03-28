#!/bin/bash
git merge-base --is-ancestor HEAD master || {
    echo "Not on master ancestry"
    exit 1
}
[ "$(git rev-parse HEAD)" = "$(git rev-parse master)" ] && {
    echo "At master head"
    exit 0
}
next=$(git rev-list --topo-order --reverse HEAD..master | head -1)
git checkout $next && git log -1 --oneline $next
