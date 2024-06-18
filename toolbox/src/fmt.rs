use core::{cmp, fmt, str::from_utf8};

#[macro_export]
macro_rules! format {
    ($buf:expr, $($format_args:tt)*) => {{
        use toolbox::fmt::FmtWriter;
        use core::fmt;
        let mut writer = FmtWriter::new(&mut $buf);
        fmt::write(&mut writer, format_args!($($format_args)*)).unwrap();
        writer.get()
    }};
}

pub struct FmtWriter<'out> {
    buf: &'out mut [u8],
    len: usize,
}

impl<'out> FmtWriter<'out> {
    pub fn new(buf: &'out mut [u8]) -> Self {
        Self { buf, len: 0 }
    }

    pub fn get(self) -> &'out str {
        from_utf8(&self.buf[..self.len]).unwrap()
    }
}

impl<'out> fmt::Write for FmtWriter<'out> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.len > self.buf.len() {
            return Err(fmt::Error);
        }

        let rem = &mut self.buf[self.len..];
        let raw_s = s.as_bytes();
        let num = cmp::min(raw_s.len(), rem.len());

        rem[..num].copy_from_slice(&raw_s[..num]);
        self.len += raw_s.len();

        if num < raw_s.len() {
            Err(fmt::Error)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_formats() {
        let mut buf = [0u8; 128];
        let result = format!(buf, "{}-{}-{}", "hello", 3.12345, false);
        assert_eq!("hello-3.12345-false", result);
    }

    #[test]
    #[should_panic]
    fn test_panics() {
        let mut buf = [0u8; 12];
        let result = format!(buf, "{}-{}-{}", "hello", 3.12345, false);
        assert_eq!("hello-3.12345-false", result);
    }
}
