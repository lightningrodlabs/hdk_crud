
#[cfg(test)]
use ::fixt::prelude::*;

mod signals;
pub use signals::*;

mod wrapped_hash;
pub use wrapped_hash::*;

mod retrieval;
pub use retrieval::*;

mod validation_support;
pub use validation_support::*;

mod crud;
pub use crud::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
