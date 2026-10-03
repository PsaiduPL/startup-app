use jiff::Zoned;

pub fn formatted_time() -> String {
    jiff::civil::DateTime::from(Zoned::now())
        .strftime("%Y-%m-%d %H:%M")
        .to_string()
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn time_test() {
        let t = formatted_time();
        assert_eq!(t, "xd");
    }
}
