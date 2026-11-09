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

//! Python bindings for the `protovalidate` crate.

mod constants;
mod hints;
mod locks;
mod runtime;
mod view;
mod violation;

use std::collections::HashSet;
use std::sync::{Arc, PoisonError, RwLock};

use crate::locks::RwLockExt;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::import_exception;
use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyList, PyString};

use protovalidate::{DescriptorError, Error};

use constants::{Constants, Imports};
use hints::{PbMessage, ViolationList};
use runtime::{ProtoAdapter, ProtoRuntime};
use view::{Ctx, PyRuntime, TypeSource};

/// The core validator for messages of one Python runtime.
type Core = protovalidate::Validator<PyRuntime>;

import_exception!(protovalidate_unofficial.validator, ValidationError);
import_exception!(protovalidate_unofficial.validator, CompilationError);
import_exception!(protovalidate_unofficial.validator, EvaluationError);

/// Maps a failure of validation itself; `Error::Validation` is handled by
/// the caller, as it is a result rather than a failure.
fn to_py_err(error: Error<Box<PyErr>>) -> PyErr {
    match error {
        // The Python error raised while the message was being read, as it
        // was raised.
        Error::Read(error) => *error,
        Error::Compilation(message) => CompilationError::new_err(message),
        Error::Evaluation(message) => EvaluationError::new_err(message),
        Error::Argument(message) => PyValueError::new_err(message),
        // Neither a compilation nor an evaluation failure; a plain Exception
        // keeps it out of both buckets rather than mislabelling it.
        Error::Unexpected(message) => PyException::new_err(message),
        other => PyException::new_err(other.to_string()),
    }
}

fn descriptor_err(error: &DescriptorError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

/// Validate Protobuf messages against static rules.
///
/// Both protobuf-py messages and legacy google.protobuf messages are
/// accepted; either is validated in place, and only serialized if a custom
/// CEL rule needs a message, list or map as a value.
///
/// Each validator instance caches internal state generated from the static
/// rules, so reusing the same instance for multiple validations
/// significantly improves performance.
#[pyclass(module = "protovalidate_unofficial._protovalidate", frozen)]
struct Validator {
    /// One engine per Python runtime. Compiled rules record how to read
    /// each message type's fields, and that differs between the two
    /// runtimes, so they cannot share an engine.
    engine: PyOnceLock<Engine>,
    constants: &'static Constants,
    /// Python types and extensions.
    imports: Arc<Imports>,
}

/// The core validator and registration state for one Python runtime.
struct Engine {
    /// Adding descriptors needs exclusive access and happens only while
    /// warming up; a `RwLock` leaves the steady state unblocked.
    core: RwLock<Core>,
    /// Descriptor files already added to the pool, by name. Mutated together
    /// with the core, under both write locks; see `register`.
    registered: RwLock<HashSet<String>>,
}

#[pymethods]
impl Validator {
    /// Create a validator for google.protobuf messages.
    #[new]
    fn new(py: Python<'_>) -> PyResult<Self> {
        Ok(Self {
            engine: PyOnceLock::new(),
            constants: Constants::get(py),
            imports: Arc::new(Imports::resolve(py)?),
        })
    }

    /// Validate the given message against the static rules defined in the message's descriptor.
    ///
    /// Parameters:
    ///     message: The message to validate.
    ///     fail_fast: If true, validation will stop after the first iteration.
    ///
    /// Raises:
    ///     CompilationError: If the static rules could not be compiled.
    ///     EvaluationError: If a rule failed while being evaluated.
    ///     ValidationError: If the message is invalid. The violations raised as part of this error should
    ///         always be equal to the list of violations returned by `collect_violations`.
    #[allow(clippy::doc_markdown)] // A Python docstring.
    #[pyo3(signature = (message, *, fail_fast = false))]
    fn validate(&self, py: Python<'_>, message: PbMessage<'_>, fail_fast: bool) -> PyResult<()> {
        let Some(invalid) = self.check(py, &message, fail_fast)? else {
            return Ok(());
        };
        let violations = self.violations(py, &message, &invalid)?;
        Err(ValidationError::new_err((
            format!(
                "invalid {}",
                message
                    .0
                    .getattr(&self.constants.descriptor_upper)?
                    .getattr(&self.constants.name)?
                    .extract::<String>()?
            ),
            violations.unbind(),
        )))
    }

    /// Validates the given message against the static rules defined in the message's descriptor.
    ///
    /// Compared to `validate`, `collect_violations` simply returns the violations as a list and puts
    /// the burden of raising an appropriate exception on the caller.
    ///
    /// The violations returned from this method should always be equal to the violations
    /// raised as part of the ValidationError in the call to `validate`.
    ///
    /// Parameters:
    ///     message: The message to validate.
    ///     fail_fast: If true, validation will stop after the first iteration.
    ///
    /// Returns:
    ///     A list of Violation objects that describe the validation errors.
    ///
    /// Raises:
    ///     CompilationError: If the static rules could not be compiled.
    ///     EvaluationError: If a rule failed while being evaluated.
    #[allow(clippy::doc_markdown)] // A Python docstring.
    #[pyo3(signature = (message, *, fail_fast = false))]
    fn collect_violations<'py>(
        &self,
        py: Python<'py>,
        message: PbMessage<'py>,
        fail_fast: bool,
    ) -> PyResult<ViolationList<'py>> {
        Ok(ViolationList(
            match self.check(py, &message, fail_fast)? {
                None => PyList::empty(py),
                Some(invalid) => self.violations(py, &message, &invalid)?,
            },
        ))
    }
}

/// A message with validation errors.
struct Invalid {
    adapter: ProtoAdapter,
    error: protovalidate::ValidationError,
}

impl Validator {
    /// Resolves a message and validates it, returning `None` when it is
    /// valid.
    fn check(
        &self,
        py: Python<'_>,
        message: &PbMessage<'_>,
        fail_fast: bool,
    ) -> PyResult<Option<Invalid>> {
        let adapter = ProtoAdapter::resolve(&message.0, self.constants)?;
        let engine = self.engine(py, adapter.runtime)?;
        let file = adapter.descriptor(py).getattr(&self.constants.file)?;
        engine.register(py, adapter.runtime, &file, self.constants)?;

        let type_name = adapter.type_name(py, self.constants)?;
        let error = self.evaluate(
            py,
            engine,
            &adapter,
            type_name.to_cow()?.as_ref(),
            &message.0,
            fail_fast,
        )?;
        Ok(error.map(|error| Invalid { adapter, error }))
    }

    /// Converts the violation errors to Python violations.
    fn violations<'py>(
        &self,
        py: Python<'py>,
        message: &PbMessage<'py>,
        invalid: &Invalid,
    ) -> PyResult<Bound<'py, PyList>> {
        violation::build_violations(
            py,
            &invalid.error.encode_violations(),
            &message.0,
            &invalid.adapter,
            self.constants,
            &self.imports,
        )
    }

    /// Returns the engine for `runtime`.
    fn engine(&self, py: Python<'_>, runtime: ProtoRuntime) -> PyResult<&Engine> {
        let _ = runtime;
        self.engine.get_or_try_init(py, || Ok(Engine::new()))
    }

    /// Validates the message in place, returning its violations, or `None`
    /// when the message is valid.
    ///
    /// Reading the message calls into Python, so the interpreter stays
    /// attached throughout; the core lock is taken with the
    /// interpreter-aware variant so a registration waiting on it cannot
    /// deadlock with a validation that Python has preempted.
    fn evaluate<'py>(
        &self,
        py: Python<'py>,
        engine: &Engine,
        adapter: &ProtoAdapter,
        type_name: &str,
        message: &Bound<'py, PyAny>,
        fail_fast: bool,
    ) -> PyResult<Option<protovalidate::ValidationError>> {
        let core = engine
            .core
            .read_py_attached(py)
            .unwrap_or_else(PoisonError::into_inner);
        let source = TypeSource::Descriptor(adapter.descriptor(py));
        let ctx = Ctx::new(py, adapter.runtime, self.constants, message, source);
        match core.validate_message(type_name, &ctx, fail_fast) {
            Ok(()) => Ok(None),
            Err(Error::Validation(error)) => Ok(Some(error)),
            Err(error) => Err(to_py_err(error)),
        }
    }
}

impl Engine {
    fn new() -> Self {
        Self {
            core: RwLock::new(Core::new()),
            registered: RwLock::new(HashSet::new()),
        }
    }

    /// Registers `file`, a descriptor file of `runtime`, together with its
    /// imports. Files already registered are skipped.
    fn register(
        &self,
        py: Python<'_>,
        runtime: ProtoRuntime,
        file: &Bound<'_, PyAny>,
        constants: &Constants,
    ) -> PyResult<()> {
        let name_attr = file.getattr(&constants.name)?;
        let name_value = name_attr.cast::<PyString>()?.to_cow()?;
        let name = name_value.as_ref();
        if self
            .registered
            .read_py_attached(py)
            .unwrap_or_else(PoisonError::into_inner)
            .contains(name)
        {
            return Ok(());
        }
        // Registration is rare (once per file), so holding the write locks
        // across the collection walk costs little.
        let mut registered = self
            .registered
            .write_py_attached(py)
            .unwrap_or_else(PoisonError::into_inner);
        let mut core = self
            .core
            .write_py_attached(py)
            .unwrap_or_else(PoisonError::into_inner);
        runtime.collect_files(file, constants, &mut registered, &mut |_file, bytes| {
            core.add_file_descriptor_bytes(bytes.as_bytes())
                .map_err(|error| descriptor_err(&error))?;
            Ok(())
        })
    }
}

/// The native protovalidate engine.
#[pymodule(gil_used = false)]
mod _protovalidate {
    #[pymodule_export]
    use super::Validator;
    #[pymodule_export]
    use super::violation::Violation;
}
