pub struct Entry {}

impl Entry {
    pub fn headers() -> Vec<&'static str> {
        vec!["Date", "Invested", "Investment", "Amount", "Currency"]
    }
}
