// SPDX-FileCopyrightText: 2022 Bean.Tools <github@bean.tools>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(TS)]
#[ts(export)]
pub struct BlenderRelease {
    pub version: Vec<i8>,
    pub version_detail: String,
    pub download_link: String,
    pub download_type: String,
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
        download_link: String,
        download_type: String,
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
            download_link,
            download_type,
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
#[derive(TS)]
#[ts(export)]
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
