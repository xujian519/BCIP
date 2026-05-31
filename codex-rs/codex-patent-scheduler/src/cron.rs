use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeDelta;
use chrono::Timelike;
use chrono::Utc;
use chrono::Weekday;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronExpression {
    pub minute: CronField,
    pub hour: CronField,
    pub day_of_month: CronField,
    pub month: CronField,
    pub day_of_week: CronField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CronField {
    Any,
    Single(u8),
    List(Vec<u8>),
    Range { start: u8, end: u8 },
    Step { start: u8, step: u8, max: u8 },
}

#[derive(Debug, thiserror::Error)]
pub enum CronError {
    #[error("无效的 cron 表达式: {0}")]
    InvalidExpression(String),
    #[error("字段值超出范围: {field} = {value}")]
    OutOfRange { field: &'static str, value: u8 },
}

impl CronExpression {
    pub fn parse(expr: &str) -> Result<Self, CronError> {
        let fields: Vec<&str> = expr.split_whitespace().collect();

        if fields.len() != 5 {
            return Err(CronError::InvalidExpression(format!(
                "需要 5 个字段，得到 {} 个",
                fields.len()
            )));
        }

        Ok(Self {
            minute: CronField::parse(fields[0], 0, 59, "minute")?,
            hour: CronField::parse(fields[1], 0, 23, "hour")?,
            day_of_month: CronField::parse(fields[2], 1, 31, "day_of_month")?,
            month: CronField::parse(fields[3], 1, 12, "month")?,
            day_of_week: CronField::parse(fields[4], 0, 7, "day_of_week")?,
        })
    }

    pub fn next_run(&self, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let mut candidate = from + TimeDelta::minutes(1);
        candidate = candidate.with_second(0).unwrap_or(candidate);

        let max_iterations = 60 * 24 * 366;

        for _ in 0..max_iterations {
            if self.matches(&candidate) {
                return Some(candidate);
            }
            candidate += TimeDelta::minutes(1);
        }

        None
    }

    fn matches(&self, dt: &DateTime<Utc>) -> bool {
        self.minute.matches(dt.minute() as u8)
            && self.hour.matches(dt.hour() as u8)
            && self.day_of_month.matches(dt.day() as u8)
            && self.month.matches(dt.month() as u8)
            && self
                .day_of_week
                .matches(dt.weekday().num_days_from_sunday() as u8)
    }

    pub fn to_human(&self) -> String {
        let time_part = match (&self.minute, &self.hour) {
            (CronField::Single(m), CronField::Single(h)) => {
                format!("每天 {h:02}:{m:02}")
            }
            _ => "定期".to_string(),
        };

        let week_part = match &self.day_of_week {
            CronField::Single(0) | CronField::Single(7) => "周日".to_string(),
            CronField::Single(1) => "周一".to_string(),
            CronField::Single(2) => "周二".to_string(),
            CronField::Single(3) => "周三".to_string(),
            CronField::Single(4) => "周四".to_string(),
            CronField::Single(5) => "周五".to_string(),
            CronField::Single(6) => "周六".to_string(),
            CronField::List(days) if days.len() <= 3 => {
                let names: Vec<&str> = days
                    .iter()
                    .map(|d| match d {
                        1 => "周一",
                        2 => "周二",
                        3 => "周三",
                        4 => "周四",
                        5 => "周五",
                        _ => "",
                    })
                    .filter(|s| !s.is_empty())
                    .collect();
                names.join("、")
            }
            CronField::Any => "每天".to_string(),
            _ => "定期".to_string(),
        };

        if week_part == "每天" {
            time_part
        } else {
            format!("每{week_part} {time_part}")
        }
    }
}

impl CronField {
    pub fn parse(input: &str, min: u8, max: u8, name: &'static str) -> Result<Self, CronError> {
        let validate = |v: u8| -> Result<u8, CronError> {
            if v < min || v > max {
                Err(CronError::OutOfRange {
                    field: name,
                    value: v,
                })
            } else {
                Ok(v)
            }
        };

        if input == "*" {
            return Ok(CronField::Any);
        }

        if let Some(rest) = input.strip_prefix("*/") {
            let step: u8 = rest
                .parse()
                .map_err(|_| CronError::InvalidExpression(format!("无效的步进值: {rest}")))?;
            validate(step)?;
            return Ok(CronField::Step {
                start: min,
                step,
                max,
            });
        }

        let without_prefix = input.trim_start_matches('*');

        if without_prefix.contains(',') {
            let mut values = Vec::new();
            for s in without_prefix.split(',') {
                let v: u8 = s
                    .parse()
                    .map_err(|_| CronError::InvalidExpression(format!("无效的列表值: {s}")))?;
                validate(v)?;
                values.push(v);
            }
            return Ok(CronField::List(values));
        }

        if without_prefix.contains('-') {
            let parts: Vec<&str> = without_prefix.split('-').collect();
            if parts.len() == 2 {
                let start: u8 = parts[0].parse().map_err(|_| {
                    CronError::InvalidExpression(format!("无效的范围起始: {}", parts[0]))
                })?;
                let end: u8 = parts[1].parse().map_err(|_| {
                    CronError::InvalidExpression(format!("无效的范围结束: {}", parts[1]))
                })?;
                validate(start)?;
                validate(end)?;
                return Ok(CronField::Range { start, end });
            }
        }

        if let Ok(val) = without_prefix.parse::<u8>() {
            validate(val)?;
            return Ok(CronField::Single(val));
        }

        Err(CronError::InvalidExpression(format!(
            "无法解析字段: {input}"
        )))
    }

    pub fn matches(&self, value: u8) -> bool {
        match self {
            CronField::Any => true,
            CronField::Single(v) => *v == value,
            CronField::List(vals) => vals.contains(&value),
            CronField::Range { start, end } => value >= *start && value <= *end,
            CronField::Step { start, step, max } => {
                if value < *start || value > *max {
                    return false;
                }
                (value - start).is_multiple_of(*step)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_parse_wildcard() {
        let expr = CronExpression::parse("* * * * *").unwrap();
        assert_eq!(expr.minute, CronField::Any);
        assert_eq!(expr.hour, CronField::Any);
    }

    #[test]
    fn test_parse_specific_time() {
        let expr = CronExpression::parse("30 9 * * 1-5").unwrap();
        assert_eq!(expr.minute, CronField::Single(30));
        assert_eq!(expr.hour, CronField::Single(9));
        assert_eq!(expr.day_of_week, CronField::Range { start: 1, end: 5 });
    }

    #[test]
    fn test_parse_step() {
        let expr = CronExpression::parse("*/15 * * * *").unwrap();
        assert_eq!(
            expr.minute,
            CronField::Step {
                start: 0,
                step: 15,
                max: 59
            }
        );
    }

    #[test]
    fn test_parse_list() {
        let expr = CronExpression::parse("0 9,12,18 * * *").unwrap();
        assert_eq!(expr.minute, CronField::Single(0));
        assert_eq!(expr.hour, CronField::List(vec![9, 12, 18]));
    }

    #[test]
    fn test_next_run_daily() {
        let expr = CronExpression::parse("0 9 * * *").unwrap();
        let from = Utc.with_ymd_and_hms(2026, 5, 30, 8, 0, 0).unwrap();
        let next = expr.next_run(from).unwrap();
        assert_eq!(next.hour(), 9);
        assert_eq!(next.minute(), 0);
    }

    #[test]
    fn test_next_run_weekday() {
        let expr = CronExpression::parse("0 14 * * 1").unwrap();
        let from = Utc.with_ymd_and_hms(2026, 5, 30, 0, 0, 0).unwrap();
        let next = expr.next_run(from).unwrap();
        assert_eq!(next.weekday(), Weekday::Mon);
    }

    #[test]
    fn test_to_human() {
        let expr = CronExpression::parse("0 9 * * 1-5").unwrap();
        assert!(expr.to_human().contains("9"));
    }

    #[test]
    fn test_field_matches_any() {
        assert!(CronField::Any.matches(0));
        assert!(CronField::Any.matches(59));
    }

    #[test]
    fn test_field_matches_single() {
        assert!(CronField::Single(30).matches(30));
        assert!(!CronField::Single(30).matches(31));
    }

    #[test]
    fn test_field_matches_list() {
        let field = CronField::List(vec![1, 3, 5]);
        assert!(field.matches(3));
        assert!(!field.matches(2));
    }

    #[test]
    fn test_field_matches_range() {
        let field = CronField::Range { start: 1, end: 5 };
        assert!(field.matches(3));
        assert!(!field.matches(6));
    }

    #[test]
    fn test_field_matches_step() {
        let field = CronField::Step {
            start: 0,
            step: 15,
            max: 59,
        };
        assert!(field.matches(0));
        assert!(field.matches(15));
        assert!(field.matches(30));
        assert!(!field.matches(10));
    }
}
