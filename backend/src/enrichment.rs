use axum::http::HeaderMap;
use maxminddb::{geoip2, Reader};
use std::{net::IpAddr, path::Path};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GeoLocation {
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub continent: Option<String>,
}

impl GeoLocation {
    fn is_empty(&self) -> bool {
        self.country_code.is_none()
            && self.region.is_none()
            && self.city.is_none()
            && self.continent.is_none()
    }
}

pub struct GeoIp {
    reader: Reader<Vec<u8>>,
}

impl GeoIp {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, maxminddb::MaxMindDbError> {
        Ok(Self {
            reader: Reader::open_readfile(path)?,
        })
    }

    pub fn lookup(&self, ip: IpAddr) -> Option<GeoLocation> {
        let result = self.reader.lookup(ip).ok()?;
        let city = result.decode::<geoip2::City>().ok()??;
        let location = GeoLocation {
            country_code: city.country.iso_code.map(str::to_owned),
            region: city
                .subdivisions
                .first()
                .and_then(|subdivision| subdivision.iso_code.or(subdivision.names.english))
                .map(str::to_owned),
            city: city.city.names.english.map(str::to_owned),
            continent: city.continent.code.map(str::to_owned),
        };
        (!location.is_empty()).then_some(location)
    }
}

pub fn location_from_headers(headers: &HeaderMap, trust_proxy: bool) -> Option<GeoLocation> {
    if !trust_proxy {
        return None;
    }
    let location = GeoLocation {
        country_code: header_value(headers, "cf-ipcountry", 2, 2)
            .filter(|value| value.bytes().all(|byte| byte.is_ascii_alphabetic()))
            .map(|value| value.to_ascii_uppercase()),
        region: header_value(headers, "cf-region-code", 1, 64).map(str::to_owned),
        city: header_value(headers, "cf-ipcity", 1, 128).map(str::to_owned),
        continent: header_value(headers, "cf-ipcontinent", 2, 2)
            .filter(|value| value.bytes().all(|byte| byte.is_ascii_alphabetic()))
            .map(|value| value.to_ascii_uppercase()),
    };
    (!location.is_empty()).then_some(location)
}

fn header_value<'a>(
    headers: &'a HeaderMap,
    name: &'static str,
    min: usize,
    max: usize,
) -> Option<&'a str> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| (min..=max).contains(&value.len()))
        .filter(|value| {
            value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'-' | b'_' | b'.')
            })
        })
}
