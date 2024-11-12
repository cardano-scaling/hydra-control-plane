#!/bin/bash
cargo run --bin crdgen | tfk8s > bootstrap/stage1/crd.tf

