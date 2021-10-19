#[cfg(feature = "mock")]
use mockall::automock;

pub struct Thing2 {}
#[cfg_attr(feature = "mock", automock)]
impl Thing2 {
    pub fn boo(&self, input: u32) -> u32 {
        input
    }
}