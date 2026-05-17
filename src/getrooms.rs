#![allow(dead_code)]
use chrono::NaiveDate;

pub fn rooms_available(
    bookings: &[(NaiveDate, NaiveDate)],
    data_from: NaiveDate,
    data_to: NaiveDate,
) -> bool {
    for (check_in, check_out) in bookings {
        if check_in < &data_to && check_out > &data_from {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod test {

    use super::*;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }
    //test one rooms avilable
    #[test]
    fn test_fn_rooms_available_when_no_bookings() {
        let bookings: Vec<(NaiveDate, NaiveDate)> = vec![];
        let data_from = d(2026, 3, 1);
        let data_to = d(2026, 3, 3);

        let result = rooms_available(&bookings, data_from, data_to);

        assert!(result, "Room is avilable");
    }

    //test two
    #[test]
    fn test_fn_rooms_available_when_double_booking() {
        let bookings: Vec<(NaiveDate, NaiveDate)> = vec![(d(2026, 3, 1), d(2026, 3, 3))];
        let data_from = d(2026, 3, 1);
        let data_to = d(2026, 3, 3);
        let result = rooms_available(&bookings, data_from, data_to);

        assert!(!result);
    }

    #[test]
    //test three
    fn test_fn_rooms_available_when_partial_overlap() {
        let bookings: Vec<(NaiveDate, NaiveDate)> = vec![(d(2026, 3, 1), d(2026, 3, 5))];
        let data_from = d(2026, 3, 3);
        let data_to = d(2026, 3, 7);
        let result = rooms_available(&bookings, data_from, data_to);
        assert!(!result)
    }

    #[test]
    fn test_fn_rooms_available_when_no_overlap() {
        let bookings: Vec<(NaiveDate, NaiveDate)> = vec![(d(2026, 3, 1), d(2026, 3, 3))];
        let data_from = d(2026, 3, 5);
        let data_to = d(2026, 3, 7);
        let result = rooms_available(&bookings, data_from, data_to);
        assert!(result)
    }

    #[test]
    fn test_fn_room_available_when_check_out_equals_check_in() {
        let bookings: Vec<(NaiveDate, NaiveDate)> = vec![(d(2026, 3, 1), d(2026, 3, 3))];
        let data_from = d(2026, 3, 3);
        let data_to = d(2026, 3, 5);
        let result = rooms_available(&bookings, data_from, data_to);
        assert!(result)
    }
}
