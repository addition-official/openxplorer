#!/bin/sh
# SPDX-License-Identifier: AGPL-3.0-only
# run.sh TARGET SECONDS [libFuzzer options…]: fuzzes TARGET for SECONDS with
# the limits the fuzzing job uses (a 2 s hang, 1 GiB of memory), keeping
# new inputs in corpus/TARGET and failures in artifacts/TARGET.
set -eu
target=$1
seconds=$2
shift 2
cd "$(dirname "$0")"
mkdir -p "corpus/$target" "artifacts/$target" logs
exec "target/x86_64-unknown-linux-gnu/release/$target" \
    -timeout=2 -rss_limit_mb=1024 -malloc_limit_mb=1024 -max_len=65536 \
    -max_total_time="$seconds" -print_final_stats=1 \
    -artifact_prefix="artifacts/$target/" "$@" "corpus/$target" \
    >>"logs/$target.log" 2>&1
