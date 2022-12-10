//! A way of caching your input from the great and popular [Advent of Code].
//!
//! This is an attempt to reduce server load for the creator.
//!
//! Downloads using [`ureq`][ureq], stores cache in temporary files using
//! [`scratch`][scratch].
//!
//! # Example
//!
//! ```
//! use aoc_cache::get;
//! // my.cookie is a file containing the cookie string.
//! const MY_COOKIE: &str = include_str!("my.cookie");
//! let input: Result<String, aoc_cache::Error> = // Grabs from web if it's the first run
//!     get("https://adventofcode.com/2022/day/1/input", MY_COOKIE);
//! let input: Result<String, aoc_cache::Error> = // Grabs from cache
//!     get("https://adventofcode.com/2022/day/1/input", MY_COOKIE);
//! ```
//!
//! [Advent of Code]: https://adventofcode.com/
//! [ureq]: https://docs.rs/ureq/
//! [scratch]: https://docs.rs/scratch/

mod error;

pub use error::Error;

use cookie_store::{Cookie, CookieStore};
use std::{
    collections::hash_map::DefaultHasher,
    fs::{read_to_string, File, OpenOptions},
    hash::{Hash, Hasher},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};
use tracing::{debug, error, info, instrument, trace, warn};
use ureq::AgentBuilder;

type Result<T> = std::result::Result<T, Error>;

const INDEX_FILE_NAME: &str = "index.cache";
const TEMP_DIR_NAME: &str = "aoc_cache";
const USER_AGENT: &str = "https://github.com/glennib/aoc-cache by glennib.pub@gmail.com";

/// Gets input from the url or from cache if it has been retrieved before.
///
/// The url can be, e.g., <https://adventofcode.com/2022/day/1/input>. The cookie must be one
/// retrieved by entering the site in your browser and inspecting network traffic. Instructions on
/// how to retrieve the cookie can be found [here][github-cookie-example] or [here][google-cookie].
/// The cookie should look like this: `session=abcd...` without a trailing newline.
///
/// # Example
///
/// ```
/// use aoc_cache::get;
/// // my.cookie is a file containing the cookie string.
/// const MY_COOKIE: &str = include_str!("my.cookie");
/// let input: Result<String, aoc_cache::Error> =
///     get("https://adventofcode.com/2022/day/1/input", MY_COOKIE);
/// ```
///
/// [github-cookie-example]: https://github.com/wimglenn/advent-of-code-wim/issues/1
/// [google-cookie]: https://www.google.com/search?q=adventofcode+cookie
#[instrument(skip(cookie))]
pub fn get(url: &str, cookie: &str) -> Result<String> {
    if let Some(content) = get_cache_for_url(url)? {
        info!("returning content found in cache");
        return Ok(content);
    }
    debug!("content not found in cache, requesting from web");
    let content = get_content_from_web(url, cookie)?;
    add_cache(url, &content)?;
    info!("returning content from web");
    Ok(content)
}

/// Dispatches to the `get`-function, see its documentation.
#[deprecated]
pub fn get_input_from_web_or_cache(url: &str, cookie: &str) -> Result<String> {
    get(url, cookie)
}

#[instrument(skip(url, cookie))]
fn get_content_from_web(url: &str, cookie: &str) -> Result<String> {
    if cookie.is_empty() {
        return Err(Error::InvalidCookie(
            "empty cookie is not valid".to_string(),
        ));
    }

    let url_parsed = url.parse()?;

    let jar = CookieStore::load(BufReader::new(cookie.as_bytes()), |s| {
        trace!(s, "parsed cookie from str");
        Cookie::parse(s, &url_parsed).map(Cookie::into_owned)
    })
    .map_err(|_| Error::CookieParse("couldn't create cookie store".into()))?;

    debug!(?jar);

    let agent = AgentBuilder::new()
        .cookie_store(jar)
        .user_agent(USER_AGENT)
        .build();
    let response = agent.get(url).call()?;
    let content = response.into_string()?.trim().to_string();
    Ok(content)
}

#[instrument]
fn create_or_get_cache_dir() -> PathBuf {
    let cache_dir = scratch::path(TEMP_DIR_NAME);
    debug!(?cache_dir);
    cache_dir
}

#[instrument(skip(url))]
fn get_cache_for_url(url: &str) -> Result<Option<String>> {
    let cache_file_path = get_cache_file_path_from_index(url)?;
    match cache_file_path {
        None => Ok(None),
        Some(path) => {
            debug!("cache_file_path={}", path.to_str().unwrap());
            Ok(Some(read_to_string(path)?))
        }
    }
}

#[instrument(skip(url))]
fn encode_url(url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();
    hash.to_string()
}

#[instrument(skip(url))]
fn filename_from_url(url: &str) -> String {
    let mut filename = String::from("cache_");
    filename.push_str(&encode_url(url));
    filename.push_str(".cache");
    filename
}

#[instrument(skip(url, content))]
fn add_cache(url: &str, content: &str) -> Result<()> {
    let cache_file_path = get_cache_file_path_from_index(url)?;
    if cache_file_path.is_some() {
        error!("found cache entry for {url} when attempting to add new cache for it");
        return Err(Error::Duplicate(format!(
            "found cache entry for {url} when attempting to add new cache for it"
        )));
    }

    let cache_dir = create_or_get_cache_dir();
    let cache_filename = filename_from_url(url);
    let cache_file_path = cache_dir.join(cache_filename);

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&cache_file_path)?;
    write!(file, "{content}")?;
    info!(
        "Wrote content (size={}) to {cache_file_path:?}",
        content.len()
    );

    let index_path = create_index_if_non_existent()?;
    let mut file = OpenOptions::new().append(true).open(&index_path)?;
    let cache_file_path_str = cache_file_path.to_str();
    match cache_file_path_str {
        None => {
            error!(?cache_file_path, "cannot convert to str");
            return Err(Error::Path("Cache file path was empty".to_string()));
        }
        Some(cache_file_path_str) => {
            let index_line = format!("{url}: {}", cache_file_path_str);
            writeln!(file, "{index_line}")?;
            info!("Wrote `{index_line}` to {index_path:?}");
        }
    }
    Ok(())
}

#[instrument]
fn create_file_if_non_existent(path: &Path) -> Result<()> {
    if Path::new(path).exists() {
        debug!("file already existed, doing nothing");
    } else {
        info!("file didn't exist, creating");
        File::create(path)?;
    }
    Ok(())
}

#[instrument]
fn create_index_if_non_existent() -> Result<PathBuf> {
    let cache_dir = create_or_get_cache_dir();
    let index_path = cache_dir.join(INDEX_FILE_NAME);
    create_file_if_non_existent(&index_path)?;
    Ok(index_path)
}

#[instrument(skip(url))]
fn get_cache_file_path_from_index(url: &str) -> Result<Option<PathBuf>> {
    let index_path = create_index_if_non_existent()?;
    let file = File::open(index_path)?;
    for line in BufReader::new(file).lines() {
        let line = line?;
        let parts: Vec<_> = line.split(": ").collect();
        if parts.len() != 2 {
            return Err(Error::Parse(format!("could not parse index line `{line}`")));
        }
        let url_in_line = parts[0];
        if url_in_line == url {
            let cache_file_path = parts[1].to_string();
            debug!(cache_file_path, "from index");
            return Ok(Some(cache_file_path.into()));
        }
    }
    Ok(None)
}
