#!/usr/bin/env bash

set -e
set -x

publish_crate() {
    cargo publish -p $1
}

publish_crate speka-macro
publish_crate speka
publish_crate speka-axum
