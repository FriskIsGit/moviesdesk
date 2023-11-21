use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{BufReader, Write};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Series {
    pub id: u32,
    pub name: String,
    pub original_language: String,
    pub overview: String,
    pub popularity: f32,
    pub poster_path: Option<String>,
    pub first_air_date: String,
    pub vote_average: f32,
    pub adult: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Movie {
    pub id: u32,
    pub title: String,
    pub original_language: String,
    pub overview: String,
    pub popularity: f32,
    pub poster_path: Option<String>,
    pub release_date: String,
    pub vote_average: f32,
    pub vote_count: u32,
    pub adult: bool,
}

#[derive(Clone)]
pub enum Production {
    Movie(Movie),
    Series(Series),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductionIds {
    pub id: u32,
    pub facebook_id: Option<String>,
    pub freebase_id: Option<String>,
    pub freebase_mid: Option<String>,
    pub imdb_id: Option<String>,
    pub instagram_id: Option<String>,
    pub tvdb_id: Option<u32>,
    pub tvrage_id: Option<u32>,
    pub twitter_id: Option<String>,
    pub wikidata_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trailer {
    pub name: String,
    pub key: String,
    pub published_at: String,
    pub site: String,
    pub size: u32,
    pub official: bool,
}

impl Trailer {
    pub fn youtube_url(&self) -> String {
        format!("https://youtube.com/watch?v={}", self.key)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMovie {
    pub movie: Movie,
    pub user_rating: f32,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSeries {
    pub series: Series,
    pub user_rating: f32,
    pub note: String,
    pub season_notes: Vec<SeasonNotes>,
}

impl UserSeries {
    pub fn ensure_seasons(&mut self, len: usize) {
        if self.season_notes.len() >= len {
            return;
        }
        let fill = len - self.season_notes.len();
        for _ in 0..fill {
            self.season_notes.push(SeasonNotes::new());
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeasonNotes {
    pub note: String,
    pub episode_notes: Vec<String>,
}

impl SeasonNotes {
    pub fn new() -> Self {
        Self {
            note: "".into(),
            episode_notes: Vec::new(),
        }
    }
    pub fn ensure_episodes(&mut self, len: usize) {
        if self.episode_notes.len() >= len {
            return;
        }
        let fill = len - self.episode_notes.len();
        for _ in 0..fill {
            self.episode_notes.push("".into());
        }
    }
}

pub fn serialize_user_productions(user_series: &[UserSeries], user_movies: &[UserMovie]) -> Result<(), String> {
    let john = json!({
        "series": user_series,
        "movies": user_movies
    });
    let serialized_json = serde_json::to_string(&john).expect("Failed to serialize JSON");
    let temp_path = "res/user_prod_temp.json";
    let mut file = match File::create(temp_path) {
        Ok(file_handle) => file_handle,
        Err(err) => return Err(err.to_string()),
    };

    if let Err(err) = file.write(serialized_json.as_bytes()) {
        return Err(err.to_string());
    }

    // Write to a file, or write to a temp file then move files.
    let path = "res/user_prod.json";
    match std::fs::rename(temp_path, path) {
        Err(err) => Err(err.to_string()),
        Ok(_) => Ok(()),
    }
}

pub fn deserialize_user_productions(path: Option<String>) -> Result<(Vec<UserSeries>, Vec<UserMovie>), String> {
    let path = match path {
        Some(s) => s,
        None => "res/user_prod.json".into(),
    };
    let file = match File::open(path) {
        Ok(file_handle) => file_handle,
        Err(err) => return Err(err.to_string()),
    };
    let reader = BufReader::new(file);
    let mut json: Value = serde_json::from_reader(reader).expect("Failed on read from memory");
    let series_arr = json["series"].take();
    let movies_arr = json["movies"].take();
    let user_series = match serde_json::from_value(series_arr) {
        Ok(vec_value) => vec_value,
        Err(err) => return Err(err.to_string()),
    };
    let user_movies = match serde_json::from_value(movies_arr) {
        Ok(vec_value) => vec_value,
        Err(err) => return Err(err.to_string()),
    };
    Ok((user_series, user_movies))
}

/*
Serialization:
user_prod.json
{
    "series":[
        {UserSeries}
        {UserSeries}
    ]
    "movies":[
        {UserMovie}
        {UserMovie}
    ]
}
*/

type ProductionId = u32;

#[derive(Default, Copy, Clone)]
pub enum EntryType {
    Movie(ProductionId),
    Series(ProductionId),
    #[default]
    None,
}

// NOTE: Central list could hold UserProduction instead that is displayed on top of the right panel maybe?
#[derive(Clone)]
pub struct ListEntry {
    pub production_id: EntryType,

    // NOTE: Those below could be references to the item from the UserProduction?
    pub name: String,
    pub poster_path: Option<String>, // Shouldn't be an option, should always have a fallback image btw.
    pub rating: f32,
}

impl ListEntry {
    pub fn from_movie(movie: &Movie) -> Self {
        Self {
            production_id: EntryType::Movie(movie.id),

            name: movie.title.clone(),
            poster_path: movie.poster_path.clone(),
            rating: movie.vote_average,
        }
    }

    pub fn from_series(series: &Series) -> Self {
        Self {
            production_id: EntryType::Series(series.id),

            name: series.name.clone(),
            poster_path: series.poster_path.clone(),
            rating: series.vote_average,
        }
    }

    pub fn is_selected(&self, entry: &EntryType) -> bool {
        match entry {
            EntryType::Movie(selected_id) => {
                let EntryType::Movie(list_entry_id) = &self.production_id else {
                    return false;
                };
                selected_id == list_entry_id
            }
            EntryType::Series(selected_id) => {
                let EntryType::Series(list_entry_id) = &self.production_id else {
                    return false;
                };
                selected_id == list_entry_id
            }
            EntryType::None => false,
        }
    }
}

pub enum CentralListOrdering {
    // This will require to store the list separately
    UserDefined,
    Alphabetic,
    RatingAscending,
    RatingDescending,

    // TODO(maybe?):
    // UserRatingAscending,
    // UserRatingDescending,
    // WatchedFirst,
    // UnwatchedFirst,
    // Favourites,
    // WatchTime,
}
