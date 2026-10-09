# Owl/Avenys protocol schema

`owl_avenys.proto` is the repository-owned schema for the versioned Owl to
Avenys build contract. It defines the synchronous `CompileReq` and
`CompileResp` messages. The workspace-level `Arch/protocol/owl_avenys.proto`
is a mirror used for cross-repository design work; update both copies in the
same change.

Validate syntax and verify that the workspace mirror compiles to the same
descriptor:

```sh
./scripts/validate-protocol-schema.sh /tmp/owl_avenys_schema.pb
```

The script requires `protoc`. GitHub CI and release jobs install
`protobuf-compiler` and run this check. The Mire and Rust payload codecs are
hand-written, so no protobuf library is needed to build or run either tool.
Cross-language interoperability is checked separately with
`owl/scripts/verify-protocol-cross-decode.sh` from the Arch workspace.

Field numbers are permanent. Retire fields with `reserved`; never reuse their
numbers or names. Update the Rust and Mire codecs, both golden fixture sets,
their tests, and this schema together when a compatible protocol version is
extended.
