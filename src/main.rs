// blender_org::scrape() returns a Vec<Version> , which is a list of versions of blender. Each version has a version number, a download link, and a release date.
// The version number is a String, the download link is a String, and the release date is a chrono::NaiveDate.

use chrono::NaiveDateTime;
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

            // Get the build meta div element
            let build_meta_selector = Selector::parse("div.build-meta").unwrap();
            let build_meta = match element.select(&build_meta_selector).next() {
                Some(build_meta) => build_meta,
                None => continue,
            };

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
            let download_link_installer = match element.select(&download_installer_selector).next()
            {
                Some(download_link) => download_link.value().attr("href").unwrap().to_string(),
                None => continue,
            };

            // Extract the download link for the build archive from the build meta element
            let download_archive_selector = Selector::parse("a").unwrap();
            let download_link_archive = match build_meta.select(&download_archive_selector).next() {
                Some(download_link) => download_link.value().attr("href").unwrap().to_string(),
                None => continue,
            };

            // Extract the download size from the li element with the title "File size"
            let download_size_selector = Selector::parse("li[title='File size']").unwrap();
            let download_size = match element.select(&download_size_selector).next() {
                Some(download_size) => download_size.text().collect::<String>(),
                None => continue,
            };

            // Extract the version number and build details from the filename of the download link
            let filename = download_link_installer
                .split("/")
                .last()
                .unwrap()
                .to_string();
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
                None => "".to_string()
            };

            let os_selector = Selector::parse("a.build-title").unwrap();
            let ga_label = match element.select(&os_selector).next() {
                Some(o) => {
                    let ga_label = o.value().attr("ga_label").unwrap().to_string().to_lowercase();
                    
                    // There are some invisible elements that are just links to sha256 hashes
                    // We don't need these so we just continue to the next element
                    if ga_label.contains("sha256") {
                        continue;
                    }

                    ga_label
                },
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

            let os_arch: &str;
            if ga_label.contains(&"64bit") {
                os_arch = "64bit";
            } else if ga_label.contains(&"32bit") {
                os_arch = "32bit";
            } else {
                os_arch = "unknown";
            }

            release_list.0.releases.push(BlenderRelease::new(
                version,
                version_detail,
                download_link_installer,
                download_link_archive,
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

fn main() {
    let download_list = HashMap::from([
        ("daily", scrape("daily".to_string())),
        ("experimental", scrape("experimental".to_string())),
        ("patch", scrape("patch".to_string())),
    ]);
    let json = serde_json::to_string(&download_list).unwrap();
    println!("{}", json);
}
