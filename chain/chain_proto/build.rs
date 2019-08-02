// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This compiles all the `.proto` files under `src/` directory.
//!
//! For example, if there is a file `src/a/b/c.proto`, it will generate `src/a/b/c.rs` and
//! `src/a/b/c_grpc.rs`.

fn main() {
    let proto_root = "src/proto";
    let dependent_types = "../../types/src/proto";
    let dependent_star_types = "../../star_types/src/proto";

    build_helpers::build_helpers::compile_proto(
        proto_root,
        vec![dependent_types, dependent_star_types], /* dependent roots */
        true,  /* generate_client_stub */
    );
}