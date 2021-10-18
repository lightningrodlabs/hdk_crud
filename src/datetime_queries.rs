use crate::retrieval::*;
use crate::wire_element::WireElement;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use hdk::prelude::*;
use mockall_double::double;

use std::convert::identity;

// here I am testing out mockall to see if it can be used to mock nested functions
// for example, fetch_entries_by_day calls fetch_entries_by_hour. When we test
// fetch_entries_by_day, we don't want it to call the original fetch_entries_by_hour,
// but to use the mocked version. The following code simplifies this situation to check
// how mockall handles nested functions.
// we can think of thing::Thing::foo as fetch_entries_by_day, which then makes use of a nested function:
// thing::Thing2::boo, which is like fetch_entries_by_hour
// We need to be able to pass in the struct that contains boo so that when the function is used for real
// it executes the code in boo, but when we test foo, we can pass in the mocked struct instead
// this doesn't seem to be possible as the compiler won't accept MockThing2 as an input when it expects Thing2
// perhaps we could use generic types instead?
mod thing {
    #[double]
    use super::thing2::Thing2;

    use mockall_double::double;
    pub struct Thing {}
    impl Thing {
        pub fn foo(thing2: &Thing2, number: u32) -> u32 {
            // number;
            super::do_stuff(thing2, number)

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
        let mocked = thing2::MockThing2::new();
        let res = thing::Thing::foo(&mocked, 5);
    }
}

// fn do_more(thing: &Thing, input: u32) -> u32 {
//     thing.foo()
// }

// pub struct TimeQueries {}
// #[automock]
// impl TimeQueries {
//     pub fn fetch_entries_by_hour<
//         EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>,
//     >(
//         &self,
//         year: i32,
//         month: u32,
//         day: u32,
//         hour: u32,
//         base_component: String,
//     ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
//         let path = hour_path_from_date(base_component.clone(), year, month, day, hour);
//         let links = get_links(path.hash()?, None)?;

//         let entries: Vec<WireElement<EntryType>> = links
//             .into_inner()
//             .into_iter()
//             .map(|link| {
//                 get_latest_for_entry::<EntryType>(link.target, GetOptions::latest())
//             })
//             .filter_map(Result::ok)
//             .filter_map(identity)
//             .map(|x| WireElement::from(x))
//             .collect();
//         Ok(entries)
//     }
// }

// #[double]
// use TimeQueries as MockQueries;

// fn mock_fetch_entries_by_hour(
//     mock: &MockQueries,
//     year: i32,
//     month: u32,
//     day: u32,
//     hour: u32,
//     base_component: String,
// ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
//     mock.fetch_entries_by_hour(year, month, day, hour, base_component)
// }

pub fn fetch_entries_by_time<EntryType: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    time: FetchEntriesTime,
    base_component: String,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    Ok(match time.hour {
        None => fetch_entries_by_day(time, base_component),
        Some(h) => fetch_entries_by_hour(time.year, time.month, time.day, h, base_component),
    }?)
}

pub fn fetch_entries_by_day<EntryType: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    time: FetchEntriesTime,
    base_component: String,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    let path = day_path_from_date(base_component.clone(), time.year, time.month, time.day);

    let children = path.children()?;

    let entries = children
        .into_inner()
        .into_iter()
        .map(|hour_link| {
            let hour_str = get_last_component_string(hour_link.tag)?;

            let hour = hour_str.parse::<u32>().or(Err(err("Invalid path")))?;

            fetch_entries_by_hour(
                time.year,
                time.month,
                time.day,
                hour,
                base_component.clone(),
            )
        })
        .filter_map(Result::ok)
        .flatten()
        .collect();
    Ok(entries)
}

// returns a vector of wire element of specific entry type
pub fn fetch_entries_by_hour<EntryType: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    base_component: String,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    let path = hour_path_from_date(base_component.clone(), year, month, day, hour);
    let links = get_links(path.hash()?, None)?;

    let entries: Vec<WireElement<EntryType>> = links
        .into_inner()
        .into_iter()
        .map(|link| get_latest_for_entry::<EntryType>(link.target, GetOptions::latest()))
        .filter_map(Result::ok)
        .filter_map(identity)
        .map(|x| WireElement::from(x))
        .collect();
    Ok(entries)
}

pub fn fetch_entries_in_time_range<
    EntryType: TryFrom<SerializedBytes, Error = SerializedBytesError>,
>(
    start_time: FetchEntriesTime,
    end_time: FetchEntriesTime,
    base_component: String,
) -> Result<Vec<WireElement<hdk::prelude::EntryType>>, hdk::prelude::WasmError> {
    is_valid_date_range(start_time.clone(), end_time.clone())?;
    match start_time.hour {
        None => {
            match end_time.hour {
                None => fetch_entries_from_day_to_day(
                    start_time.clone(),
                    end_time.clone(),
                    base_component,
                ),
                Some(_) => {
                    //day to hour: loop from 1st day to 2nd last day, then loop through hours in last day
                    fetch_entries_from_day_to_hour(
                        start_time.clone(),
                        end_time.clone(),
                        base_component,
                    )
                }
            }
        }
        Some(_) => {
            match end_time.hour {
                None => {
                    // hour to day: loop through hours on first day, then 2nd day to last day
                    fetch_entries_from_hour_to_day(
                        start_time.clone(),
                        end_time.clone(),
                        base_component,
                    )
                }
                Some(_) => {
                    // hour to hour: loop through hours on first day, then 2nd day to 2nd last day, then hours on last day
                    fetch_entries_from_hour_to_hour(
                        start_time.clone(),
                        end_time.clone(),
                        base_component,
                    )
                }
            }
        }
    }
}
fn fetch_entries_from_day_to_day(
    start: FetchEntriesTime,
    end: FetchEntriesTime,
    base_component: String,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    let mut dt = start.to_date_time();
    let mut entries = Vec::new();
    let end = end.to_date_time();
    while dt <= end {
        entries.push(fetch_entries_by_day::<EntryType>(
            FetchEntriesTime::from_date_time(dt.clone()),
            base_component.clone(),
        ));
        dt = dt + Duration::days(1);
    }
    Ok(entries
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect())
}

fn fetch_entries_from_day_to_hour(
    start: FetchEntriesTime,
    end: FetchEntriesTime,
    base_component: String,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    let mut dt = start.to_date_time();
    let mut entries = Vec::new();
    let end = end.to_date_time();
    while dt < end {
        entries.push(fetch_entries_by_day::<EntryType>(
            FetchEntriesTime::from_date_time(dt.clone()),
            base_component.clone(),
        ));
        dt = dt + Duration::days(1);
    }
    while dt <= end {
        entries.push(fetch_entries_by_hour::<EntryType>(
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            base_component.clone(),
        ));
        dt = dt + Duration::hours(1);
    }
    Ok(entries
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect())
}

fn fetch_entries_from_hour_to_day(
    start: FetchEntriesTime,
    end: FetchEntriesTime,
    base_component: String,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    let mut dt = start.to_date_time();
    let mut entries = Vec::new();
    let end = end.to_date_time();
    let second_day = next_day(dt.clone());
    while dt < second_day {
        entries.push(fetch_entries_by_hour::<EntryType>(
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            base_component.clone(),
        ));
        dt = dt + Duration::hours(1);
    }
    while dt <= end {
        entries.push(fetch_entries_by_day::<EntryType>(
            FetchEntriesTime::from_date_time(dt.clone()),
            base_component.clone(),
        ));
        dt = dt + Duration::days(1);
    }
    Ok(entries
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect())
}

fn fetch_entries_from_hour_to_hour(
    start: FetchEntriesTime,
    end: FetchEntriesTime,
    base_component: String,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    let mut dt = start.to_date_time();
    let mut entries = Vec::new();
    let end = end.to_date_time();
    let second_day = next_day(dt.clone());
    let second_last_day = end.clone() - Duration::days(1);

    // if hour range is on same day, skip first two loops
    match next_day(dt.clone()) == next_day(end.clone()) {
        true => {}
        false => {
            while dt < second_day {
                entries.push(fetch_entries_by_hour::<EntryType>(
                    dt.year(),
                    dt.month(),
                    dt.day(),
                    dt.hour(),
                    base_component.clone(),
                ));
                dt = dt + Duration::hours(1);
            }
            while dt <= second_last_day {
                entries.push(fetch_entries_by_day::<EntryType>(
                    FetchEntriesTime::from_date_time(dt.clone()),
                    base_component.clone(),
                ));
                dt = dt + Duration::days(1);
            }
        }
    }
    while dt <= end {
        entries.push(fetch_entries_by_hour::<EntryType>(
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            base_component.clone(),
        ));
        dt = dt + Duration::hours(1);
    }
    Ok(entries
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect())
}

fn is_valid_date_range(start: FetchEntriesTime, end: FetchEntriesTime) -> Result<(), WasmError> {
    match start.to_date_time() < end.to_date_time() {
        true => Ok(()),
        false => Err(err("invalid date range")),
    }
}
fn next_day(date_time: DateTime<Utc>) -> DateTime<Utc> {
    let next_day = date_time + Duration::days(1);
    DateTime::from_utc(
        NaiveDate::from_ymd(next_day.year(), next_day.month(), next_day.day()).and_hms(0, 0, 0),
        Utc,
    )
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FetchEntriesTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: Option<u32>,
}

impl FetchEntriesTime {
    fn to_date_time(&self) -> DateTime<Utc> {
        match self.hour {
            None => DateTime::from_utc(
                NaiveDate::from_ymd(self.year, self.month, self.day).and_hms(0, 0, 0),
                Utc,
            ),
            Some(h) => DateTime::from_utc(
                NaiveDate::from_ymd(self.year, self.month, self.day).and_hms(h, 0, 0),
                Utc,
            ),
        }
    }
    fn from_date_time(dt: DateTime<Utc>) -> Self {
        Self {
            year: dt.year(),
            month: dt.month(),
            day: dt.day(),
            hour: Some(dt.hour()),
        }
    }
}

pub fn err(reason: &str) -> WasmError {
    WasmError::Guest(String::from(reason))
}

pub fn get_last_component_string(path_tag: LinkTag) -> ExternResult<String> {
    let hour_path = Path::try_from(&path_tag)?;
    let hour_components: Vec<hdk::hash_path::path::Component> = hour_path.into();

    let hour_bytes: &hdk::hash_path::path::Component =
        hour_components.last().ok_or(err("Invalid path"))?;
    let hour_str: String = hour_bytes.try_into()?;

    Ok(hour_str)
}

pub fn day_path_from_date(base_component: String, year: i32, month: u32, day: u32) -> Path {
    Path::from(format!("{}.{}-{}-{}", base_component, year, month, day))
}

pub fn hour_path_from_date(
    base_component: String,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
) -> Path {
    Path::from(format!(
        "{}.{}-{}-{}.{}",
        base_component, year, month, day, hour
    ))
}

// use mockall_double::double;
// mod thing {
//     use mockall::automock;
//     pub struct Thing{}
//     #[automock]
//     impl Thing {
//         pub fn foo(&self, number: u32) -> u32 {
//             number + number
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use hdk::prelude::*;
    #[test]
    fn test_foo() {
        let mut mock = Thing2::default();
        let input: u32 = 1;
        let output: u32 = 3;

        mock.expect_boo()
            .with(mockall::predicate::eq(input))
            .times(1)
            .return_const(output);
        assert_eq!(thing::Thing::foo(&mock, input), 3);
    }
    // #[test]
    // fn test_boo() {
    //     let mut mock = Boo::default();
    //     let input: u32 = 1;
    //     let input2: u32 = 1;
    //     let output: u32 = 3;

    //     mock
    //         .expect_boo()
    //         .with(mockall::predicate::eq(input), mockall::predicate::eq(input2))
    //         .times(1)
    //         .return_const(output);
    //     assert_eq!(do_more(&mock, input, input2),3);
    // }

    // #[test]
    // fn test_fetch_entries_by_day() {

    //     let mut mock_hdk = MockHdkT::new();

    //     let path = Path::from("create.2021-10-15");
    //     let path_entry = Entry::try_from(path).unwrap();
    //     let path_hash = fixt!(EntryHash); // are there other ways to randomly generate data types?
    //     mock_hdk
    //         .expect_hash_entry()
    //         .with(mockall::predicate::eq(path_entry))
    //         .times(1)
    //         .return_const(Ok(path_hash));

    //     // set up io for get
    //     let path_get_input = vec![GetInput::new(
    //         AnyDhtHash::from(path_hash.clone()),
    //         GetOptions::content(),
    //     )];
    //     let expected_get_output = vec![Some(fixt!(Element))]; // this should return the path
    //     mock_hdk
    //         .expect_get()
    //         .with(mockall::predicate::eq(path_get_input))
    //         .times(1)
    //         .return_const(Ok(expected_get_output));

    //     // set up input and outputs for hash entry
    //     mock_hdk
    //         .expect_hash_entry()
    //         .with(mockall::predicate::eq(path_entry))
    //         .times(1)
    //         .return_const(Ok(path_hash));

    //     // set up input for get links
    //     // set up input and outputs for hash entry
    //     pub const NAME: [u8; 8] = [0x68, 0x64, 0x6b, 0x2e, 0x70, 0x61, 0x74, 0x68];
    //     let get_links_input = vec![GetLinksInput::new(
    //         path_hash,
    //         Some(holochain_zome_types::link::LinkTag::new(NAME)),
    //     )];
    //     let get_links_output = vec![fixt!(Links)]; // this is where I would arbitrarily choose what the children are (ie hours), what is the format of these links?
    //     // Links is a vec of Link, how does the base `get_links` structure the list of links? within one element of vec<Links> or one link per Links?
    //     // constructing this vec of links would be important to testing the functionality
    //     mock_hdk
    //         .expect_get_links()
    //         .with(mockall::predicate::eq(get_links_input))
    //         .times(1)
    //         .return_const(Ok(get_links_output));

    //     // set up a mock of fetch_entries_by_hour
    //     let mut mock_queries = MockQueries::default();

    //     mock_queries
    //         .expect_fetch_entries_by_hour()
    //         .with(
    //             mockall::predicate::eq(year),
    //             mockall::predicate::eq(month),
    //             mockall::predicate::eq(day),
    //             mockall::predicate::eq(hour),
    //             mockall::predicate::eq(base_component),
    //         )
    //         .times(1)
    //         .return_const(output);

    //     mock_fetch_entries_by_hour(&mock_queries, year, month, day, hour, base_component);
    //     assert_eq!(fetch_entries_by_day(&mock_queries, year, month, day, base_component), expected_output);
    //     // how does it know to use the mock version from line 470? And how can it pass that in?
    //     // would fetch_entries_by_day call the 'by hour' trait, or would it call the 'do_stuff' equivalent?
    // }

    fn test_fetch_entries_by_hour() {
        // mock `path.hash()
        // mock `get_links`
        // mock `get_latest_for entry`
    }
    fn test_fetch_entries_by_time() {
        // not necessary
    }
    fn test_fetch_entries_in_time_range() {
        // need to test the logic of all 4 sub functions
        // not sure how we could unit test the date range logic and avoid retesting fetch entries by day or hour
    }
}
