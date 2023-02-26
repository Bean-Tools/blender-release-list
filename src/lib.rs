use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlenderRelease {
    pub version: Vec<i8>,
    pub version_detail: String,
    pub download_link_installer: String,
    pub download_link_archive: String,
    pub download_size: String,
    pub release_date: NaiveDateTime,
    pub tag: String,
    pub os: String,
    pub arch: String,
    pub sha256: String,
    pub ga_label: String,
}

impl BlenderRelease {
    pub fn new(
        version: Vec<i8>,
        version_detail: String,
        download_link_installer: String,
        download_link_archive: String,
        download_size: String,
        release_date: NaiveDateTime,
        tag: String,
        os: String,
        arch: String,
        sha256: String,
        ga_label: String,
    ) -> BlenderRelease {
        BlenderRelease {
            version,
            version_detail,
            download_link_installer,
            download_link_archive,
            download_size,
            release_date,
            tag,
            os,
            arch,
            sha256,
            ga_label,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlenderReleaseList {
    pub releases: Vec<BlenderRelease>,
}

impl BlenderReleaseList {
    pub fn new() -> BlenderReleaseList {
        BlenderReleaseList {
            releases: Vec::new(),
        }
    }
}
