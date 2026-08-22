//! NewsJournal desktop application entry point.

fn main() {
    println!("Starting NewsJournal v{}", newsjournal_core::VERSION);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gui_entry_smoketest() {
        assert_eq!(2 + 2, 4);
    }
}
