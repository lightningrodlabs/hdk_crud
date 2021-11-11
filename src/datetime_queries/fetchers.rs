#[cfg(feature = "mock")]
use mockall_double::double;

#[cfg_attr(feature = "mock", double)]
use super::{
    fetch_by_day::FetchByDay, fetch_by_hour::FetchByHour,
    fetch_entries_from_day_to_day::FetchByDayDay, fetch_entries_from_day_to_hour::FetchByDayHour,
    fetch_entries_from_hour_to_day::FetchByHourDay,
    fetch_entries_from_hour_to_hour::FetchByHourHour,
};

#[cfg_attr(feature = "mock", double)]
use crate::retrieval::get_latest_for_entry::GetLatestEntry;

pub struct Fetchers {
    pub day_to_day: FetchByDayDay,
    pub day_to_hour: FetchByDayHour,
    pub hour_to_day: FetchByHourDay,
    pub hour_to_hour: FetchByHourHour,
    pub day: FetchByDay,
    pub hour: FetchByHour,
    pub get_latest: GetLatestEntry,
}
impl Fetchers {
    pub fn new(
        day_to_day: FetchByDayDay,
        day_to_hour: FetchByDayHour,
        hour_to_day: FetchByHourDay,
        hour_to_hour: FetchByHourHour,
        day: FetchByDay,
        hour: FetchByHour,
        get_latest: GetLatestEntry,
    ) -> Self {
        Self {
            day_to_day,
            day_to_hour,
            hour_to_day,
            hour_to_hour,
            day,
            hour,
            get_latest,
        }
    }
}