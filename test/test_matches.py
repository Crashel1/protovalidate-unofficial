import pytest
from google.protobuf import descriptor_pb2, descriptor_pool, message_factory
from buf.validate import validate_pb2
import protovalidate_unofficial as protovalidate


def patterned(pattern):
    file = descriptor_pb2.FileDescriptorProto(name="pattern.proto", package="pattern", syntax="proto3")
    file.dependency.append("buf/validate/validate.proto")
    field = file.message_type.add(name="Value").field.add(name="text", number=1, type=9)
    field.options.Extensions[validate_pb2.field].string.pattern = pattern
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
    descriptor = pool.FindMessageTypeByName("pattern.Value")
    if hasattr(message_factory, "GetMessageClass"):
        return message_factory.GetMessageClass(descriptor)
    return message_factory.MessageFactory(pool).GetPrototype(descriptor)


def test_re2_patterns_through_public_validator():
    valid = patterned(r"^\z")
    validator = protovalidate.Validator()
    validator.validate(valid(text=""))
    with pytest.raises(protovalidate.ValidationError):
        validator.validate(valid(text="a"))
    invalid = patterned(r"^\Z")
    with pytest.raises(protovalidate.CompilationError):
        protovalidate.Validator().validate(invalid(text=""))
