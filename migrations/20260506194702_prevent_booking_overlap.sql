-- Add migration script here
CREATE TRIGGER prevent_booking_overlap 
BEFORE INSERT ON bookings
FOR EACH ROW
WHEN EXISTS (
    SELECT 1 FROM bookings
    WHERE room_id = NEW.room_id
    AND status != 'Cancelled'
    AND start_time < NEW.end_time
    AND end_time > NEW.start_time
)
BEGIN
   SELECT RAISE(ABORT, 'ERR_OVERLAP');
END;