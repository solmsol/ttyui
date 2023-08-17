use std::io;
use std::io::Write;

use chrono::{DateTime, Days, Duration, Local, Months};
use console::{Key, Term};

// DateTimeField represents selector element for date and time.
#[derive(Clone, Eq, PartialEq, Debug)]
enum DateTimeField {
    Day,
    Month,
    Year,
    Hour,
    Minute,
    Second,
}

impl DateTimeField {
    fn switch_prev(&mut self) -> Self {
        match self {
            Self::Year => Self::Second,
            Self::Month => Self::Year,
            Self::Day => Self::Month,
            Self::Hour => Self::Day,
            Self::Minute => Self::Hour,
            Self::Second => Self::Minute,
        }
    }
    fn switch_next(&mut self) -> Self {
        match self {
            Self::Year => Self::Month,
            Self::Month => Self::Day,
            Self::Day => Self::Hour,
            Self::Hour => Self::Minute,
            Self::Minute => Self::Second,
            Self::Second => Self::Year,
        }
    }
}

const DEFAULT_DATE_NAME: &str = "due date";

/// DateSelector represents the interactive selector interface for date or time.
///
/// ```rust
/// use ttyui::selector::DateSelector;
/// let mut d = DateSelector::new();
/// d.select().unwrap();
/// println!("selected date: {}", d.to_string());
/// ```
#[derive(Clone, Debug)]
pub struct DateSelector {
    name: String,
    active_field: DateTimeField,
    pub has_time: bool,
    date: DateTime<Local>,
    term: Term,
}

impl DateSelector {
    pub fn new() -> Self {
        Self {
            name: DEFAULT_DATE_NAME.to_string(),
            active_field: DateTimeField::Day,
            date: Local::now(),
            has_time: false,
            term: Term::stdout(),
        }
    }

    pub fn from(date: DateTime<Local>) -> Self {
        Self {
            name: DEFAULT_DATE_NAME.to_string(),
            active_field: DateTimeField::Day,
            date,
            has_time: false,
            term: Term::stdout(),
        }
    }

    pub fn set_title(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn set_date(&mut self, date: DateTime<Local>) {
        self.date = date;
    }

    pub fn is_out_of_field(&self) -> bool {
        match &self.has_time {
            true => false,
            false => match self.active_field {
                DateTimeField::Year | DateTimeField::Month | DateTimeField::Day => false,
                _ => true,
            },
        }
    }

    pub fn left(&mut self) -> io::Result<()> {
        self.active_field = self.active_field.switch_prev();
        if self.is_out_of_field() {
            self.active_field = DateTimeField::Day;
        }
        self.adjust()?;
        Ok(())
    }
    pub fn right(&mut self) -> io::Result<()> {
        self.active_field = self.active_field.switch_next();
        if self.is_out_of_field() {
            self.active_field = DateTimeField::Year;
        }
        self.adjust()?;
        Ok(())
    }
    fn adjust(&self) -> io::Result<()> {
        let msg_len = self.to_string().len();
        self.term.move_cursor_left(msg_len)?;
        match &self.active_field {
            DateTimeField::Year => self.term.move_cursor_right(3)?,
            DateTimeField::Month => self.term.move_cursor_right(6)?,
            DateTimeField::Day => self.term.move_cursor_right(9)?,
            DateTimeField::Hour => self.term.move_cursor_right(12)?,
            DateTimeField::Minute => self.term.move_cursor_right(15)?,
            DateTimeField::Second => self.term.move_cursor_right(18)?,
        };
        Ok(())
    }
    pub fn up(&mut self) -> io::Result<()> {
        match &self.active_field {
            DateTimeField::Year => {
                self.date = self.date.checked_add_months(Months::new(12)).unwrap();
            }
            DateTimeField::Month => {
                self.date = self.date.checked_add_months(Months::new(1)).unwrap();
            }
            DateTimeField::Day => {
                self.date = self.date.checked_add_days(Days::new(1)).unwrap();
            }
            _ => {
                match &self.has_time {
                    true => match &self.active_field {
                        DateTimeField::Hour => {
                            self.date = self.date + Duration::hours(1);
                        }
                        DateTimeField::Minute => {
                            self.date = self.date + Duration::minutes(1);
                        }
                        DateTimeField::Second => {
                            self.date = self.date + Duration::seconds(1);
                        }
                        _ => {}
                    },
                    false => {
                        self.active_field = DateTimeField::Day;
                    }
                };
            }
        };
        Ok(())
    }
    pub fn down(&mut self) -> io::Result<()> {
        match &self.active_field {
            DateTimeField::Year => {
                self.date = self.date.checked_sub_months(Months::new(12)).unwrap();
            }
            DateTimeField::Month => {
                self.date = self.date.checked_sub_months(Months::new(1)).unwrap();
            }
            DateTimeField::Day => {
                self.date = self.date.checked_sub_days(Days::new(1)).unwrap();
            }
            _ => {
                match self.has_time {
                    true => match &self.active_field {
                        DateTimeField::Hour => {
                            self.date = self.date - Duration::hours(1);
                        }
                        DateTimeField::Minute => {
                            self.date = self.date - Duration::minutes(1);
                        }
                        DateTimeField::Second => {
                            self.date = self.date - Duration::seconds(1);
                        }
                        _ => {}
                    },
                    false => {
                        self.active_field = DateTimeField::Day;
                    }
                };
            }
        };
        Ok(())
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_date(&self) -> DateTime<Local> {
        self.date.clone()
    }
    pub fn select(&mut self) -> io::Result<&mut Self> {
        loop {
            self.term.clear_screen()?;
            write!(
                &self.term,
                "{}: {}",
                String::from(self.get_name()),
                self.to_string()
            )?;
            self.adjust()?;

            match self.term.read_key()? {
                Key::ArrowLeft => {
                    self.left()?;
                    self.adjust()?;
                }
                Key::ArrowRight => {
                    self.right()?;
                    self.adjust()?;
                }
                Key::ArrowUp => {
                    self.up()?;
                }
                Key::ArrowDown => {
                    self.down()?;
                }
                Key::Enter => break,
                _ => {}
            };
        }
        self.term.clear_screen()?;
        Ok(self)
    }
}

impl ToString for DateSelector {
    fn to_string(&self) -> String {
        match self.has_time {
            true => format!("{}", self.date.format("%Y-%m-%d %H:%M:%S")),
            false => format!("{}", self.date.format("%Y-%m-%d")),
        }
    }
}

pub fn select_datetime(initial_date: DateTime<Local>) -> io::Result<DateTime<Local>> {
    // println!("input {:?}", initial_date);
    let mut t = DateSelector::from(initial_date);
    t.has_time = true;
    Ok(t.select()?.get_date())
}

pub fn select_date(initial_date: DateTime<Local>) -> io::Result<DateTime<Local>> {
    // println!("input {:?}", initial_date);
    Ok(DateSelector::from(initial_date).select()?.get_date())
}

pub fn ask_yes_no(question_msg: &str) -> io::Result<bool> {
    let mut term = Term::stdout();
    let mut msg = format!("{}: ", question_msg);

    write!(term, "{}", msg)?;
    loop {
        match term.read_key().unwrap() {
            Key::Char('Y') | Key::Char('y') => {
                write!(term, "y\n")?;
                return Ok(true);
            }
            Key::Char('N') | Key::Char('n') => {
                write!(term, "n\n")?;
                return Ok(false);
            }
            _ => {
                term.clear_chars(msg.len())?;
                term.move_cursor_left(msg.len())?;
                msg = "Answer with y or n: ".to_string();
                write!(term, "{}", msg)?;
                continue;
            }
        }
    }
}

/// select_word_from_words provides word selection interface for a slice of words.
/// This method returns selected word or "" when quit with Q or Escape key.
///
pub fn select_word_from_words(description: &str, words: &[&str]) -> io::Result<String> {
    let term = Term::stdout();
    term.clear_line()?;
    let mut seq = 0;
    let word_count = words.len();
    let mut table: Vec<&str> = Vec::with_capacity(word_count);
    table.push("\x1b[32m*\x1b[0m");
    for _ in 0..word_count - 1 {
        table.push(" ");
    }
    loop {
        term.clear_screen()?;
        term.write_line(description)?;
        for i in 0..word_count {
            write!(&term, "{} {}", table[i], words[i])?;
        }
        seq = match term.read_key().unwrap() {
            Key::ArrowUp | Key::Char('k') => {
                if seq == 0 {
                    word_count - 1
                } else {
                    seq - 1
                }
            }
            Key::ArrowDown | Key::Char('j') => {
                if seq == word_count - 1 {
                    0
                } else {
                    seq + 1
                }
            }
            Key::Char('q') | Key::Char('Q') | Key::Escape => {
                term.clear_screen()?;
                return Err(io::Error::new(io::ErrorKind::Other, "quit"));
            }
            Key::Enter => {
                term.clear_screen()?;
                return Ok(String::from(words[seq]));
            }
            _ => seq,
        };

        for i in 0..word_count {
            if i == seq {
                table[i] = "\x1b[32m*\x1b[0m";
            } else {
                table[i] = " ";
            }
        }
    }
}
// #[cfg(test)]
// mod word_selector_tests {
//     use crate::selector::*;
//
//     const DUMMY_WORDS: [&str; 5] = ["asa", "ishi", "usu", "ese", "OSO"];
//
//     #[test]
//     fn s() {
//         select_word_from_words("ko", &DUMMY_WORDS).unwrap();
//     }
// }

#[cfg(test)]
mod date_selector_tests {
    use crate::selector::*;
    use chrono::{Duration, Months};
    use std::thread::sleep;
    use std::time;

    fn date_init() -> (DateSelector, DateSelector) {
        let o = DateSelector::new();
        (o.clone(), o)
    }

    fn datetime_init() -> (DateSelector, DateSelector) {
        let mut o = DateSelector::new();
        o.has_time = true;
        (o.clone(), o)
    }

    #[test]
    fn date_up_increments_day_by_default() {
        let (mut t, s) = date_init();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Duration::days(1))
    }

    #[test]
    fn date_down_decrements_day_by_default() {
        let (mut t, s) = date_init();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Duration::days(1))
    }

    #[test]
    fn date_left_down2_decrements_months() {
        let (mut t, s) = date_init();
        t.left().unwrap();
        t.down().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Months::new(2))
    }

    #[test]
    fn date_left_up_down_results_same_date() {
        let (mut t, s) = date_init();
        t.left().unwrap();
        t.up().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date())
    }

    #[test]
    fn date_left2_down_decrements_year() {
        let (mut t, s) = date_init();
        t.left().unwrap();
        t.left().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Months::new(12))
    }

    #[test]
    fn date_left3_down_decrements_day() {
        let (mut t, s) = date_init();
        t.left().unwrap();
        t.left().unwrap();
        t.left().unwrap();
        t.down().unwrap();
        assert_eq!(t.active_field, s.active_field);
        assert_eq!(t.get_date(), s.get_date() - Duration::days(1));
    }

    #[test]
    fn datetime_left3_down_decrements_second() {
        let (mut t, s) = datetime_init();
        t.left().unwrap();
        t.left().unwrap();
        t.left().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Duration::seconds(1))
    }

    #[test]
    fn date_right_left_up_increments_day() {
        let (mut t, s) = date_init();
        t.right().unwrap();
        t.left().unwrap();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Duration::days(1))
    }

    #[test]
    fn datetime_right_left_up_increments_day() {
        let (mut t, s) = datetime_init();
        t.right().unwrap();
        t.left().unwrap();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Duration::days(1))
    }

    #[test]
    fn date_right_up_increments_year() {
        let (mut t, s) = date_init();
        t.right().unwrap();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Months::new(12))
    }

    #[test]
    fn datetime_right_up_increments_hour() {
        let (mut t, s) = datetime_init();
        t.right().unwrap();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Duration::hours(1))
    }

    #[test]
    fn datetime_right2_down2_decrements_minutes() {
        let (mut t, s) = datetime_init();
        t.right().unwrap();
        t.right().unwrap();
        t.down().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Duration::minutes(2))
    }

    #[test]
    fn datetime_right3_up_increments_second() {
        let (mut t, s) = datetime_init();
        t.right().unwrap();
        t.right().unwrap();
        t.right().unwrap();
        t.up().unwrap();
        assert_eq!(t.get_date(), s.get_date() + Duration::seconds(1))
    }

    #[test]
    fn datetime_right4_down_decrements_year() {
        let (mut t, s) = datetime_init();
        t.right().unwrap();
        t.right().unwrap();
        t.right().unwrap();
        t.right().unwrap();
        t.down().unwrap();
        assert_eq!(t.get_date(), s.get_date() - Months::new(12))
    }

    #[test]
    fn test_date_set_get_name() {
        let mut t = date_init().0;
        t.set_title("ok");
        assert_eq!(t.get_name(), "ok")
    }

    #[test]
    fn test_date_set_date() {
        let (mut t, s) = date_init();
        sleep(time::Duration::from_secs(1));
        t.set_date(Local::now());
        assert_ne!(t.get_date(), s.get_date())
    }
}
