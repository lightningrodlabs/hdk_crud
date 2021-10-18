use mockall_double::double;
mod thing {
    #[double]
    use super::thing2::Thing2;

    use mockall_double::double;
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
}

mod thing2 {
    #[cfg(feature = "mock")]
    use mockall::automock;

    pub struct Thing2 {}
    #[cfg_attr(feature = "mock", automock)]
    impl Thing2 {
        pub fn boo(&self, input: u32) -> u32 {
            input
        }
    }
}

#[double]
use thing2::Thing2;

pub fn do_stuff(thing2: &Thing2, input: u32) -> u32 {
    thing2.boo(input)
}

#[cfg(test)]
mod tests {
    use super::thing;
    use super::thing2;

    #[test]
    fn test_test_test() {
        let mut mocked = thing2::MockThing2::new();
        mocked 
            .expect_boo()
            .with(mockall::predicate::eq(1 as u32))
            .times(1)
            .return_const(1 as u32);
        let res = thing::Thing::foo(&mocked, 1 as u32);
    }
}