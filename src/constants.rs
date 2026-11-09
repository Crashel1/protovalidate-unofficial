// Copyright (c) 2023-2026 Buf Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Interned strings, and the Python types the bindings reach for.

use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyString, PyType};

/// Python types the bindings construct or parse with.
pub(crate) struct Types {
    /// The class `buf.validate.validate_pb2.Violations`.
    pub(crate) violations: Py<PyType>,
    /// The class `buf.validate.validate_pb2.Violation`.
    pub(crate) violation: Py<PyType>,
    /// The class `google.protobuf.descriptor_pb2.FieldOptions`.
    pub(crate) field_options: Py<PyType>,
    /// The class `google.protobuf.descriptor_pb2.MessageOptions`.
    pub(crate) message_options: Py<PyType>,
}

/// Interned strings, shared by every call site.
pub(crate) struct Constants {
    /// The string `DESCRIPTOR`.
    pub(crate) descriptor_upper: Py<PyString>,
    /// The string `FindMessageTypeByName`.
    pub(crate) find_message_type_by_name: Py<PyString>,
    /// The string `GetOptions`.
    pub(crate) get_options: Py<PyString>,
    /// The string `HasField`.
    pub(crate) has_field: Py<PyString>,
    /// The string `SerializeToString`.
    pub(crate) serialize_to_string: Py<PyString>,
    /// The string `dependencies`.
    pub(crate) dependencies: Py<PyString>,
    /// The string `elements`.
    pub(crate) elements: Py<PyString>,
    /// The string `field`.
    pub(crate) field: Py<PyString>,
    /// The string `field_name`.
    pub(crate) field_name: Py<PyString>,
    /// The string `field_number`.
    pub(crate) field_number: Py<PyString>,
    /// The string `fields`.
    pub(crate) fields: Py<PyString>,
    /// The string `fields_by_name`.
    pub(crate) fields_by_name: Py<PyString>,
    /// The string `fields_by_number`.
    pub(crate) fields_by_number: Py<PyString>,
    /// The string `file`.
    pub(crate) file: Py<PyString>,
    /// The string `for_key`.
    pub(crate) for_key: Py<PyString>,
    /// The string `full_name`.
    pub(crate) full_name: Py<PyString>,
    /// The string `items`.
    pub(crate) items: Py<PyString>,
    /// The string `message`.
    pub(crate) message: Py<PyString>,
    /// The string `name`.
    pub(crate) name: Py<PyString>,
    /// The string `number`.
    pub(crate) number: Py<PyString>,
    /// The string `pool`.
    pub(crate) pool: Py<PyString>,
    /// The string `rule`.
    pub(crate) rule: Py<PyString>,
    /// The string `rule_id`.
    pub(crate) rule_id: Py<PyString>,
    /// The string `serialized_pb`.
    pub(crate) serialized_pb: Py<PyString>,
    /// The string `value`.
    pub(crate) value: Py<PyString>,
    /// The string `violations`.
    pub(crate) violations: Py<PyString>,
}

impl Constants {
    /// The process-wide constants, interned on first use.
    pub(crate) fn get(py: Python<'_>) -> &'static Self {
        static INSTANCE: PyOnceLock<Constants> = PyOnceLock::new();

        INSTANCE.get_or_init(py, || Self::new(py))
    }

    fn new(py: Python<'_>) -> Self {
        Self {
            descriptor_upper: PyString::intern(py, "DESCRIPTOR").unbind(),
            find_message_type_by_name: PyString::intern(py, "FindMessageTypeByName").unbind(),
            get_options: PyString::intern(py, "GetOptions").unbind(),
            has_field: PyString::intern(py, "HasField").unbind(),
            serialize_to_string: PyString::intern(py, "SerializeToString").unbind(),
            dependencies: PyString::intern(py, "dependencies").unbind(),
            elements: PyString::intern(py, "elements").unbind(),
            field: PyString::intern(py, "field").unbind(),
            field_name: PyString::intern(py, "field_name").unbind(),
            field_number: PyString::intern(py, "field_number").unbind(),
            fields: PyString::intern(py, "fields").unbind(),
            fields_by_name: PyString::intern(py, "fields_by_name").unbind(),
            fields_by_number: PyString::intern(py, "fields_by_number").unbind(),
            file: PyString::intern(py, "file").unbind(),
            for_key: PyString::intern(py, "for_key").unbind(),
            full_name: PyString::intern(py, "full_name").unbind(),
            items: PyString::intern(py, "items").unbind(),
            message: PyString::intern(py, "message").unbind(),
            name: PyString::intern(py, "name").unbind(),
            number: PyString::intern(py, "number").unbind(),
            pool: PyString::intern(py, "pool").unbind(),
            rule: PyString::intern(py, "rule").unbind(),
            rule_id: PyString::intern(py, "rule_id").unbind(),
            serialized_pb: PyString::intern(py, "serialized_pb").unbind(),
            value: PyString::intern(py, "value").unbind(),
            violations: PyString::intern(py, "violations").unbind(),
        }
    }
}

/// Python types and extension objects, resolved on demand.
pub(crate) struct Imports {
    pub(crate) types: Types,
}

impl Imports {
    pub(crate) fn resolve(py: Python<'_>) -> PyResult<Self> {
        let validate = py.import("buf.validate.validate_pb2")?;
        let descriptor = py.import("google.protobuf.descriptor_pb2")?;
        Ok(Self {
            types: Types {
                violations: validate
                    .getattr("Violations")?
                    .cast_into::<PyType>()?
                    .unbind(),
                violation: validate
                    .getattr("Violation")?
                    .cast_into::<PyType>()?
                    .unbind(),
                field_options: descriptor
                    .getattr("FieldOptions")?
                    .cast_into::<PyType>()?
                    .unbind(),
                message_options: descriptor
                    .getattr("MessageOptions")?
                    .cast_into::<PyType>()?
                    .unbind(),
            },
        })
    }
}
