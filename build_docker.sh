#!/bin/bash
# Build sqlx into .sqlx 
cargo sqlx prepare -- --all-targets --all-features

docker build -t jsfong/model-parser-mcp:0.1.0 .   