use pyo3::prelude::*;
use pyo3::types::PyList;
use std::convert::Infallible;
#[derive(Clone)]
pub(crate) struct PbMessage<'py>(pub(crate) Bound<'py, PyAny>);
pub(crate) struct PbFieldPath<'py>(pub(crate) Bound<'py, PyAny>);
macro_rules! extract {
    ($name:ident) => {
        impl<'py> FromPyObject<'py> for $name<'py> {
            fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
                Ok(Self(obj.clone()))
            }
        }
    };
}
extract!(PbMessage);
extract!(PbFieldPath);
pub(crate) struct ViolationList<'py>(pub(crate) Bound<'py, PyList>);
impl<'py> IntoPyObject<'py> for ViolationList<'py> {
    type Target = PyList;
    type Output = Bound<'py, PyList>;
    type Error = Infallible;
    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0)
    }
}
pub(crate) struct ViolationProto<'py>(pub(crate) Bound<'py, PyAny>);
impl<'py> IntoPyObject<'py> for ViolationProto<'py> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = Infallible;
    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0)
    }
}
