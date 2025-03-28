#!/bin/bash
git rev-list --max-parents=0 HEAD | head -1 | xargs git checkout
