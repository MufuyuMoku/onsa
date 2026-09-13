//! Smart playlists: rules in, parameterised SQL out (SPEC §6.3).
//!
//! The rules are stored as JSON and turned into one `SELECT` with bound
//! parameters. **No value from the listener is ever put into the SQL text**:
//! the field decides which column expression is used, the operator decides
//! the comparison, and the value is always a parameter. A rule set that
//! cannot be understood is refused rather than guessed at.

use rusqlite::types::Value;
use rusqlite::ToSql;
use serde::{Deserialize, Serialize};

use crate::db::Library;
use crate::error::{Error, Result};
use crate::search::{track_columns, track_row, TrackRow};

/// Whether a track has to match every rule or any of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Match {
    /// Every rule.
    #[default]
    All,
    /// At least one rule.
    Any,
}

/// What a rule looks at (SPEC §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    /// Track title.
    Title,
    /// Track artist.
    Artist,
    /// Album title.
    Album,
    /// Album artist.
    AlbumArtist,
    /// Genre.
    Genre,
    /// Year.
    Year,
    /// Rating, 0 to 5.
    Rating,
    /// How many times it was played.
    PlayCount,
    /// When it was last played.
    LastPlayed,
    /// When it was added to the library.
    AddedAt,
    /// Container or codec.
    Codec,
    /// Length in seconds.
    Duration,
    /// The folder the file sits in.
    Folder,
}

/// The kinds of value a field holds, which decide the operators it takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Words.
    Text,
    /// Numbers.
    Number,
    /// Points in time.
    Date,
}

impl Field {
    /// What kind of value this field holds.
    pub fn kind(self) -> Kind {
        match self {
            Self::Title
            | Self::Artist
            | Self::Album
            | Self::AlbumArtist
            | Self::Genre
            | Self::Codec
            | Self::Folder => Kind::Text,
            Self::Year | Self::Rating | Self::PlayCount | Self::Duration => Kind::Number,
            Self::LastPlayed | Self::AddedAt => Kind::Date,
        }
    }

    /// The column expression this field reads. Fixed text, never built from
    /// anything the listener typed.
    fn column(self) -> &'static str {
        match self {
            Self::Title => "v.title",
            Self::Artist => "v.artist",
            Self::Album => "v.album",
            Self::AlbumArtist => "v.album_artist",
            Self::Genre => "v.genre",
            Self::Year => "v.year",
            Self::Rating => "COALESCE(s.rating, 0)",
            Self::PlayCount => "COALESCE(s.play_count, 0)",
            Self::LastPlayed => "s.last_played",
            Self::AddedAt => "v.added_at",
            Self::Codec => "v.codec",
            // Seconds, so the rules speak the listener's units.
            Self::Duration => "(v.duration_ms / 1000.0)",
            Self::Folder => "onsa_parent(v.path)",
        }
    }
}

/// How a rule compares (SPEC §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    /// Text holds this.
    Contains,
    /// Text does not hold this.
    NotContains,
    /// Text is exactly this.
    Is,
    /// Text is anything but this.
    IsNot,
    /// Text begins with this.
    StartsWith,
    /// Text does not begin with this.
    NotStartsWith,
    /// Equal.
    #[serde(rename = "=")]
    Eq,
    /// Not equal.
    #[serde(rename = "!=")]
    Ne,
    /// Less than.
    #[serde(rename = "<")]
    Lt,
    /// At most.
    #[serde(rename = "<=")]
    Le,
    /// More than.
    #[serde(rename = ">")]
    Gt,
    /// At least.
    #[serde(rename = ">=")]
    Ge,
    /// Between two numbers, both included.
    Between,
    /// Within the last so many days.
    InLast,
    /// Not within the last so many days.
    NotInLast,
    /// Before a moment in time.
    Before,
    /// After a moment in time.
    After,
}

impl Op {
    /// Which kind of field this operator belongs to.
    fn kind(self) -> Kind {
        match self {
            Self::Contains
            | Self::NotContains
            | Self::Is
            | Self::IsNot
            | Self::StartsWith
            | Self::NotStartsWith => Kind::Text,
            Self::Eq | Self::Ne | Self::Lt | Self::Le | Self::Gt | Self::Ge | Self::Between => {
                Kind::Number
            }
            Self::InLast | Self::NotInLast | Self::Before | Self::After => Kind::Date,
        }
    }
}

/// One rule: a field, how it is compared, and what it is compared with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    /// What the rule looks at.
    pub field: Field,
    /// How it compares.
    pub op: Op,
    /// What it compares with: a string, a number, `[a, b]`, or `{"days": n}`.
    pub value: serde_json::Value,
}

/// How the result is ordered (SPEC §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    /// By title.
    Title,
    /// By artist, then album, then track number.
    Artist,
    /// By album, then track number.
    Album,
    /// By year.
    Year,
    /// By length.
    Duration,
    /// By rating.
    Rating,
    /// By how often it was played.
    PlayCount,
    /// By when it was last played.
    LastPlayed,
    /// By when it was added.
    AddedAt,
    /// In no order at all, freshly shuffled each time.
    Random,
}

impl SortField {
    fn order(self, descending: bool) -> String {
        let direction = if descending { "DESC" } else { "ASC" };
        let then = "v.album COLLATE NOCASE, v.disc_number, v.track_number, v.title COLLATE NOCASE";
        match self {
            Self::Title => format!("v.title COLLATE NOCASE {direction}"),
            Self::Artist => format!("v.artist COLLATE NOCASE {direction}, {then}"),
            Self::Album => {
                format!("v.album COLLATE NOCASE {direction}, v.disc_number, v.track_number")
            }
            Self::Year => format!("v.year {direction}, {then}"),
            Self::Duration => format!("v.duration_ms {direction}, v.title COLLATE NOCASE"),
            Self::Rating => format!("COALESCE(s.rating, 0) {direction}, {then}"),
            Self::PlayCount => format!("COALESCE(s.play_count, 0) {direction}, {then}"),
            Self::LastPlayed => format!("s.last_played {direction}, {then}"),
            Self::AddedAt => format!("v.added_at {direction}, {then}"),
            Self::Random => "RANDOM()".to_string(),
        }
    }
}

/// The order a smart playlist comes back in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sort {
    /// What to order by.
    pub field: SortField,
    /// Largest first.
    #[serde(default)]
    pub descending: bool,
}

impl Default for Sort {
    fn default() -> Self {
        Self {
            field: SortField::Artist,
            descending: false,
        }
    }
}

/// A whole rule set, as stored with the playlist (SPEC §6.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rules {
    /// Whether every rule has to match, or any of them.
    #[serde(rename = "match", default)]
    pub match_all: Match,
    /// The rules themselves. None at all matches the whole library.
    #[serde(default)]
    pub rules: Vec<Rule>,
    /// What order to bring them back in.
    #[serde(default)]
    pub sort: Sort,
    /// How many at most.
    #[serde(default)]
    pub limit: Option<usize>,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            match_all: Match::All,
            rules: Vec::new(),
            sort: Sort::default(),
            limit: None,
        }
    }
}

/// The most tracks a smart playlist will ever return, so a missing limit
/// cannot load the whole library into the interface at once.
const MAX_ROWS: usize = 10_000;

/// One compiled rule set: the SQL and the parameters it expects.
#[derive(Debug, Clone, PartialEq)]
pub struct Compiled {
    /// The statement, with `?` placeholders.
    pub sql: String,
    /// The values to bind, in order.
    pub params: Vec<Value>,
}

impl Rules {
    /// Reads a rule set from the JSON it is stored as.
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Writes a rule set as the JSON it is stored as.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Turns the rules into one parameterised statement (SPEC §6.3).
    pub fn compile(&self, now_ms: i64) -> Result<Compiled> {
        let mut params: Vec<Value> = Vec::new();
        let mut tests: Vec<String> = Vec::new();
        for rule in &self.rules {
            tests.push(compile_rule(rule, now_ms, &mut params)?);
        }

        let where_clause = if tests.is_empty() {
            String::new()
        } else {
            let joiner = match self.match_all {
                Match::All => " AND ",
                Match::Any => " OR ",
            };
            format!("WHERE {}", tests.join(joiner))
        };

        let limit = self.limit.unwrap_or(MAX_ROWS).clamp(1, MAX_ROWS);
        params.push(Value::Integer(limit as i64));

        let sql = format!(
            "SELECT {} FROM track_view v
             LEFT JOIN stats s ON s.track_id = v.id
             {where_clause}
             ORDER BY {}
             LIMIT ?",
            track_columns("v."),
            self.sort.field.order(self.sort.descending)
        );
        Ok(Compiled { sql, params })
    }
}

/// Builds the test for one rule, adding its values to `params`.
fn compile_rule(rule: &Rule, now_ms: i64, params: &mut Vec<Value>) -> Result<String> {
    let kind = rule.field.kind();
    if rule.op.kind() != kind {
        return Err(Error::Invalid(format!(
            "the {:?} field does not take the {:?} operator",
            rule.field, rule.op
        )));
    }
    let column = rule.field.column();

    match rule.op {
        Op::Contains | Op::NotContains | Op::StartsWith | Op::NotStartsWith => {
            let text = as_text(&rule.value, rule.field)?;
            let pattern = match rule.op {
                Op::StartsWith | Op::NotStartsWith => format!("{}%", escape_like(&text)),
                _ => format!("%{}%", escape_like(&text)),
            };
            params.push(Value::Text(pattern));
            let negated = matches!(rule.op, Op::NotContains | Op::NotStartsWith);
            // A track with no value at all does not "contain" anything, but
            // it does count as not containing it.
            Ok(if negated {
                format!("COALESCE({column}, '') NOT LIKE ? ESCAPE '\\'")
            } else {
                format!("{column} LIKE ? ESCAPE '\\'")
            })
        }
        Op::Is | Op::IsNot => {
            params.push(Value::Text(as_text(&rule.value, rule.field)?));
            Ok(if rule.op == Op::Is {
                format!("{column} = ? COLLATE NOCASE")
            } else {
                format!("COALESCE({column}, '') <> ? COLLATE NOCASE")
            })
        }
        Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge => {
            params.push(Value::Real(as_number(&rule.value, rule.field)?));
            let comparison = match rule.op {
                Op::Eq => "=",
                Op::Ne => "<>",
                Op::Lt => "<",
                Op::Le => "<=",
                Op::Gt => ">",
                _ => ">=",
            };
            Ok(format!("{column} {comparison} ?"))
        }
        Op::Between => {
            let (low, high) = as_pair(&rule.value, rule.field)?;
            params.push(Value::Real(low.min(high)));
            params.push(Value::Real(low.max(high)));
            Ok(format!("{column} BETWEEN ? AND ?"))
        }
        Op::InLast | Op::NotInLast => {
            let days = as_days(&rule.value, rule.field)?;
            let since = now_ms - (days * 24 * 60 * 60 * 1000);
            params.push(Value::Integer(since));
            // Never played is never "in the last thirty days", and always
            // counts as not in them.
            Ok(if rule.op == Op::InLast {
                format!("{column} >= ?")
            } else {
                format!("COALESCE({column}, 0) < ?")
            })
        }
        Op::Before | Op::After => {
            params.push(Value::Integer(as_time(&rule.value, rule.field)?));
            Ok(if rule.op == Op::Before {
                format!("COALESCE({column}, 0) < ?")
            } else {
                format!("{column} > ?")
            })
        }
    }
}

fn wrong(field: Field, wanted: &str) -> Error {
    Error::Invalid(format!("the {field:?} rule needs {wanted}"))
}

fn as_text(value: &serde_json::Value, field: Field) -> Result<String> {
    match value {
        serde_json::Value::String(text) => Ok(text.clone()),
        serde_json::Value::Number(number) => Ok(number.to_string()),
        _ => Err(wrong(field, "a word to compare with")),
    }
}

fn as_number(value: &serde_json::Value, field: Field) -> Result<f64> {
    value.as_f64().ok_or_else(|| wrong(field, "a number"))
}

fn as_pair(value: &serde_json::Value, field: Field) -> Result<(f64, f64)> {
    let pair = value
        .as_array()
        .filter(|items| items.len() == 2)
        .ok_or_else(|| wrong(field, "two numbers"))?;
    Ok((as_number(&pair[0], field)?, as_number(&pair[1], field)?))
}

fn as_days(value: &serde_json::Value, field: Field) -> Result<i64> {
    let days = match value {
        serde_json::Value::Object(map) => map.get("days").and_then(serde_json::Value::as_i64),
        other => other.as_i64(),
    };
    days.filter(|days| *days >= 0)
        .ok_or_else(|| wrong(field, "a number of days"))
}

fn as_time(value: &serde_json::Value, field: Field) -> Result<i64> {
    value
        .as_i64()
        .ok_or_else(|| wrong(field, "a moment in time, in milliseconds"))
}

/// Escapes what LIKE would otherwise read as a wildcard.
fn escape_like(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

impl Library {
    /// The tracks a rule set matches, worked out from the library as it is
    /// right now (SPEC §6.3).
    pub fn smart_tracks(&self, rules: &Rules) -> Result<Vec<TrackRow>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_millis() as i64)
            .unwrap_or_default();
        let compiled = rules.compile(now)?;
        let mut statement = self.conn.prepare(&compiled.sql)?;
        let params: Vec<&dyn ToSql> = compiled
            .params
            .iter()
            .map(|value| value as &dyn ToSql)
            .collect();
        let rows = statement
            .query_map(params.as_slice(), track_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }
}
