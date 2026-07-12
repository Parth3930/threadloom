#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"
for spec in "yew-app:dist" "leptos-app:dist" "tanstack-app:dist" "dioxus-app:dist" "react-app:dist" "nextjs-app:out"; do
  app="${spec%%:*}"
  dir="${spec##*:}"
  echo "===== BENCHMARKING $app ($dir) ====="
  node bench.js "../$app/$dir"
done
echo "ALL DONE"
