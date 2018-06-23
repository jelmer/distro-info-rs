//! Parse Debian and Ubuntu distro-info-data files and provide them as easy-to-consume Rust data
//! structures.
//!
//! Use [``UbuntuDistroInfo``](struct.UbuntuDistroInfo.html) to access the Ubuntu data.  (The
//! Debian implementation has yet to happen.)
extern crate chrono;
extern crate csv;
#[macro_use]
extern crate failure;

use chrono::naive::NaiveDate;
use csv::ReaderBuilder;
use failure::Error;

const UBUNTU_CSV_PATH: &str = "/usr/share/distro-info/ubuntu.csv";

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

pub struct UbuntuDistroInfo {
    _releases: Vec<DistroRelease>,
}

/// A struct capturing the Ubuntu releases stored in `/usr/share/distro-info/ubuntu.csv`
impl UbuntuDistroInfo {
    /// Open `/usr/share/distro-info/ubuntu.csv` and parse the Ubuntu release data contained
    /// therein
    pub fn new() -> Result<UbuntuDistroInfo, Error> {
        let mut distro_info = UbuntuDistroInfo { _releases: vec![] };
        let mut rdr = ReaderBuilder::new().flexible(true)
            .from_path(UBUNTU_CSV_PATH)?;

        let parse_required_str = |field: &Option<&str>| -> Result<String, Error> {
            Ok(field.ok_or(format_err!("failed to read required option"))?.to_string())
        };
        let parse_date = |field: &Option<&str>| -> Result<Option<NaiveDate>, Error> {
            match field {
                &Some(field) => Ok(Some(NaiveDate::parse_from_str(field, "%Y-%m-%d")?)),
                &None => Err(format_err!("unexpected error from: {:?}", field)),
            }
        };
        let parse_server_eol = |field: &Option<&str>| -> Result<Option<NaiveDate>, Error> {
            match field {
                &Some(field) => parse_date(&Some(field)),
                &None => Ok(None),
            }
        };

        for record in rdr.records() {
            let record = record?;
            distro_info._releases
                .push(DistroRelease::new(parse_required_str(&record.get(0))?,
                                         parse_required_str(&record.get(1))?,
                                         parse_required_str(&record.get(2))?,
                                         parse_date(&record.get(3))?,
                                         parse_date(&record.get(4))?,
                                         parse_date(&record.get(5))?,
                                         parse_server_eol(&record.get(6))?))
        }
        Ok(distro_info)
    }

    /// Returns a vector of `DistroRelease`s for Ubuntu releases that were released and supported at
    /// the given date
    pub fn supported<'a>(&'a self, date: NaiveDate) -> Vec<&'a DistroRelease> {
        self._releases
            .iter()
            .filter(|distro_release| match distro_release.eol {
                Some(eol) => date < eol,
                None => false,
            })
            .filter(|distro_release| match distro_release.release {
                Some(release) => date > release,
                None => false,
            })
            .collect()
    }

    /// Returns a vector of `DistroRelease`s for Ubuntu releases that were in development at the
    /// given date
    pub fn devel<'a>(&'a self, date: NaiveDate) -> Vec<&'a DistroRelease> {
        self._releases
            .iter()
            .filter(|distro_release| match distro_release.release {
                Some(release) => date < release,
                None => false,
            })
            .filter(|distro_release| match distro_release.created {
                Some(created) => date > created,
                None => false,
            })
            .collect()
    }

    /// Returns a vector of `DistroRelease`s for Ubuntu releases that had been created at the given
    /// date
    pub fn all_at<'a>(&'a self, date: NaiveDate) -> Vec<&'a DistroRelease> {
        self._releases
            .iter()
            .filter(|distro_release| match distro_release.created {
                Some(created) => date > created,
                None => false,
            })
            .collect()
    }

    /// Returns a `DistroRelease` for the latest Ubuntu release at the given date
    pub fn latest<'a>(&'a self, date: NaiveDate) -> &DistroRelease {
        // This will only be None if there are no entries in the CSV, which means things are very
        // broken
        self.all_at(date).last().unwrap()
    }

    pub fn iter(&self) -> ::std::slice::Iter<DistroRelease> {
        self._releases.iter()
    }
}

impl IntoIterator for UbuntuDistroInfo {
    type Item = DistroRelease;
    type IntoIter = ::std::vec::IntoIter<DistroRelease>;

    fn into_iter(self) -> Self::IntoIter {
        self._releases.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use chrono::naive::NaiveDate;
    use DistroRelease;
    use UbuntuDistroInfo;

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

    #[test]
    fn ubuntu_distro_info_new() {
        UbuntuDistroInfo::new().unwrap();
        ()
    }

    #[test]
    fn ubuntu_distro_info_item() {
        let distro_release = UbuntuDistroInfo::new().unwrap().into_iter().next().unwrap();
        assert_eq!("4.10", distro_release.version);
        assert_eq!("Warty Warthog", distro_release.codename);
        assert_eq!("warty", distro_release.series);
        assert_eq!(Some(NaiveDate::from_ymd(2004, 3, 5)),
                   distro_release.created);
        assert_eq!(Some(NaiveDate::from_ymd(2004, 10, 20)),
                   distro_release.release);
        assert_eq!(Some(NaiveDate::from_ymd(2006, 4, 30)), distro_release.eol);
        assert_eq!(None, distro_release.eol_server);
    }

    #[test]
    fn ubuntu_distro_info_eol_server() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        for distro_release in ubuntu_distro_info {
            if distro_release.series == "dapper" {
                assert_eq!(Some(NaiveDate::from_ymd(2011, 6, 1)),
                           distro_release.eol_server);
                break;
            }
        }
    }

    #[test]
    fn ubuntu_distro_info_supported() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        let date = NaiveDate::from_ymd(2018, 6, 14);
        let supported_series: Vec<String> = ubuntu_distro_info.supported(date)
            .iter()
            .map(|distro_release| distro_release.series.clone())
            .collect();
        assert_eq!(vec!["trusty".to_string(),
                        "xenial".to_string(),
                        "artful".to_string(),
                        "bionic".to_string()],
                   supported_series);
    }

    #[test]
    fn ubuntu_distro_info_devel() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        let date = NaiveDate::from_ymd(2018, 6, 14);
        let devel_series: Vec<String> = ubuntu_distro_info.devel(date)
            .iter()
            .map(|distro_release| distro_release.series.clone())
            .collect();
        assert_eq!(vec!["cosmic".to_string()], devel_series);
    }

    #[test]
    fn ubuntu_distro_info_all_at() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        let date = NaiveDate::from_ymd(2005, 6, 14);
        let all_series: Vec<String> = ubuntu_distro_info.all_at(date)
            .iter()
            .map(|distro_release| distro_release.series.clone())
            .collect();
        assert_eq!(vec!["warty".to_string(), "hoary".to_string(), "breezy".to_string()],
                   all_series);
    }

    #[test]
    fn ubuntu_distro_info_latest() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        let date = NaiveDate::from_ymd(2005, 6, 14);
        let latest_series = ubuntu_distro_info.latest(date).series.clone();
        assert_eq!("breezy".to_string(), latest_series);
    }

    #[test]
    fn ubuntu_distro_info_iter() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        let iter_suites: Vec<String> =
            ubuntu_distro_info.iter().map(|distro_release| distro_release.series.clone()).collect();
        let mut for_loop_suites = vec![];
        for distro_release in ubuntu_distro_info {
            for_loop_suites.push(distro_release.series.clone());
        }
        assert_eq!(for_loop_suites, iter_suites);
    }

    #[test]
    fn ubuntu_distro_info_iters_are_separate() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        let mut iter1 = ubuntu_distro_info.iter();
        let mut iter2 = ubuntu_distro_info.iter();
        assert_eq!(iter1.next().unwrap().series, iter2.next().unwrap().series);
    }

}
