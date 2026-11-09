from __future__ import annotations
from buf.validate import validate_pb2

class CompilationError(Exception):
    pass

class EvaluationError(RuntimeError):
    pass

Violations = validate_pb2.Violations

class ValidationError(ValueError):
    def __init__(self, msg, violations):
        super().__init__(msg)
        self._violations = violations

    @property
    def violations(self):
        return self._violations

    def to_proto(self):
        return Violations(violations=[v.proto for v in self._violations])

from protovalidate_unofficial._protovalidate import Validator, Violation
