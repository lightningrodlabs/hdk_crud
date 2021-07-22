use hdk::prelude::*;
use thiserror::Error;

/// This is an Err type that can be returned by [resolve_dependency] if an element that is
/// supposed to contain an entry happens not to. It can be converted into
/// a [ValidateCallbackResult] type automatically.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Element missing its Entry")]
    EntryMissing,
}

impl From<Error> for ValidateCallbackResult {
    fn from(e: Error) -> Self {
        ValidateCallbackResult::Invalid(e.to_string())
    }
}

impl From<Error> for ExternResult<ValidateCallbackResult> {
    fn from(e: Error) -> Self {
        Ok(e.into())
    }
}

/// A generic type for the return type of the generic [resolve_dependency] helper function
/// for use without your validation rules.
pub struct ResolvedDependency<D>(pub Element, pub D);

/// During validation, take any hash that might be in the DHT and
/// try to find it, and deserialize it into the inner entry.
/// This is useful if we need one thing to exist in the DHT before we are
/// to allow some other thing in.
/// Notice that it has nested result types, which simplifies things
/// for the caller of this function.
pub fn resolve_dependency<'a, O>(
    hash: AnyDhtHash,
) -> ExternResult<Result<ResolvedDependency<O>, ValidateCallbackResult>>
where
    O: TryFrom<SerializedBytes, Error = SerializedBytesError>,
{
    let element = match get(hash.clone(), GetOptions::content())? {
        Some(element) => element,
        None => {
            return Ok(Err(ValidateCallbackResult::UnresolvedDependencies(vec![
                hash,
            ])))
        }
    };

    let output: O = match element.entry().to_app_option() {
        Ok(Some(output)) => output,
        Ok(None) => return Ok(Err(Error::EntryMissing.into())),
        Err(e) => return Ok(Err(ValidateCallbackResult::Invalid(e.to_string()))),
    };

    Ok(Ok(ResolvedDependency(element, output)))
}
