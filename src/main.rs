// SPDX-FileCopyrightText: 2022 Bean.Tools <github@bean.tools>
// SPDX-License-Identifier: GPL-3.0-or-later

// blender_org::scrape() returns a Vec<Version> , which is a list of versions of blender. 
// Each version has a version number, a download link, and a release date.
// The version number is a String, the download link is a String, and the release
// date is a chrono::NaiveDate.

use chrono::{NaiveDate, NaiveDateTime, Utc};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;

use blender_beans_scraper::{BlenderRelease, BlenderReleaseList};

#[derive(serde::Serialize)]
pub struct CustomBlenderReleaseList(BlenderReleaseList);

impl CustomBlenderReleaseList {
    pub fn new() -> CustomBlenderReleaseList {
        CustomBlenderReleaseList {
            0: BlenderReleaseList::new(),
        }
    }
}

impl From<Vec<ElementRef<'_>>> for CustomBlenderReleaseList {
    fn from(elements: Vec<ElementRef>) -> CustomBlenderReleaseList {
        let mut release_list = CustomBlenderReleaseList::new();

        for element in elements {
            // First we get a few elements as ElementRef that we need to extract data from
            // We use the select() method to get the first element that matches the selector
            // If the element is not found, we continue to the next element in the list

            // Extract the build detail element from the ul element with the class "build-details" inside the parent element
            let build_details_selector = Selector::parse("ul.build-details").unwrap();
            let build_details = match element.select(&build_details_selector).next() {
                Some(build_details) => build_details,
                None => continue,
            };

            // After we have the elements we need, we can extract the data we need from them
            // We either use the text() method to get the text content of the element,
            // or the attr() method to get the value of an attribute

            // Extract the download link from the href attribute of the first a element in the li element
            let download_installer_selector = Selector::parse("a:first-child").unwrap();
            let download_link = match element.select(&download_installer_selector).next() {
                Some(link) => link.value().attr("href").unwrap().to_string(),
                None => continue,
            };

            // Extract the archive type for the build from the build meta element
            let download_type_selector = Selector::parse("li[title='File extension']").unwrap();
            let download_type = match element.select(&download_type_selector).next() {
                Some(download_type) => download_type.text().collect::<String>(),
                None => continue,
            };

            // Extract the download size from the li element with the title "File size"
            let download_size_selector = Selector::parse("li[title='File size']").unwrap();
            let download_size = match element.select(&download_size_selector).next() {
                Some(download_size) => download_size.text().collect::<String>(),
                None => continue,
            };

            // Extract the version number and build details from the filename of the download link
            let filename = download_link.split("/").last().unwrap().to_string();
            let version_raw = match filename.split("-").nth(1) {
                Some(version) => version.to_string(),
                None => continue,
            };

            let version = match version_raw.split("-").nth(0) {
                Some(version) => {
                    let v = version.split(".").collect::<Vec<&str>>();
                    vec![
                        v[0].parse::<i8>().unwrap(),
                        v[1].parse::<i8>().unwrap(),
                        v[2].parse::<i8>().unwrap(),
                    ]
                }
                None => continue,
            };

            let version_detail = match filename.split("-").nth(2) {
                Some(version) => version.to_string(),
                None => continue,
            };

            // Extract the date from the title of the first li element in the build details
            let date_selector = Selector::parse("li:first-child").unwrap();
            let release_date = match build_details.select(&date_selector).next() {
                Some(date) => NaiveDateTime::parse_from_str(
                    date.value().attr("title").unwrap(),
                    "%Y-%m-%dT%H:%M:%S%z",
                )
                .unwrap(),
                None => continue,
            };

            // Extract the branch(stable, beta, etc.) from the span element with the class "build-var"
            let tag_selector = Selector::parse(".build-var").unwrap();
            let tag = match element.select(&tag_selector).next() {
                Some(tag) => tag.text().collect::<String>(),
                None => "Unknown".to_string(),
            };

            // Extract the sha hash link from the href attribute of the a element with the class "sha"
            let sha_selector = Selector::parse("a.sha").unwrap();
            let sha = match element.select(&sha_selector).next() {
                Some(sha) => sha.value().attr("href").unwrap().to_string(),
                None => "".to_string(),
            };

            let os_selector = Selector::parse("a.build-title").unwrap();
            let ga_label = match element.select(&os_selector).next() {
                Some(o) => {
                    let ga_label = o
                        .value()
                        .attr("ga_label")
                        .unwrap()
                        .to_string()
                        .to_lowercase();

                    // There are some invisible elements that are just links to sha256 hashes
                    // We don't need these so we just continue to the next element
                    if ga_label.contains("sha256") {
                        continue;
                    }

                    ga_label
                }
                None => continue,
            };

            let os_name: &str;
            if ga_label.contains(&"windows") {
                os_name = "windows";
            } else if ga_label.contains(&"darwin") {
                os_name = "darwin";
            } else if ga_label.contains(&"linux") {
                os_name = "linux";
            } else {
                os_name = "unknown";
            }

            let build_platform = element
                .select(
                    &Selector::parse(".build-meta span.build-architecture[title='Architecture']")
                        .unwrap(),
                )
                .next()
                .unwrap()
                .text()
                .collect::<String>()
                .to_lowercase();

            let os_arch: &str;
            if build_platform == "windows x64"
                || build_platform == "macos intel"
                || build_platform == "linux x64"
            {
                os_arch = "x86_64";
            } else if build_platform == "macos apple silicon" {
                os_arch = "arm64";
            } else {
                os_arch = "unknown";
            }

            release_list.0.releases.push(BlenderRelease::new(
                version,
                version_detail,
                download_link,
                download_type,
                download_size,
                release_date,
                tag,
                os_name.to_string(),
                os_arch.to_string(),
                sha,
                ga_label,
            ));
        }

        release_list
    }
}

pub fn scrape(tag: String) -> CustomBlenderReleaseList {
    let data = reqwest::blocking::get(format!("https://builder.blender.org/download/{}/", tag))
        .unwrap()
        .text()
        .unwrap();
    let document = Html::parse_document(&data);

    let download_selector = Selector::parse(".builds-list > li").unwrap();
    let elements = document
        .select(&download_selector)
        .collect::<Vec<ElementRef>>();
    let download_list = CustomBlenderReleaseList::from(elements);

    download_list
}

pub fn scrape_stable() -> CustomBlenderReleaseList {
    let data = reqwest::blocking::get("https://www.blender.org/download/")
        .unwrap()
        .text()
        .unwrap();
    let document = Html::parse_document(&data);

    let download_selector = Selector::parse("#menu-other-platforms li.os").unwrap();

    let elements = document
        .select(&download_selector)
        .collect::<Vec<ElementRef>>();

    let mut releases = vec![];

    for element in elements {
        let download_size = match element
            .select(&Selector::parse("span.size").unwrap())
            .next()
        {
            Some(size) => size.text().collect::<String>(),
            None => continue,
        };

        let download_link = element
            .select(&Selector::parse("a").unwrap())
            .next()
            .unwrap()
            .value()
            .attr("href")
            .unwrap()
            .to_string();

        let mut link_split = download_link.split("/");
        let file_name = link_split.nth(link_split.clone().count() - 2).unwrap();
        let version = match file_name.split("-").nth(1) {
            Some(version) => {
                let v = version.split(".").collect::<Vec<&str>>();
                vec![
                    v[0].parse::<i8>().unwrap(),
                    v[1].parse::<i8>().unwrap(),
                    v[2].parse::<i8>().unwrap(),
                ]
            }
            None => continue,
        };

        let os_name: &str;
        let release_date_selector: Selector;

        if element.value().attr("class").unwrap().contains("windows") {
            os_name = "windows";
        } else if element.value().attr("class").unwrap().contains("linux") {
            os_name = "linux";
        } else if element.value().attr("class").unwrap().contains("mac") {
            os_name = "mac";
        } else {
            os_name = "unknown";
        }

        // Get the arch from the build span
        let arch_raw = match element
            .select(&Selector::parse("span.build").unwrap())
            .next()
        {
            Some(arch) => arch.text().collect::<String>(),
            None => "".into(),
        };

        let os_arch: &str;
        let sha_selector: Selector;
        if arch_raw == "Apple Silicon" {
            os_arch = "arm64";
            release_date_selector =
                Selector::parse("#menu-info-macos-apple-silicon > small").unwrap();
            sha_selector =
                Selector::parse("#menu-info-macos-apple-silicon > small.checksum a").unwrap();
        } else {
            if os_name == "linux" {
                release_date_selector = Selector::parse("#menu-info-linux > small").unwrap();
                sha_selector = Selector::parse("#menu-info-linux > small.checksum a").unwrap();
            } else if os_name == "darwin" {
                release_date_selector = Selector::parse("#menu-info-macos > small").unwrap();
                sha_selector = Selector::parse("#menu-info-macos > small.checksum a").unwrap();
            } else {
                release_date_selector = Selector::parse("#menu-info-windows > small").unwrap();
                sha_selector = Selector::parse("#menu-info-windows > small.checksum a").unwrap();
            }
            os_arch = "x86_64";
        }

        let sha256 = match document.select(&sha_selector).nth(1) {
            Some(sha) => sha.value().attr("href").unwrap().to_string(),
            None => "".to_string(),
        };

        // Get the release date from the menu
        let release_date = match document.select(&release_date_selector).next() {
            Some(release_date) => NaiveDate::parse_from_str(
                &release_date.text().collect::<String>(),
                "Released on %B %d, %Y · ",
            )
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
            None => Utc::now().naive_utc(),
        };

        // Extract the file type from the last part of the download link
        let download_type = match download_link.split(".").last() {
            Some(download_type) => download_type
                .to_string()
                .strip_suffix("/")
                .unwrap()
                .to_string(),
            None => "".into(),
        };

        releases.push(BlenderRelease {
            os: os_name.into(),
            download_link: download_link,
            version: version,
            download_size: download_size,
            sha256: sha256,
            arch: os_arch.into(),
            download_type: download_type,
            ga_label: "".into(),
            release_date: release_date,
            tag: "current-stable".into(),
            version_detail: "".into(),
        });
    }

    CustomBlenderReleaseList { 0: BlenderReleaseList { releases: releases } }
}

fn main() {
    let download_list = HashMap::from([
        ("stable", scrape_stable()),
        ("daily", scrape("daily".to_string())),
        ("experimental", scrape("experimental".to_string())),
        ("patch", scrape("patch".to_string())),
    ]);
    let json = serde_json::to_string(&download_list).unwrap();
    println!("{}", json);
}
