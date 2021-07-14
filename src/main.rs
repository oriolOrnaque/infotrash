use std::{
    fs::{File, OpenOptions},
    io::Read,
    str,
    fmt,
    iter::FromIterator,
};

use bytes::{Bytes, Buf};

use bindings::{
    Windows::Win32::System::Time::FileTimeToSystemTime,
    Windows::Win32::Foundation::{FILETIME, SYSTEMTIME},
};

use clap::{Arg, App};

#[allow(dead_code)]
struct DollarI {
    // bytes on the file
    header: u64,
    file_size: u64,
    file_time: FILETIME,
    file_name_length: u32,
    file_name: String,
    // internal conversion
    system_time: SYSTEMTIME,
}

impl DollarI {
    fn from_bytes(bytes: &mut Bytes) -> Self {
        let header = bytes.get_u64_le();
        let file_size = bytes.get_u64_le();
        let file_time = FILETIME {
            dwLowDateTime: bytes.get_u32_le(),
            dwHighDateTime: bytes.get_u32_le(),
        };
        // TODO: support also legacy versions
        // https://df-stream.com/2016/04/fun-with-recycle-bin-i-files-windows-10/
        //versions previous to windows 10 do not use this field
        let file_name_length = bytes.get_u32_le();
        //https://stackoverflow.com/questions/36251992/casting-a-vecu8-to-a-u16
        let file_name = Vec::from_iter(&bytes[..]);
        let file_name: Vec<u16> = file_name.chunks_exact(2)
                                        .into_iter()
                                        .map(|a| u16::from_ne_bytes([*a[0], *a[1]]))
                                        .collect();
        let file_name = file_name.as_slice();
        let file_name_utf16 = String::from_utf16_lossy(file_name);

        let mut system_time: SYSTEMTIME = SYSTEMTIME {
            wYear: 0,
            wMonth: 0,
            wDayOfWeek: 0,
            wDay: 0,
            wHour: 0,
            wMinute: 0,
            wSecond: 0,
            wMilliseconds: 0,
        };
        // TODO: find a cross-platform way of compute the Windows SystemTime from FileTime
        // currently all the libs are based on UNIX timestamps which use the UNIX epoch: 1970 ...
        // windows uses its own epoch: 1601 ... just in case Isaac Newton wanted to publish in Word
        unsafe {
            FileTimeToSystemTime(&file_time as *const FILETIME, &mut system_time as *mut SYSTEMTIME).as_bool();
        }

        DollarI {
            header: header,
            file_size: file_size,
            file_time: file_time,
            file_name_length: file_name_length,
            file_name: file_name_utf16,
            system_time: system_time,
        }
    }
}

impl fmt::Display for DollarI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} | Deleted on {}/{}/{} {}:{}:{} UTC",
            self.file_name,
            self.system_time.wDay,
            self.system_time.wMonth,
            self.system_time.wYear,
            self.system_time.wHour,
            self.system_time.wMinute,
            self.system_time.wSecond)
    }
}

fn main() {
    let matches = App::new("infotrash")
                    .version("0.1")
                    .author("Oriol Ornaque")
                    .about("Displays information from $IXXXXXX files")
                    .arg(Arg::with_name("file")
                            .help("Input file(s)")
                            .required(true)
                            .multiple(true)
                            .index(1)
                        )
                    .get_matches();
    
    // unwrap is safe because the presence of values is guaranteed by clap
    let paths: Vec<&str> = matches.values_of("file").unwrap().collect();

    for path in paths {
        let mut data: Vec<u8> = Vec::with_capacity(256);
        match OpenOptions::new().read(true).open(&path).and_then(|mut file: File| file.read_to_end(&mut data)) {
            Ok(_) => {
                let mut bytes = Bytes::copy_from_slice(&data);
                let dollar_i = DollarI::from_bytes(&mut bytes);
                println!("{}", dollar_i);
            },
            Err(err_str) => println!("Could not read file {}: {}", path, err_str),
        }
    }
}
