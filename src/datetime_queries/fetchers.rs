use mockall_double::double;
#[double]
use super::{
    fetch_by_day::FetchByDay, fetch_by_hour::FetchByHour,
    fetch_entries_from_day_to_day::FetchByDayDay, fetch_entries_from_day_to_hour::FetchByDayHour,
    fetch_entries_from_hour_to_day::FetchByHourDay,
    fetch_entries_from_hour_to_hour::FetchByHourHour,
};

#[double]
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
