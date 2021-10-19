use mockall_double::double;

#[double]
use crate::datetime_queries::thing2::Thing2;
pub struct Thing {}
impl Thing {
    pub fn foo(thing2: &Thing2, number: u32) -> u32 {
        // number;
        // super::do_stuff(thing2, number) // this works
        thing2.boo(number)

        // code that we want to test

        // thing2.boo(number) // a nested function that we want to test separately, and want to be able to mock
    }
}

#[cfg(test)]
mod tests {
    use super::Thing;
    use crate::datetime_queries::thing2;

    #[test]
    fn test_test_test() {
        let mut mocked = thing2::MockThing2::new();
        mocked 
            .expect_boo()
            .with(mockall::predicate::eq(1 as u32))
            .times(1)
            .return_const(1 as u32);
        let res = Thing::foo(&mocked, 1 as u32);
    }
}