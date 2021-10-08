use std::convert::identity;

use holo_hash::EntryHashB64;
use hdk::prelude::*;

use crate::wire_element::WireElement;

// create_entry - calls create_entry_inner which adds the path to it
// if there are any kind of date fields - make a path `entry_type.field.date`
// how to do the search based on a specified range

const SECONDS: i64 = 60;
const MINUTES: i64 = 60;
const HOURS: i64 = 24;

pub fn timestamp_to_days() -> i64 {
    let timestamp = sys_time().unwrap();
    timestamp.as_seconds_and_nanos().0 / (SECONDS * MINUTES * HOURS)
}

#[cfg(test)]
mod tests {
    // use hdk::prelude::sys_time;

    use crate::pagination::timestamp_to_days;

    #[test]
    fn timestamping() {
        // let now = sys_time().unwrap();
        println!("{}", timestamp_to_days());
    }
}