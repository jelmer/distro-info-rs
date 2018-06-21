extern crate chrono;

use chrono::naive::NaiveDate;

pub struct DistroRelease {
    pub version: String,
    pub codename: String,
    pub series: String,
    pub created: Option<NaiveDate>,
    pub release: Option<NaiveDate>,
    pub eol: Option<NaiveDate>,
    pub eol_server: Option<NaiveDate>,
}

impl DistroRelease {
    pub fn new(version: String,
               codename: String,
               series: String,
               created: Option<NaiveDate>,
               release: Option<NaiveDate>,
               eol: Option<NaiveDate>,
               eol_server: Option<NaiveDate>)
               -> DistroRelease {
        DistroRelease {
            version: version,
            codename: codename,
            series: series,
            created: created,
            release: release,
            eol: eol,
            eol_server: eol_server,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::naive::NaiveDate;
    use DistroRelease;

    #[test]
    fn create_struct() {
        DistroRelease {
            version: "version".to_string(),
            codename: "codename".to_string(),
            series: "series".to_string(),
            created: Some(NaiveDate::from_ymd(2018, 6, 14)),
            release: Some(NaiveDate::from_ymd(2018, 6, 14)),
            eol: Some(NaiveDate::from_ymd(2018, 6, 14)),
            eol_server: Some(NaiveDate::from_ymd(2018, 6, 14)),
        };
        ()
    }

    #[test]
    fn distro_release_new() {
        let get_date = |mut n| {
            let mut date = NaiveDate::from_ymd(2018, 6, 14);
            while n > 0 {
                date = date.succ();
                n = n - 1;
            }
            date
        };
        let distro_release = DistroRelease::new("version".to_string(),
                                                "codename".to_string(),
                                                "series".to_string(),
                                                Some(get_date(0)),
                                                Some(get_date(1)),
                                                Some(get_date(2)),
                                                Some(get_date(3)));
        assert_eq!("version", distro_release.version);
        assert_eq!("codename", distro_release.codename);
        assert_eq!("series", distro_release.series);
        assert_eq!(Some(get_date(0)), distro_release.created);
        assert_eq!(Some(get_date(1)), distro_release.release);
        assert_eq!(Some(get_date(2)), distro_release.eol);
        assert_eq!(Some(get_date(3)), distro_release.eol_server);
    }
}
