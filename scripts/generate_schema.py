"""Generate google.protobuf bindings for the bundled validation schema."""
from pathlib import Path
from google.protobuf import descriptor_pb2

root = Path(__file__).resolve().parents[1]
file_set = descriptor_pb2.FileDescriptorSet.FromString(
    (root / "crates/protovalidate/src/gen/buf.validate.binpb").read_bytes()
)
proto = next(file for file in file_set.file if file.name == "buf/validate/validate.proto")
imports = "\n".join(
    "from google.protobuf import " + dependency.rsplit("/", 1)[1].replace(".proto", "_pb2")
    for dependency in proto.dependency
)
header = """# Generated from the official v2.1.0 embedded descriptor by scripts/generate_schema.py.
# Copyright 2023-2026 Buf Technologies, Inc. Licensed under Apache-2.0.
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf.internal import builder as _builder
"""
body = (
    header + imports + "\n\nDESCRIPTOR = _descriptor_pool.Default().AddSerializedFile("
    + repr(proto.SerializeToString()) + ")\n"
    + "_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, globals())\n"
    + '_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, "buf.validate.validate_pb2", globals())\n'
)
(root / "buf/validate/validate_pb2.py").write_text(body)
