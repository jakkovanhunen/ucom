use std::env;
use std::fs;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::SystemTime;

use anyhow::anyhow;
use chrono::{DateTime, Duration, TimeDelta, Utc};
use dirs::cache_dir;

static CACHE_ENABLED: OnceLock<bool> = OnceLock::new();

const CACHE_REFRESH_SECONDS: i64 = 3600;

enum CacheState {
    /// The cache is expired.
    Expired,
    /// The cache is still valid.
    Valid,
    /// The cache is still valid but needs to be refreshed.
    RefreshNeeded,
}

/// Gets the content of the given URL. Gets the content from the cache if it exists and is not too old.
pub(crate) fn get_cached_content(
    url: &str,
    check_for_remote_change: bool,
) -> anyhow::Result<String> {
    if !is_cache_enabled() {
        return ureq::get(url)
            .call()?
            .into_body()
            .read_to_string()
            .map_err(anyhow::Error::msg);
    }

    let cache_dir = get_cache_dir();
    let filename = cache_dir.join(sanitize_filename(url));

    match check_cache_state(url, &filename, check_for_remote_change)? {
        CacheState::Expired => download_and_cache(url, &filename, &cache_dir),
        CacheState::Valid => fs::read_to_string(&filename).map_err(anyhow::Error::msg),
        CacheState::RefreshNeeded => read_and_update_timestamp(&filename),
    }
}

/// Clears the cache.
pub(crate) fn clear_cache() {
    _ = fs::remove_dir_all(get_cache_dir());
}

/// Sets whether the cache is enabled or not.
/// This value can only be set once.
pub(crate) fn configure_cache(enabled: bool) -> anyhow::Result<()> {
    CACHE_ENABLED
        .set(enabled)
        .map_err(|_| anyhow!("Failed to set CACHE_ENABLED"))
}

/// Sets whether the cache is enabled or not based on environment variable `UCOM_ENABLE_CACHE`.
pub(crate) fn init_cache_from_env() -> anyhow::Result<()> {
    match env::var("UCOM_ENABLE_CACHE") {
        Ok(val) => configure_cache(val == "true" || val == "1"),
        Err(_) => Ok(()),
    }
}

/// Returns whether the cache is enabled or not.
pub(crate) fn is_cache_enabled() -> bool {
    *CACHE_ENABLED.get().unwrap_or(&true)
}

pub(crate) fn get_cache_dir() -> PathBuf {
    cache_dir()
        .expect("unable to get cache directory")
        .join("ucom")
}

/// Checks if the cached content is up-to-date.
fn check_cache_state(
    url: &str,
    cached: &Path,
    check_for_remote_change: bool,
) -> anyhow::Result<CacheState> {
    let state = if cached.exists() {
        let cached_time = cached.metadata()?.modified()?;
        let delta_time = Utc::now() - DateTime::<Utc>::from(cached_time);

        if delta_time <= TimeDelta::try_seconds(CACHE_REFRESH_SECONDS).unwrap() {
            // Local file is still new enough
            CacheState::Valid
        } else if check_for_remote_change && !is_remote_content_newer(url, &cached_time) {
            // Local file is newer than remote Last-Modified
            CacheState::RefreshNeeded
        } else {
            // Local file is out of date
            CacheState::Expired
        }
    } else {
        // Has no cache file
        CacheState::Expired
    };
    Ok(state)
}

/// Returns whether the cached file is expired.
pub(crate) fn is_expired(path: &Path) -> bool {
    let cached_time: DateTime<Utc> = match path.metadata().and_then(|m| m.modified()) {
        Ok(modified) => DateTime::<Utc>::from(modified),
        Err(_) => return true, // Cannot get modification time; consider expired
    };

    let delta_time = Utc::now() - cached_time;

    if delta_time < Duration::zero() {
        return true; // Modification time is in the future; consider expired
    }

    delta_time > Duration::seconds(CACHE_REFRESH_SECONDS)
}

/// Touches the timestamp of the given file.
pub(crate) fn update_timestamp(filename: &PathBuf) -> anyhow::Result<()> {
    // Update the local timestamp
    match fs::File::open(filename)?.set_modified(Utc::now().into()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            // If error is a permission error, do workaround by re-saving the file
            let content = fs::read_to_string(filename)?;
            fs::write(filename, &content)?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

/// Checks if the page has been updated since the given time.
fn is_remote_content_newer(url: &str, local_time: &SystemTime) -> bool {
    if let Ok(server_utc) = get_remote_modified_time(url) {
        DateTime::<Utc>::from(*local_time) < server_utc
    } else {
        // Always update if we couldn't check
        true
    }
}

fn sanitize_filename(filename: &str) -> String {
    filename.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
}

/// Downloads the content from the given URL and saves it to the given filename.
fn download_and_cache(url: &str, filename: &Path, cache_dir: &Path) -> anyhow::Result<String> {
    let content = ureq::get(url).call()?.into_body().read_to_string()?;
    create_dir_all(cache_dir).expect("unable to create cache directory");
    fs::write(filename, &content)?;
    Ok(content)
}

fn read_and_update_timestamp(filename: &PathBuf) -> anyhow::Result<String> {
    // Update the local timestamp
    match fs::File::open(filename)?.set_modified(Utc::now().into()) {
        Ok(()) => fs::read_to_string(filename).map_err(anyhow::Error::msg),
        Err(e) if e.raw_os_error() == Some(5) => {
            // If error is a permission error, do workaround by re-saving the file
            let content = fs::read_to_string(filename)?;
            fs::write(filename, &content)?;
            Ok(content)
        }
        Err(e) => Err(e.into()),
    }
}

fn get_remote_modified_time(url: &str) -> anyhow::Result<DateTime<Utc>> {
    let response = ureq::head(url).call()?;
    let lm = response
        .headers()
        .get("Last-Modified")
        .and_then(|s| s.to_str().ok())
        .ok_or_else(|| anyhow!("Last-Modified header not found in response from {}", url))?;
    let time = DateTime::parse_from_rfc2822(lm)?.with_timezone(&Utc);
    Ok(time)
}
