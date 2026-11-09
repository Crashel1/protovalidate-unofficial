# Native source versions

The validator core and Python adapter are based on
[bufbuild/protovalidate-py v2.1.0](https://github.com/bufbuild/protovalidate-py/tree/v2.1.0),
commit `1026204a4a1f16ef2573448dc7cf89b9f87db018` (Apache-2.0).
The adapter uses PyO3 0.26.0 to support Python 3.8 and `google.protobuf` objects.

| Dependency | Commit |
|---|---|
| abseil-cpp | d407ef122a08203648451e0fec77b3f868b71112 |
| antlr4 | cc82115a4e7f53d71d9d905caa2c2dfa4da58899 |
| cel-cpp | f8ed1bdd658435dee2de9dfd7712a57cf5fe07f2 |
| protobuf | edaa823d8b36a8656d7b2b9241b7d0bfe50af878 |
| re2 | 927f5d53caf8111721e734cf24724686bb745f55 |

Only compilation inputs, headers, and license files are retained under
`crates/protovalidate-deps/third_party`. Each dependency keeps its license files.
The generated parser and Protobuf sources come from the same upstream tag.
The file lists specify which native sources are built.
