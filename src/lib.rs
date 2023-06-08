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
const DEBIAN_CSV_PATH: &str = "/usr/share/distro-info/debian.csv";

pub struct DistroRelease {
    version: String,
    codename: String,
    series: String,
    created: Option<NaiveDate>,
    release: Option<NaiveDate>,
    eol: Option<NaiveDate>,
    eol_server: Option<NaiveDate>,
}

impl DistroRelease {
    pub fn new(
        version: String,
        codename: String,
        series: String,
        created: Option<NaiveDate>,
        release: Option<NaiveDate>,
        eol: Option<NaiveDate>,
        eol_server: Option<NaiveDate>,
    ) -> Self {
        Self {
            version,
            codename,
            series,
            created,
            release,
            eol,
            eol_server,
        }
    }

    // Getters
    pub fn version(&self) -> &String {
        &self.version
    }
    pub fn codename(&self) -> &String {
        &self.codename
    }
    pub fn series(&self) -> &String {
        &self.series
    }
    pub fn created(&self) -> &Option<NaiveDate> {
        &self.created
    }
    pub fn release(&self) -> &Option<NaiveDate> {
        &self.release
    }
    pub fn eol(&self) -> &Option<NaiveDate> {
        &self.eol
    }
    pub fn eol_server(&self) -> &Option<NaiveDate> {
        &self.eol_server
    }

    // Non-getters
    pub fn is_lts(&self) -> bool {
        self.version.contains("LTS")
    }

    pub fn created_at(&self, date: NaiveDate) -> bool {
        match self.created {
            Some(created) => date >= created,
            None => false,
        }
    }

    pub fn released_at(&self, date: NaiveDate) -> bool {
        match self.release {
            Some(release) => date >= release,
            None => false,
        }
    }

    pub fn supported_at(&self, date: NaiveDate) -> bool {
        self.created_at(date)
            && match self.eol {
                Some(eol) => match self.eol_server {
                    Some(eol_server) => date <= ::std::cmp::max(eol, eol_server),
                    None => date <= eol,
                },
                None => false,
            }
    }
}

pub trait DistroInfo: Sized {
    fn distro_name(&self) -> &'static str;
    fn releases(&self) -> &Vec<DistroRelease>;
    fn from_vec(releases: Vec<DistroRelease>) -> Self;
    /// The full path to the CSV file to read from for this distro
    fn csv_path() -> &'static str;
    /// Read records from the given CSV reader to create a Debian/UbuntuDistroInfo object
    ///
    /// (These records must be in the format used in debian.csv/ubuntu.csv as provided by the
    /// distro-info-data package in Debian/Ubuntu.)
    fn from_csv_reader<T: std::io::Read>(mut rdr: csv::Reader<T>) -> Result<Self, Error> {
        let parse_required_str = |field: &Option<&str>| -> Result<String, Error> {
            Ok(field
                .ok_or(format_err!("failed to read required option"))?
                .to_string())
        };
        let parse_date = |field: &str| -> Result<NaiveDate, Error> {
            Ok(NaiveDate::parse_from_str(field, "%Y-%m-%d")?)
        };

        let mut releases = vec![];
        for record in rdr.records() {
            let record = record?;
            releases.push(DistroRelease::new(
                parse_required_str(&record.get(0))?,
                parse_required_str(&record.get(1))?,
                parse_required_str(&record.get(2))?,
                record.get(3).map(parse_date).transpose()?,
                record.get(4).map(parse_date).transpose()?,
                record.get(5).map(parse_date).transpose()?,
                record.get(6).map(parse_date).transpose()?,
            ))
        }
        Ok(Self::from_vec(releases))
    }

    /// Open this distro's CSV file and parse the release data contained therein
    fn new() -> Result<Self, Error> {
        Self::from_csv_reader(
            ReaderBuilder::new()
                .flexible(true)
                .from_path(Self::csv_path())?,
        )
    }

    /// Returns a vector of `DistroRelease`s for releases that had been created at the given date
    fn all_at<'a>(&'a self, date: NaiveDate) -> Vec<&'a DistroRelease> {
        self.releases()
            .iter()
            .filter(|distro_release| match distro_release.created {
                Some(created) => date >= created,
                None => false,
            })
            .collect()
    }

    /// Returns a vector of `DistroRelease`s for releases that were released at the given date
    fn released(&self, date: NaiveDate) -> Vec<&DistroRelease> {
        self.releases()
            .iter()
            .filter(|distro_release| distro_release.released_at(date))
            .collect()
    }

    /// Returns a vector of `DistroRelease`s for releases that were released and supported at the
    /// given date
    fn supported(&self, date: NaiveDate) -> Vec<&DistroRelease> {
        self.releases()
            .iter()
            .filter(|distro_release| distro_release.supported_at(date))
            .collect()
    }

    /// Returns a vector of `DistroRelease`s for releases that were released but no longer
    /// supported at the given date
    fn unsupported(&self, date: NaiveDate) -> Vec<&DistroRelease> {
        self.released(date)
            .into_iter()
            .filter(|distro_release| !distro_release.supported_at(date))
            .collect()
    }

    /// Returns a vector of `DistroRelease`s for releases that were in development at the given
    /// date
    fn devel(&self, date: NaiveDate) -> Vec<&DistroRelease> {
        self.all_at(date)
            .into_iter()
            .filter(|distro_release| match distro_release.release {
                Some(release) => date < release,
                None => false,
            })
            .collect()
    }

    /// Returns a `DistroRelease` for the latest supported, non-EOL release at the given date
    fn latest(&self, date: NaiveDate) -> Option<&DistroRelease> {
        self.supported(date)
            .into_iter()
            .filter(|distro_release| distro_release.released_at(date))
            .collect::<Vec<_>>()
            .last()
            .copied()
    }

    fn iter(&self) -> ::std::slice::Iter<DistroRelease> {
        self.releases().iter()
    }
}

pub struct UbuntuDistroInfo {
    releases: Vec<DistroRelease>,
}

impl DistroInfo for UbuntuDistroInfo {
    fn distro_name(&self) -> &'static str {
        "Ubuntu"
    }
    fn releases(&self) -> &Vec<DistroRelease> {
        &self.releases
    }
    fn csv_path() -> &'static str {
        UBUNTU_CSV_PATH
    }
    /// Initialise an UbuntuDistroInfo struct from a vector of DistroReleases
    fn from_vec(releases: Vec<DistroRelease>) -> Self {
        Self { releases }
    }
}

impl IntoIterator for UbuntuDistroInfo {
    type Item = DistroRelease;
    type IntoIter = ::std::vec::IntoIter<DistroRelease>;

    fn into_iter(self) -> Self::IntoIter {
        self.releases.into_iter()
    }
}

pub struct DebianDistroInfo {
    releases: Vec<DistroRelease>,
}

impl DistroInfo for DebianDistroInfo {
    fn distro_name(&self) -> &'static str {
        "Debian"
    }
    fn releases(&self) -> &Vec<DistroRelease> {
        &self.releases
    }
    fn csv_path() -> &'static str {
        DEBIAN_CSV_PATH
    }
    /// Initialise an DebianDistroInfo struct from a vector of DistroReleases
    fn from_vec(releases: Vec<DistroRelease>) -> Self {
        Self { releases }
    }
}

impl IntoIterator for DebianDistroInfo {
    type Item = DistroRelease;
    type IntoIter = ::std::vec::IntoIter<DistroRelease>;

    fn into_iter(self) -> Self::IntoIter {
        self.releases.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use chrono::naive::NaiveDate;
    use {
        super::DebianDistroInfo, super::DistroInfo, super::DistroRelease, super::UbuntuDistroInfo,
    };

    #[test]
    fn create_struct() {
        DistroRelease {
            version: "version".to_string(),
            codename: "codename".to_string(),
            series: "series".to_string(),
            created: Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            release: Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            eol: Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            eol_server: Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
        };
    }

    #[test]
    fn distro_release_new() {
        let get_date = |mut n| {
            let mut date = NaiveDate::from_ymd_opt(2018, 6, 14).unwrap();
            while n > 0 {
                date = date.succ_opt().unwrap();
                n -= 1;
            }
            date
        };
        let distro_release = DistroRelease::new(
            "version".to_string(),
            "codename".to_string(),
            "series".to_string(),
            Some(get_date(0)),
            Some(get_date(1)),
            Some(get_date(2)),
            Some(get_date(3)),
        );
        assert_eq!("version", distro_release.version);
        assert_eq!("codename", distro_release.codename);
        assert_eq!("series", distro_release.series);
        assert_eq!(Some(get_date(0)), distro_release.created);
        assert_eq!(Some(get_date(1)), distro_release.release);
        assert_eq!(Some(get_date(2)), distro_release.eol);
        assert_eq!(Some(get_date(3)), distro_release.eol_server);

        assert_eq!(&"version", distro_release.version());
        assert_eq!(&"codename", distro_release.codename());
        assert_eq!(&"series", distro_release.series());
        assert_eq!(&Some(get_date(0)), distro_release.created());
        assert_eq!(&Some(get_date(1)), distro_release.release());
        assert_eq!(&Some(get_date(2)), distro_release.eol());
        assert_eq!(&Some(get_date(3)), distro_release.eol_server());
    }

    #[test]
    fn distro_release_is_lts() {
        let distro_release = DistroRelease::new(
            "98.04 LTS".to_string(),
            "codename".to_string(),
            "series".to_string(),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
        );
        assert!(distro_release.is_lts());

        let distro_release = DistroRelease::new(
            "98.04".to_string(),
            "codename".to_string(),
            "series".to_string(),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
        );
        assert!(!distro_release.is_lts());
    }

    #[test]
    fn distro_release_released_at() {
        let distro_release = DistroRelease::new(
            "98.04 LTS".to_string(),
            "codename".to_string(),
            "series".to_string(),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 16).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
        );
        // not released before release day
        assert!(!distro_release.released_at(NaiveDate::from_ymd_opt(2018, 6, 13).unwrap()));
        // released on release day
        assert!(distro_release.released_at(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()));
        // still released after EOL
        assert!(distro_release.released_at(NaiveDate::from_ymd_opt(2018, 6, 17).unwrap()));
    }

    #[test]
    fn distro_release_supported_at() {
        let distro_release = DistroRelease::new(
            "98.04 LTS".to_string(),
            "codename".to_string(),
            "series".to_string(),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 16).unwrap()),
            Some(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()),
        );
        // not supported before release day
        assert!(!distro_release.supported_at(NaiveDate::from_ymd_opt(2018, 6, 13).unwrap()));
        // supported on release day
        assert!(distro_release.supported_at(NaiveDate::from_ymd_opt(2018, 6, 14).unwrap()));
        // not supported after EOL
        assert!(!distro_release.supported_at(NaiveDate::from_ymd_opt(2018, 6, 17).unwrap()));
    }

    #[test]
    fn debian_distro_info_new() {
        DebianDistroInfo::new().unwrap();
        ()
    }

    #[test]
    fn ubuntu_distro_info_new() {
        UbuntuDistroInfo::new().unwrap();
    }

    #[test]
    fn debian_distro_info_item() {
        let distro_release = DebianDistroInfo::new().unwrap().into_iter().next().unwrap();
        assert_eq!("1.1", distro_release.version);
        assert_eq!("Buzz", distro_release.codename);
        assert_eq!("buzz", distro_release.series);
        assert_eq!(
            Some(NaiveDate::from_ymd_opt(1993, 8, 16).unwrap()),
            distro_release.created
        );
        assert_eq!(
            Some(NaiveDate::from_ymd_opt(1996, 6, 17).unwrap()),
            distro_release.release
        );
        assert_eq!(
            Some(NaiveDate::from_ymd_opt(1997, 6, 5).unwrap()),
            distro_release.eol
        );
        assert_eq!(None, distro_release.eol_server);
    }

    #[test]
    fn ubuntu_distro_info_item() {
        let distro_release = UbuntuDistroInfo::new().unwrap().into_iter().next().unwrap();
        assert_eq!("4.10", distro_release.version);
        assert_eq!("Warty Warthog", distro_release.codename);
        assert_eq!("warty", distro_release.series);
        assert_eq!(
            Some(NaiveDate::from_ymd_opt(2004, 3, 5).unwrap()),
            distro_release.created
        );
        assert_eq!(
            Some(NaiveDate::from_ymd_opt(2004, 10, 20).unwrap()),
            distro_release.release
        );
        assert_eq!(
            Some(NaiveDate::from_ymd_opt(2006, 4, 30).unwrap()),
            distro_release.eol
        );
        assert_eq!(None, distro_release.eol_server);
    }

    #[test]
    fn ubuntu_distro_info_eol_server() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        for distro_release in ubuntu_distro_info {
            match distro_release.series.as_ref() {
                "breezy" => assert_eq!(None, distro_release.eol_server),
                "dapper" => {
                    assert_eq!(
                        Some(NaiveDate::from_ymd_opt(2011, 6, 1).unwrap()),
                        distro_release.eol_server
                    );
                    break;
                }
                _ => {}
            }
        }
    }
    #[test]
    fn ubuntu_distro_info_released() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        // Use dapper's release date to confirm we don't have a boundary issue
        let date = NaiveDate::from_ymd_opt(2006, 6, 1).unwrap();
        let released_series: Vec<String> = ubuntu_distro_info
            .released(date)
            .iter()
            .map(|distro_release| distro_release.series.clone())
            .collect();
        assert_eq!(
            vec![
                "warty".to_string(),
                "hoary".to_string(),
                "breezy".to_string(),
                "dapper".to_string(),
            ],
            released_series
        );
    }

    #[test]
    fn ubuntu_distro_info_supported() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        // Use bionic's release date to confirm we don't have a boundary issue
        let date = NaiveDate::from_ymd_opt(2018, 4, 26).unwrap();
        let supported_series: Vec<String> = ubuntu_distro_info
            .supported(date)
            .iter()
            .map(|distro_release| distro_release.series.clone())
            .collect();
        assert_eq!(
            vec![
                "trusty".to_string(),
                "xenial".to_string(),
                "artful".to_string(),
                "bionic".to_string(),
                "cosmic".to_string(),
            ],
            supported_series
        );
    }

    #[test]
    fn ubuntu_distro_info_unsupported() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        // Use bionic's release date to confirm we don't have a boundary issue
        let date = NaiveDate::from_ymd_opt(2006, 11, 1).unwrap();
        let unsupported_series: Vec<String> = ubuntu_distro_info
            .unsupported(date)
            .iter()
            .map(|distro_release| distro_release.series.clone())
            .collect();
        assert_eq!(
            vec!["warty".to_string(), "hoary".to_string()],
            unsupported_series
        );
    }

    #[test]
    fn ubuntu_distro_info_supported_on_eol_day() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        // Use artful's EOL date to confirm we don't have a boundary issue
        let date = NaiveDate::from_ymd_opt(2018, 7, 19).unwrap();
        let supported_series: Vec<String> = ubuntu_distro_info
            .supported(date)
            .iter()
            .map(|distro_release| distro_release.series.clone())
            .collect();
        assert_eq!(
            vec![
                "trusty".to_string(),
                "xenial".to_string(),
                "artful".to_string(),
                "bionic".to_string(),
                "cosmic".to_string(),
            ],
            supported_series
        );
    }

    #[test]
    fn ubuntu_distro_info_supported_with_server_eol() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        let date = NaiveDate::from_ymd_opt(2011, 5, 14).unwrap();
        let supported_series: Vec<String> = ubuntu_distro_info
            .supported(date)
            .iter()
            .map(|distro_release| distro_release.series.clone())
            .collect();
        assert!(supported_series.contains(&"dapper".to_string()));
    }

    #[test]
    fn ubuntu_distro_info_devel() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        let date = NaiveDate::from_ymd_opt(2018, 4, 26).unwrap();
        let devel_series: Vec<String> = ubuntu_distro_info
            .devel(date)
            .iter()
            .map(|distro_release| distro_release.series.clone())
            .collect();
        assert_eq!(vec!["cosmic".to_string()], devel_series);
    }

    #[test]
    fn ubuntu_distro_info_all_at() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        let date = NaiveDate::from_ymd_opt(2005, 4, 8).unwrap();
        let all_series: Vec<String> = ubuntu_distro_info
            .all_at(date)
            .iter()
            .map(|distro_release| distro_release.series.clone())
            .collect();
        assert_eq!(
            vec![
                "warty".to_string(),
                "hoary".to_string(),
                "breezy".to_string(),
            ],
            all_series
        );
    }

    #[test]
    fn ubuntu_distro_info_latest() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        let date = NaiveDate::from_ymd_opt(2005, 4, 8).unwrap();
        let latest_series = ubuntu_distro_info.latest(date).unwrap().series.clone();
        assert_eq!("hoary".to_string(), latest_series);
    }

    #[test]
    fn ubuntu_distro_info_iter() {
        let ubuntu_distro_info = UbuntuDistroInfo::new().unwrap();
        let iter_suites: Vec<String> = ubuntu_distro_info
            .iter()
            .map(|distro_release| distro_release.series.clone())
            .collect();
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
