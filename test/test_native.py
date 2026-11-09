from concurrent.futures import ThreadPoolExecutor

import pytest
from google.protobuf import descriptor_pb2, descriptor_pool, message_factory
from buf.validate import validate_pb2
import protovalidate_unofficial as protovalidate
from tests.example.v1 import validations_pb2


def uint63_message():
    file = descriptor_pb2.FileDescriptorProto(name="uint63.proto", package="native", syntax="proto3")
    file.dependency.append("buf/validate/validate.proto")
    field = file.message_type.add(name="Value").field.add(name="value", number=1, type=4)
    field.options.Extensions[validate_pb2.field].uint64.lte = (1 << 63) - 1
    pool = descriptor_pool.DescriptorPool()
    def register(desc):
        for dependency in desc.dependencies:
            register(dependency)
        try:
            pool.AddSerializedFile(desc.serialized_pb)
        except TypeError:
            pass
    register(validate_pb2.DESCRIPTOR)
    pool.Add(file)
    descriptor = pool.FindMessageTypeByName("native.Value")
    if hasattr(message_factory, "GetMessageClass"):
        return message_factory.GetMessageClass(descriptor)
    return message_factory.MessageFactory(pool).GetPrototype(descriptor)


def test_uint64_boundaries_and_violation_proto():
    value = uint63_message()
    validator = protovalidate.Validator()
    for number in [0, 1, (1 << 63) - 1]:
        validator.validate(value(value=number))
    for number in [1 << 63, (1 << 64) - 1]:
        with pytest.raises(protovalidate.ValidationError) as error:
            validator.validate(value(value=number))
        proto = error.value.to_proto()
        assert isinstance(proto, validate_pb2.Violations)
        assert proto.violations[0].rule_id == "uint64.lte"
        assert error.value.violations[0].field_value == number
        assert error.value.violations[0].rule_value == (1 << 63) - 1


def test_shared_validator_concurrent_first_use():
    validator = protovalidate.Validator()
    def validate(number):
        msg = validations_pb2.MultipleValidations(title="foo", name="valid")
        validator.validate(msg)
        invalid = validations_pb2.MultipleValidations(title="bad", name="a")
        failures = validator.collect_violations(invalid)
        return [v.proto.rule_id for v in failures]
    with ThreadPoolExecutor(max_workers=8) as pool:
        results = list(pool.map(validate, range(128)))
    assert all(set(result) == {"string.prefix", "string.min_len"} for result in results)


def test_reject_non_protobuf_object():
    with pytest.raises(TypeError, match="protobuf message"):
        protovalidate.validate(object())
