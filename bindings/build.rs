fn main() {
    windows::build!(
        Windows::Win32::System::Time::FileTimeToSystemTime,
        Windows::Win32::Foundation::{FILETIME, SYSTEMTIME},
    );
}