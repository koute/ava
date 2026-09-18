//! The report over chosen tournaments: how every agent did against the
//! tokens and the seconds it spent, as one document standing on its own,
//! the styles and the fonts inside it, so it reads the same saved as served.

use crate::{chart, views};
use ava_run::{docker, registry, runs, tournament, usage};

const TITLE: &str = "report";
const HEAD_END: &str = "</head>";
/// The tag of the layout loading the styles from the server, replaced by
/// the styles themselves.
const TAILWIND_TAG: &str = "<script src=\"/assets/tailwind.js\"></script>";
/// Where the layout loads the fonts from, replaced by the fonts themselves.
const FONT_ADDRESS_PREFIX: &str = "/assets/fonts/";
const FONT_DATA_PREFIX: &str = "data:font/woff2;base64,";
const BODY_CLASSES: &str = "bg-neutral-950 text-neutral-200 font-sans text-sm antialiased";
const MAIN_CLASSES: &str = "w-full px-6 py-6";
const HEADING_CLASSES: &str = "text-lg font-semibold text-neutral-100";
const HEADING_ROW_CLASSES: &str = "flex items-baseline gap-4";
const SUBTITLE_CLASSES: &str = "text-xs text-neutral-500 mt-1";
const FILE_PREFIX: &str = "report";
const FILE_SUFFIX: &str = ".html";
const NAME_SEPARATOR: &str = "-";
const DOWNLOAD_LABEL: &str = "download";
const NO_TOURNAMENTS_NOTE: &str = "no tournament chosen, check some on the tournaments page";
const NO_ROUNDS_NOTE: &str = "no finished round in the chosen tournaments";
const NO_MODELS_NOTE: &str = "no run in a finished round";
const PERCENT: f64 = 100.0;
const THOUSAND: f64 = 1_000.0;
const MILLION: f64 = 1_000_000.0;
const SECONDS_PER_MINUTE: f64 = 60.0;
/// Where an agent without a score or a price sorts: after every other.
const UNSCORED: f64 = -1.0;
const UNPRICED: f64 = f64::INFINITY;
const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const BASE64_PAD: char = '=';
const BASE64_BLOCK_BYTES: usize = 3;
const BASE64_BLOCK_CHARACTERS: usize = 4;
const BASE64_BITS: u32 = 6;

const MODEL_HEADER: &str = "*model";
/// The cells in front of the columns of a row: the model.
const MODEL_CELLS: usize = 1;
/// The hues a series is coloured from, hashed from the name of its model.
const HUES: u64 = 360;
/// The hosts of a run that are not a backend: its git remote and its scorer.
const GIT_HOST: &str = "git";
const SCORE_HOST: &str = "score";
/// The request a push to the git remote ends in.
const PUSH_PATH: &str = "/git-receive-pack";
/// The token counts a logged request carries, all of them through the backend.
const TOKEN_FIELDS: [&str; 4] = [
    "input_tokens",
    "output_tokens",
    "cache_read_tokens",
    "cache_write_tokens",
];
const MATRIX_TABLE: &str = "report-matrix";
const SPREAD_HEADER: &str = "#spread|the score of the best harness with the model minus that of the worst, how much the harness moves the outcome";
const POOLED_HEADER: &str =
    "#all harnesses|the share of the rounds won over every harness that drove the model";
/// The row pooling every model, showing what every harness did over all of them.
const EVERY_MODEL: &str = "every model";
const NO_MATRIX_NOTE: &str = "no model with a round against another agent";
/// The names the script keeps the chosen sort of the two tables under.
const COST_TABLE: &str = "report-cost";
const TIME_TABLE: &str = "report-time";

/// The columns of the table over what the agents got for their dollars and
/// tokens.
const COST_COLUMNS: [Column; 12] = [
    Column::Runs,
    Column::Passed,
    Column::Rounds,
    Column::Score,
    Column::Dollars,
    Column::DollarsPerRun,
    Column::DollarsPerPass,
    Column::DollarsPerRoundWon,
    Column::Output,
    Column::OutputPerRoundWon,
    Column::Input,
    Column::CacheRead,
];
/// The columns of the table over how the agents spent their seconds.
const TIME_COLUMNS: [Column; 12] = [
    Column::FirstPassSeconds,
    Column::FirstPassTokens,
    Column::HighScoreSeconds,
    Column::HighScoreTokens,
    Column::BudgetUsed,
    Column::Waiting,
    Column::FirstToken,
    Column::Burn,
    Column::Requests,
    Column::PeakContext,
    Column::Compactions,
    Column::Failed,
];
/// One measure of a group of runs, a column of the tables.
#[derive(Clone, Copy)]
enum Column {
    Runs,
    Passed,
    Rounds,
    Score,
    Dollars,
    DollarsPerRun,
    DollarsPerPass,
    DollarsPerRoundWon,
    Output,
    OutputPerRoundWon,
    Input,
    CacheRead,
    FirstPassSeconds,
    FirstPassTokens,
    HighScoreSeconds,
    HighScoreTokens,
    BudgetUsed,
    Waiting,
    FirstToken,
    Burn,
    Requests,
    PeakContext,
    Compactions,
    Failed,
}

impl Column {
    /// The header of the column, with its explanation behind the hover.
    fn header(self) -> &'static str {
        match self {
            Self::Runs => "#runs|the runs it played in the finished rounds",
            Self::Passed => "#passed|the runs a push of which passed the verifier",
            Self::Rounds => "#rounds|the rounds against other agents as won-drawn-lost",
            Self::Score => "#score|the share of those rounds won, half for a draw",
            Self::Dollars => "#$|the dollars of its runs at the prices of the registry",
            Self::DollarsPerRun => {
                "#$ per run|those dollars over the runs with a price, the runs without one behind the hover"
            }
            Self::DollarsPerPass => "#$ per pass|those dollars over the runs that passed",
            Self::DollarsPerRoundWon => {
                "#$ per round won|those dollars over the rounds it won, a draw counting half"
            }
            Self::Output => "#output|the output tokens of its runs",
            Self::OutputPerRoundWon => {
                "#output per round won|the output tokens over the rounds it won, a draw counting half"
            }
            Self::Input => "#input|the input tokens not read from the cache",
            Self::CacheRead => "#cache read|the input tokens read from the cache",
            Self::FirstPassSeconds => {
                "#first pass|the median second of the scoring clock at which the first push passed, over the runs that passed"
            }
            Self::FirstPassTokens => {
                "#tokens to first pass|the median tokens through the backend until that push, input, cache and output alike"
            }
            Self::HighScoreSeconds => {
                "#high score|the median second at which the entry of record was pushed, the best entry of the run, over the runs that kept one"
            }
            Self::HighScoreTokens => {
                "#tokens to high score|the median tokens through the backend until that push"
            }
            Self::BudgetUsed => {
                "#budget used|the seconds of its runs over the seconds they were given"
            }
            Self::Waiting => {
                "#waiting|the seconds its runs spent inside requests to the backend over their seconds, above the whole when requests overlapped"
            }
            Self::FirstToken => {
                "#first token|the mean seconds to the first generated token, over the runs reporting one"
            }
            Self::Burn => {
                "#burn|the tokens through the backend a minute, input, cache and output alike"
            }
            Self::Requests => "#requests|the requests to the backend a minute",
            Self::PeakContext => {
                "#peak context|the largest share of the window a run of it reached"
            }
            Self::Compactions => "#compactions|the compactions its harness reported",
            Self::Failed => {
                "#failed|the model calls answered with an error, the ones cut short upstream and the ones abandoned behind the hover"
            }
        }
    }

    /// The cell of the column for `sum`.
    fn cell(self, sum: &Sum) -> String {
        match self {
            Self::Runs => sum.runs.to_string(),
            Self::Passed => sum.passed.to_string(),
            Self::Rounds => views::tally_label(&sum.rounds),
            Self::Score => sum
                .score()
                .map(|score| format!("{score:.2}"))
                .unwrap_or_default(),
            Self::Dollars => {
                if sum.priced > 0 {
                    usage::money(sum.dollars)
                } else {
                    String::new()
                }
            }
            Self::DollarsPerRun => match sum.dollars_per_run() {
                Some(dollars) if sum.unpriced() > 0 => titled(
                    &usage::money(dollars),
                    &format!("{} runs without a price", sum.unpriced()),
                ),
                Some(dollars) => usage::money(dollars),
                None => String::new(),
            },
            Self::DollarsPerPass => sum.dollars_per_pass().map(usage::money).unwrap_or_default(),
            Self::DollarsPerRoundWon => sum
                .dollars_per_round_won()
                .map(usage::money)
                .unwrap_or_default(),
            Self::Output => tokens_label(sum.output_tokens),
            Self::OutputPerRoundWon => sum
                .output_per_round_won()
                .map(|tokens| tokens_label(tokens as u64))
                .unwrap_or_default(),
            Self::Input => tokens_label(sum.input_tokens),
            Self::CacheRead => tokens_label(sum.cache_read_tokens),
            Self::FirstPassSeconds => seconds_label(median(&sum.first_pass_seconds)),
            Self::FirstPassTokens => count_label(median(&sum.first_pass_tokens)),
            Self::HighScoreSeconds => seconds_label(median(&sum.high_score_seconds)),
            Self::HighScoreTokens => count_label(median(&sum.high_score_tokens)),
            Self::BudgetUsed => percent_label(sum.budget_used()),
            Self::Waiting => percent_label(sum.waiting()),
            Self::FirstToken => sum
                .first_token()
                .map(|seconds| format!("{seconds:.1}s"))
                .unwrap_or_default(),
            Self::Burn => sum
                .burn()
                .map(|tokens| tokens_label(tokens as u64))
                .unwrap_or_default(),
            Self::Requests => sum
                .requests_per_minute()
                .map(|requests| format!("{requests:.1}"))
                .unwrap_or_default(),
            Self::PeakContext => percent_label(sum.peak_share),
            Self::Compactions => sum.compactions.to_string(),
            Self::Failed => titled(
                &sum.failed_requests.to_string(),
                &format!(
                    "{} cut short, {} abandoned",
                    sum.truncated_requests, sum.aborted_requests
                ),
            ),
        }
    }
}

/// One run of a seat in a finished round: what it spent and what came of it.
struct Played {
    setup: ava_wire::Setup,
    limit_seconds: u64,
    wall_seconds: u64,
    cost: Option<f64>,
    metrics: Option<ava_wire::Metrics>,
    /// The largest context a request carried, as a share of the window.
    peak_share: Option<f64>,
    compactions: u64,
    passed: bool,
    /// The second of the first push that passed, on the scoring clock, and
    /// the tokens through the backend until then.
    first_pass: Option<u64>,
    first_pass_tokens: Option<u64>,
    /// The second of the entry of record, the best entry of the run, and the
    /// tokens through the backend until then.
    high_score: Option<u64>,
    high_score_tokens: Option<u64>,
    /// The rounds the seat got against other agents in the round, on the run
    /// of the last turn.
    rounds: ava_wire::Tally,
}

/// Every run played in the finished rounds of `record`.
fn played_runs(
    record: &ava_wire::Tournament,
    registry: &registry::Registry,
) -> std::io::Result<Vec<Played>> {
    let game = ava_game::find(&record.game);
    let last_turn = game.map_or(0, |game| game.turns().len() - 1);
    let labels: Vec<String> = record.seats.iter().map(|seat| seat.agent.label()).collect();
    let mut played = Vec::new();

    for round in &record.rounds {
        if round.finished_seconds.is_none() {
            continue;
        }
        let mut tallies = vec![ava_wire::Tally::default(); record.seats.len()];
        for pairing in tournament::pairings(record, round)? {
            if labels.get(pairing.first) == labels.get(pairing.second) {
                continue;
            }
            if let Some(tally) = tallies.get_mut(pairing.first) {
                add_tally(tally, pairing.tally);
            }
            if let Some(tally) = tallies.get_mut(pairing.second) {
                add_tally(tally, views::mirrored(&pairing.tally));
            }
        }

        for entry in &round.entries {
            let Some(setup) = record.seats.get(entry.seat) else {
                continue;
            };
            let directory = std::path::Path::new(docker::RUN_DIRECTORY).join(&entry.run);
            let Ok(run) = runs::read(&directory) else {
                continue;
            };
            let last = entry.turn == last_turn;
            let cost = run
                .metrics
                .as_ref()
                .and_then(|metrics| registry.cost(&run.setup(), metrics));
            let peak_share = run.metrics.as_ref().and_then(|metrics| {
                let window = f64::from(run.context_window.unwrap_or_default());
                (metrics.peak_context_tokens > 0 && window > 0.0)
                    .then(|| metrics.peak_context_tokens as f64 / window)
            });
            let requests = requests(&directory);
            let first_pass = run
                .attempts
                .iter()
                .enumerate()
                .find(|(_, attempt)| attempt.verdict.passed);
            let high_score = entry.attempt.and_then(|seconds| {
                run.attempts
                    .iter()
                    .enumerate()
                    .find(|(_, attempt)| attempt.seconds == seconds)
            });
            played.push(Played {
                setup: setup.clone(),
                limit_seconds: run.limit_seconds,
                wall_seconds: run.wall_seconds().unwrap_or_default(),
                cost,
                peak_share,
                compactions: run.compactions.unwrap_or_default(),
                passed: run.passed(),
                first_pass: first_pass.map(|(_, attempt)| attempt.seconds),
                first_pass_tokens: first_pass.and_then(|(index, attempt)| {
                    tokens_until(&requests, &run, index, attempt.seconds)
                }),
                high_score: entry.attempt,
                high_score_tokens: high_score.and_then(|(index, attempt)| {
                    tokens_until(&requests, &run, index, attempt.seconds)
                }),
                rounds: if last {
                    tallies[entry.seat]
                } else {
                    ava_wire::Tally::default()
                },
                metrics: run.metrics,
            });
        }
    }

    Ok(played)
}

/// One request the proxy of a run logged: the second it was answered, the
/// tokens it moved, and whether it was a push to the git host, which is what
/// puts an attempt on the wall clock.
struct Request {
    seconds: u64,
    tokens: u64,
    push: bool,
}

/// The requests of the run in `directory` from its proxy log, in order,
/// none when the log is not there.
fn requests(directory: &std::path::Path) -> Vec<Request> {
    let Ok(logged) = std::fs::read_to_string(directory.join(docker::ACCESS_LOG)) else {
        return Vec::new();
    };
    let count = |line: &serde_json::Value, field: &str| {
        line.get(field)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default()
    };

    logged
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|line| {
            let seconds = usage::epoch_of(line.get("time")?.as_str()?)?;
            let host = line.get("host")?.as_str()?;
            let push = host == GIT_HOST
                && line.get("method")?.as_str()? == "POST"
                && line.get("uri")?.as_str()?.ends_with(PUSH_PATH);
            let tokens = if host == GIT_HOST || host == SCORE_HOST {
                0
            } else {
                TOKEN_FIELDS.iter().map(|field| count(&line, field)).sum()
            };
            Some(Request {
                seconds,
                tokens,
                push,
            })
        })
        .collect()
}

/// The tokens of `requests` up to the attempt at `index` with `seconds` on
/// the scoring clock: put on the wall clock by the push that made it when
/// every attempt has its push in the log, else by the start of the run.
fn tokens_until(
    requests: &[Request],
    run: &ava_wire::Run,
    index: usize,
    seconds: u64,
) -> Option<u64> {
    let pushes: Vec<u64> = requests
        .iter()
        .filter(|request| request.push)
        .map(|request| request.seconds)
        .collect();
    if requests.is_empty() {
        return None;
    }
    let wall = if pushes.len() == run.attempts.len() {
        pushes[index]
    } else {
        run.started_seconds + seconds
    };

    Some(
        requests
            .iter()
            .filter(|request| !request.push && request.seconds <= wall)
            .map(|request| request.tokens)
            .sum(),
    )
}

fn add_tally(tally: &mut ava_wire::Tally, view: ava_wire::Tally) {
    tally.won += view.won;
    tally.drawn += view.drawn;
    tally.lost += view.lost;
}

/// What a group of runs spent and got, summed.
#[derive(Default)]
struct Sum {
    runs: u64,
    passed: u64,
    rounds: ava_wire::Tally,
    /// The dollars of the runs with a price, and how many had one.
    dollars: f64,
    priced: u64,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_write_tokens: u64,
    requests: u64,
    failed_requests: u64,
    truncated_requests: u64,
    aborted_requests: u64,
    request_seconds: f64,
    /// The mean seconds to the first token summed over the runs reporting
    /// one, and how many did.
    first_token_seconds: f64,
    first_token_runs: u64,
    wall_seconds: u64,
    limit_seconds: u64,
    peak_share: Option<f64>,
    compactions: u64,
    /// The share of its budget every run that passed had spent at its first
    /// pass, the second of that pass and the tokens until it.
    first_pass_shares: Vec<f64>,
    first_pass_seconds: Vec<f64>,
    first_pass_tokens: Vec<f64>,
    /// The second every run that kept an entry pushed its entry of record at,
    /// and the tokens until it.
    high_score_seconds: Vec<f64>,
    high_score_tokens: Vec<f64>,
}

impl Sum {
    fn add(&mut self, played: &Played) {
        self.runs += 1;
        self.passed += u64::from(played.passed);
        add_tally(&mut self.rounds, played.rounds);
        if let Some(cost) = played.cost {
            self.dollars += cost;
            self.priced += 1;
        }
        if let Some(metrics) = &played.metrics {
            self.input_tokens += metrics.input_tokens;
            self.output_tokens += metrics.output_tokens;
            self.cache_read_tokens += metrics.cache_read_tokens;
            self.cache_write_tokens += metrics.cache_write_tokens;
            self.requests += metrics.requests;
            self.failed_requests += metrics.failed_requests;
            self.truncated_requests += metrics.truncated_requests;
            self.aborted_requests += metrics.aborted_requests;
            self.request_seconds += metrics.request_seconds;
            if metrics.mean_first_token_seconds > 0.0 {
                self.first_token_seconds += metrics.mean_first_token_seconds;
                self.first_token_runs += 1;
            }
        }
        self.wall_seconds += played.wall_seconds;
        self.limit_seconds += played.limit_seconds;
        if let Some(share) = played.peak_share {
            self.peak_share = Some(self.peak_share.unwrap_or_default().max(share));
        }
        self.compactions += played.compactions;
        if let Some(seconds) = played.first_pass {
            self.first_pass_shares
                .extend(budget_share(seconds, played.limit_seconds));
            self.first_pass_seconds.push(seconds as f64);
        }
        self.first_pass_tokens
            .extend(played.first_pass_tokens.map(|tokens| tokens as f64));
        self.high_score_seconds
            .extend(played.high_score.map(|seconds| seconds as f64));
        self.high_score_tokens
            .extend(played.high_score_tokens.map(|tokens| tokens as f64));
    }

    fn unpriced(&self) -> u64 {
        self.runs - self.priced
    }

    /// The rounds won, a draw counting half.
    fn rounds_won(&self) -> f64 {
        self.rounds.won as f64 + self.rounds.drawn as f64 / 2.0
    }

    fn score(&self) -> Option<f64> {
        self.rounds.score()
    }

    fn dollars_per_run(&self) -> Option<f64> {
        ratio(self.dollars, self.priced as f64).filter(|_| self.priced > 0)
    }

    fn dollars_per_pass(&self) -> Option<f64> {
        ratio(self.dollars, self.passed as f64).filter(|_| self.priced > 0)
    }

    fn dollars_per_round_won(&self) -> Option<f64> {
        ratio(self.dollars, self.rounds_won()).filter(|_| self.priced > 0)
    }

    fn output_per_round_won(&self) -> Option<f64> {
        ratio(self.output_tokens as f64, self.rounds_won())
    }

    fn budget_used(&self) -> Option<f64> {
        ratio(self.wall_seconds as f64, self.limit_seconds as f64)
    }

    fn waiting(&self) -> Option<f64> {
        ratio(self.request_seconds, self.wall_seconds as f64)
    }

    fn first_token(&self) -> Option<f64> {
        ratio(self.first_token_seconds, self.first_token_runs as f64)
    }

    /// The tokens through the backend over the rounds it won, a draw counting half.
    fn tokens_per_round_won(&self) -> Option<f64> {
        ratio(self.tokens() as f64, self.rounds_won())
    }

    fn tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_read_tokens + self.cache_write_tokens
    }

    fn minutes(&self) -> f64 {
        self.wall_seconds as f64 / SECONDS_PER_MINUTE
    }

    fn burn(&self) -> Option<f64> {
        ratio(self.tokens() as f64, self.minutes())
    }

    fn requests_per_minute(&self) -> Option<f64> {
        ratio(self.requests as f64, self.minutes())
    }

    /// The output tokens one run generated, in thousands.
    fn thousand_output_per_run(&self) -> Option<f64> {
        ratio(self.output_tokens as f64 / THOUSAND, self.runs as f64)
    }
}

/// `numerator` over `denominator`, nothing over nothing.
fn ratio(numerator: f64, denominator: f64) -> Option<f64> {
    (denominator > 0.0).then(|| numerator / denominator)
}

/// The share of a budget of `limit` seconds spent at `seconds`, the whole
/// of it at the latest, nothing without a budget.
fn budget_share(seconds: u64, limit: u64) -> Option<f64> {
    ratio(seconds as f64, limit as f64).map(|share| share.min(1.0))
}

/// The runs sharing one key, summed, with the setup of the first of them.
struct Group {
    setup: ava_wire::Setup,
    sum: Sum,
}

/// `played` grouped by `key`, the best score first and the cheapest run
/// among equals.
fn grouped<'a>(
    played: impl IntoIterator<Item = &'a Played>,
    key: impl Fn(&Played) -> String,
) -> Vec<Group> {
    let mut groups: Vec<(String, Group)> = Vec::new();
    for run in played {
        let key = key(run);
        let group = match groups.iter_mut().find(|(known, _)| *known == key) {
            Some((_, group)) => group,
            None => {
                groups.push((
                    key,
                    Group {
                        setup: run.setup.clone(),
                        sum: Sum::default(),
                    },
                ));
                &mut groups.last_mut().expect("a group was just pushed").1
            }
        };
        group.sum.add(run);
    }

    let mut groups: Vec<Group> = groups.into_iter().map(|(_, group)| group).collect();
    groups.sort_by(|left, right| {
        let score = |group: &Group| group.sum.score().unwrap_or(UNSCORED);
        let price = |group: &Group| group.sum.dollars_per_run().unwrap_or(UNPRICED);
        score(right)
            .total_cmp(&score(left))
            .then(price(left).total_cmp(&price(right)))
    });

    groups
}

fn agent_key(played: &Played) -> String {
    played.setup.agent.label()
}

fn model_key(played: &Played) -> String {
    played.setup.agent.model.clone()
}

/// The report over the tournaments `names`, with a link to itself as a file
/// when `linked`, which the file itself leaves out.
pub(crate) fn page(names: &[String], linked: bool) -> std::io::Result<String> {
    let registry = registry::load()?;
    let mut records = Vec::new();
    for name in names {
        records.push(tournament::load(name)?);
    }
    let mut body = heading(&records, linked);
    if records.is_empty() {
        body.push_str(&note(NO_TOURNAMENTS_NOTE));
        return Ok(document(&body));
    }

    let mut played = Vec::new();
    for record in &records {
        played.extend(played_runs(record, &registry)?);
    }
    if played.is_empty() {
        body.push_str(&note(NO_ROUNDS_NOTE));
        return Ok(document(&body));
    }

    let by_model = grouped(&played, model_key);
    let by_agent = grouped(&played, agent_key);
    body.push_str(&summary(&records, &played));
    body.push_str(&winners(&by_model));
    body.push_str(&pass_curve(&by_model));
    body.push_str(&format!(
        "<div class=\"{}\">{}{}</div>",
        views::CHARTS_GRID_CLASSES,
        scatter(
            &by_model,
            "score against dollars",
            "the share of its rounds won every model got against the dollars one of its runs cost, over every harness that drove it",
            "dollars per run",
            |sum| sum.dollars_per_run(),
            |dollars| format!("{} per run", usage::money(dollars)),
        ),
        scatter(
            &by_model,
            "score against output tokens",
            "the share of its rounds won every model got against the thousands of output tokens one of its runs generated",
            "thousand output tokens per run",
            Sum::thousand_output_per_run,
            |thousands| format!("{thousands:.0}k output tokens per run"),
        ),
    ));
    body.push_str(&section(
        "cost",
        "what every model got for its dollars and tokens, over every harness that drove it and the finished rounds of the chosen tournaments",
        &group_table(&by_model, COST_TABLE, &COST_COLUMNS),
    ));
    body.push_str(&section(
        "time",
        "how every model spent its seconds",
        &group_table(&by_model, TIME_TABLE, &TIME_COLUMNS),
    ));
    body.push_str(&matrix(&by_agent));

    Ok(document(&body))
}

/// The name of the report over `names` as a file.
pub(crate) fn file_name(names: &[String]) -> String {
    let mut name = FILE_PREFIX.to_string();
    for chosen in names {
        name.push_str(NAME_SEPARATOR);
        name.push_str(chosen);
    }
    name.push_str(FILE_SUFFIX);

    name
}

/// The title of the report, the tournaments it spans and when it was
/// rendered, with the link to itself as a file when `linked`.
fn heading(records: &[ava_wire::Tournament], linked: bool) -> String {
    let names: Vec<String> = records.iter().map(|record| record.name.clone()).collect();
    let link = if linked && !names.is_empty() {
        let query: Vec<String> = names
            .iter()
            .map(|name| {
                format!(
                    "{}={}",
                    crate::serve::TOURNAMENT_FIELD,
                    crate::serve::urlencode(name)
                )
            })
            .collect();
        format!(
            "<a class=\"{}\" href=\"/report?{}&{}=on\">{DOWNLOAD_LABEL}</a>",
            views::LINK_CLASSES,
            query.join("&"),
            crate::serve::DOWNLOAD_FIELD
        )
    } else {
        String::new()
    };
    let over = if names.is_empty() {
        String::new()
    } else {
        format!("over {}, ", views::escape(&names.join(", ")))
    };

    format!(
        "<div class=\"{HEADING_ROW_CLASSES}\"><span class=\"{HEADING_CLASSES}\">{TITLE}</span><span class=\"grow\"></span>{link}</div>\
         <p class=\"{SUBTITLE_CLASSES}\">{over}rendered {}</p>",
        usage::utc_date(usage::epoch_now())
    )
}

fn note(text: &str) -> String {
    format!(
        "<p class=\"{} mt-8\">{}</p>",
        views::NOTE_CLASSES,
        views::escape(text)
    )
}

/// The tiles over everything the report spans.
fn summary(records: &[ava_wire::Tournament], played: &[Played]) -> String {
    let mut total = Sum::default();
    for run in played {
        total.add(run);
    }
    let rounds: usize = records
        .iter()
        .map(|record| record.finished_rounds().count())
        .sum();
    let tile = |label: &str, value: &str| views::tile(label, value, "", views::TILE_VALUE_CLASSES);

    views::tiles(&[
        tile("tournaments", &records.len().to_string()),
        tile("finished rounds", &rounds.to_string()),
        tile("runs", &total.runs.to_string()),
        tile("dollars", &usage::money(total.dollars)),
        tile("tokens", &tokens_label(total.tokens())),
        tile("run time", &usage::span(total.wall_seconds)),
    ])
}

/// The tiles naming the model that won on score, on dollars per round won
/// and on tokens per round won, each pooled over every harness that drove it.
fn winners(models: &[Group]) -> String {
    let best = |value: &dyn Fn(&Sum) -> Option<f64>, lowest: bool| {
        let mut ranked: Vec<(f64, &Group)> = models
            .iter()
            .filter(|group| group.sum.rounds_won() > 0.0)
            .filter_map(|group| Some((value(&group.sum)?, group)))
            .collect();
        ranked.sort_by(|left, right| left.0.total_cmp(&right.0));
        let winner = if lowest {
            ranked.first()
        } else {
            ranked.last()
        };
        winner.map(|(value, group)| (*value, views::escape(&group.setup.agent.model)))
    };
    let tile = |label: &str, winner: Option<(f64, String)>, detail: &dyn Fn(f64) -> String| {
        let (value, model) = winner.unwrap_or_default();
        views::tile(
            label,
            &model,
            &if model.is_empty() {
                String::new()
            } else {
                detail(value)
            },
            views::TILE_TEXT_CLASSES,
        )
    };

    views::tiles(&[
        tile("winner by score", best(&Sum::score, false), &|score| {
            format!("score {score:.2}")
        }),
        tile(
            "winner by cost",
            best(&Sum::dollars_per_round_won, true),
            &|dollars| format!("{} per round won", usage::money(dollars)),
        ),
        tile(
            "winner by token efficiency",
            best(&Sum::tokens_per_round_won, true),
            &|tokens| format!("{} tokens per round won", tokens_label(tokens as u64)),
        ),
    ])
}

/// The score of every model under every harness, one row per model and one
/// column per harness, with how far the harnesses spread it and what the
/// model did over all of them, then one row over every model, and how the
/// variance of the agents' scores splits between models and harnesses.
fn matrix(agents: &[Group]) -> String {
    let scored: Vec<&Group> = agents
        .iter()
        .filter(|group| group.sum.score().is_some())
        .collect();
    let mut harnesses: Vec<String> = scored
        .iter()
        .map(|group| group.setup.agent.harness.clone())
        .collect();
    harnesses.sort();
    harnesses.dedup();
    let mut models: Vec<String> = scored
        .iter()
        .map(|group| group.setup.agent.model.clone())
        .collect();
    models.sort();
    models.dedup();

    let mut headers: Vec<String> = vec![MODEL_HEADER.to_string()];
    headers.extend(harnesses.iter().map(|harness| {
        format!(
            "#{harness}|the share of the rounds won by {harness} driving the model, half for a draw"
        )
    }));
    headers.push(SPREAD_HEADER.to_string());
    headers.push(POOLED_HEADER.to_string());
    let headers: Vec<&str> = headers.iter().map(String::as_str).collect();

    // A row is a model over the harnesses, or every model over them: the
    // score of the pooled rounds in every cell, the spread of the cells.
    let row = |name: &str, member: &dyn Fn(&Group) -> bool| {
        let mut cells = vec![model_cell(name)];
        let mut column_scores: Vec<f64> = Vec::new();
        let mut pooled = ava_wire::Tally::default();
        for harness in &harnesses {
            let mut rounds = ava_wire::Tally::default();
            for group in scored
                .iter()
                .filter(|group| member(group) && group.setup.agent.harness == *harness)
            {
                add_tally(&mut rounds, group.sum.rounds);
                add_tally(&mut pooled, group.sum.rounds);
            }
            let score = rounds.score();
            column_scores.extend(score);
            cells.push(score.map(|score| format!("{score:.2}")).unwrap_or_default());
        }
        let spread = if column_scores.len() > 1 {
            let low = column_scores.iter().copied().fold(f64::INFINITY, f64::min);
            let high = column_scores
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            format!("{:.2}", high - low)
        } else {
            String::new()
        };
        cells.push(spread);
        cells.push(
            pooled
                .score()
                .map(|score| format!("{score:.2}"))
                .unwrap_or_default(),
        );
        (pooled.score().unwrap_or_default(), cells)
    };
    let mut rows: Vec<(f64, Vec<String>)> = models
        .iter()
        .map(|model| row(model, &|group: &Group| group.setup.agent.model == *model))
        .collect();
    rows.sort_by(|left, right| right.0.total_cmp(&left.0));
    rows.push(row(EVERY_MODEL, &|_| true));

    let model = |group: &Group| group.setup.agent.model.clone();
    let harness = |group: &Group| group.setup.agent.harness.clone();
    let (by_model, by_harness, rest) = variance_shares(&scored, &model, &harness);

    format!(
        "{}<p class=\"{} mt-3\">{}</p>",
        section(
            "models and harnesses",
            "the score of every model under every harness that drove it, over the finished rounds of the chosen tournaments",
            &views::sorted_table(
                MATRIX_TABLE,
                Some(headers.len() - 1),
                &headers,
                rows.into_iter().map(|(_, cells)| cells).collect(),
                Some(NO_MATRIX_NOTE),
            )
        ),
        views::NOTE_CLASSES,
        views::explained(
            &format!(
                "variance of the agents' scores: models {}, harnesses {}, the rest {}",
                percent_label(by_model),
                percent_label(by_harness),
                percent_label(rest)
            ),
            "how far the scores of the agents, every harness on every model, spread around their mean, split into the part between the means of the models, the part between the means of the harnesses, and what neither explains"
        )
    )
}

/// How the variance of the scores of `scored` splits: the share between the
/// means of `factor`, the share between the means of `other`, and the rest,
/// none of them without two agents to spread.
fn variance_shares(
    scored: &[&Group],
    factor: &dyn Fn(&Group) -> String,
    other: &dyn Fn(&Group) -> String,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let scores: Vec<f64> = scored
        .iter()
        .filter_map(|group| group.sum.score())
        .collect();
    if scores.len() < 2 {
        return (None, None, None);
    }
    let mean = scores.iter().sum::<f64>() / scores.len() as f64;
    let total: f64 = scores.iter().map(|score| (score - mean).powi(2)).sum();
    if total <= 0.0 {
        return (None, None, None);
    }
    let between = |key: &dyn Fn(&Group) -> String| -> f64 {
        scored
            .iter()
            .map(|group| {
                let peers: Vec<f64> = scored
                    .iter()
                    .filter(|peer| key(peer) == key(group))
                    .filter_map(|peer| peer.sum.score())
                    .collect();
                let peer_mean = peers.iter().sum::<f64>() / peers.len() as f64;
                (peer_mean - mean).powi(2)
            })
            .sum::<f64>()
            / total
    };
    let by_factor = between(factor);
    let by_other = between(other);

    (
        Some(by_factor),
        Some(by_other),
        Some((1.0 - by_factor - by_other).max(0.0)),
    )
}

/// A titled table.
fn section(title: &str, tooltip: &str, table: &str) -> String {
    format!(
        "<p class=\"{}\">{}</p>{table}",
        views::TITLE_CLASSES,
        views::explained(title, tooltip)
    )
}

/// The table `name` of `groups` over `columns`, one row per model, its
/// headers sorting it, arriving sorted by the score when it has one.
fn group_table(groups: &[Group], name: &str, columns: &[Column]) -> String {
    let mut headers = vec![MODEL_HEADER];
    headers.extend(columns.iter().map(|column| column.header()));
    let score = columns
        .iter()
        .position(|column| matches!(column, Column::Score))
        .map(|column| column + MODEL_CELLS);

    let rows = groups
        .iter()
        .map(|group| {
            let mut row = vec![model_cell(&group.setup.agent.model)];
            row.extend(columns.iter().map(|column| column.cell(&group.sum)));
            row
        })
        .collect();

    views::sorted_table(name, score, &headers, rows, Some(NO_MODELS_NOTE))
}

/// The name of a model, as a cell.
fn model_cell(model: &str) -> String {
    format!(
        "<span class=\"{}\">{}</span>",
        views::MONO_CLASSES,
        views::escape(model)
    )
}

/// The line of a model on a chart, without its points yet, in a colour
/// hashed from its name.
fn group_series(group: &Group) -> chart::Series {
    let model = &group.setup.agent.model;

    chart::Series {
        label: model.clone(),
        hover: model.clone(),
        hue: views::fnv1a(model.as_bytes()) % HUES,
        face: String::new(),
        points: Vec::new(),
    }
}

/// The share of every model's runs that had passed by every share of the
/// budget, one stepped line per model from nothing to the end of the budget.
fn pass_curve(groups: &[Group]) -> String {
    let series: Vec<chart::Series> = groups
        .iter()
        .map(|group| {
            let mut series = group_series(group);
            let mut shares = group.sum.first_pass_shares.clone();
            shares.sort_by(f64::total_cmp);
            series.points.push(chart::Point {
                x: 0.0,
                y: 0.0,
                hover: String::new(),
            });
            for (count, share) in shares.iter().enumerate() {
                let passed = count + 1;
                series.points.push(chart::Point {
                    x: share * PERCENT,
                    y: passed as f64 / group.sum.runs as f64 * PERCENT,
                    hover: format!(
                        "{}, {passed} of {} runs passed by {:.0}% of the budget",
                        series.label,
                        group.sum.runs,
                        share * PERCENT
                    ),
                });
            }
            let reached = series.points.last().map_or(0.0, |point| point.y);
            series.points.push(chart::Point {
                x: PERCENT,
                y: reached,
                hover: String::new(),
            });
            series
        })
        .collect();

    views::chart_panel(
        "passes over the budget",
        "the share of every model's runs that had a passing push by every share of the budget, over every harness that drove it and the finished rounds of the chosen tournaments, a curve climbing early for a model that passes fast",
        &chart::lines(
            &series,
            &chart::Axis::percent().titled("share of the budget spent"),
            &chart::Axis::percent().titled("share of the runs passed"),
            chart::Shape::Stepped,
            chart::WIDE_WIDTH,
            NO_MODELS_NOTE,
        ),
    )
}

/// The score of every group against `value`, one mark per group, for the
/// groups with a score and a value.
fn scatter(
    groups: &[Group],
    title: &str,
    tooltip: &str,
    measure: &str,
    value: impl Fn(&Sum) -> Option<f64>,
    detail: impl Fn(f64) -> String,
) -> String {
    let mut top = 0.0f64;
    let series: Vec<chart::Series> = groups
        .iter()
        .filter_map(|group| {
            let score = group.sum.score()?;
            let x = value(&group.sum)?;
            top = top.max(x);
            let mut series = group_series(group);
            series.points.push(chart::Point {
                x,
                y: score * PERCENT,
                hover: format!("{}, score {score:.2}, {}", series.label, detail(x)),
            });
            Some(series)
        })
        .collect();

    views::chart_panel(
        title,
        tooltip,
        &chart::lines(
            &series,
            &chart::Axis::values(top).titled(measure),
            &chart::Axis::percent().titled("share of the rounds won"),
            chart::Shape::Scatter,
            chart::NARROW_WIDTH,
            NO_MODELS_NOTE,
        ),
    )
}

/// The whole document around `body`: the head of the layout with the styles
/// and the fonts inside it in place of their addresses, and the script
/// sorting the tables, so nothing is fetched from the server.
fn document(body: &str) -> String {
    let (head, _) = views::LAYOUT_TEMPLATE
        .split_once(HEAD_END)
        .expect("the layout has a head");
    let mut head = head.replace(views::TITLE_PLACEHOLDER, TITLE);
    for (name, bytes) in crate::serve::FONTS {
        head = head.replace(
            &format!("{FONT_ADDRESS_PREFIX}{name}"),
            &format!("{FONT_DATA_PREFIX}{}", base64(bytes)),
        );
    }
    let head = head.replace(
        TAILWIND_TAG,
        &format!("<script>{}</script>", crate::serve::TAILWIND),
    );

    format!(
        "{head}{HEAD_END}<body class=\"{BODY_CLASSES}\"><main class=\"{MAIN_CLASSES}\">{body}</main><script>{}</script></body></html>",
        crate::serve::TABLE_SORT
    )
}

/// `label` with `title` behind its hover.
fn titled(label: &str, title: &str) -> String {
    format!("<span title=\"{}\">{label}</span>", views::escape(title))
}

/// A count of seconds as a span, nothing for none.
fn seconds_label(seconds: Option<f64>) -> String {
    seconds
        .map(|seconds| usage::span(seconds as u64))
        .unwrap_or_default()
}

/// A count in thousands or millions, nothing for none.
fn count_label(count: Option<f64>) -> String {
    count
        .map(|count| tokens_label(count as u64))
        .unwrap_or_default()
}

/// A share as whole percent, nothing for none.
fn percent_label(share: Option<f64>) -> String {
    share
        .map(|share| format!("{:.0}%", share * PERCENT))
        .unwrap_or_default()
}

/// A count of tokens in thousands or millions, the count itself behind the hover.
fn tokens_label(count: u64) -> String {
    let compact = match count as f64 {
        tokens if tokens >= MILLION => format!("{:.1}M", tokens / MILLION),
        tokens if tokens >= THOUSAND => format!("{:.0}k", tokens / THOUSAND),
        _ => return count.to_string(),
    };

    titled(&compact, &count.to_string())
}

/// The middle of `values`, the mean of the two middle ones of an even count.
fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        return Some(sorted[middle]);
    }

    Some((sorted[middle - 1] + sorted[middle]) / 2.0)
}

/// `bytes` in base64, for a font inside a stylesheet.
fn base64(bytes: &[u8]) -> String {
    let mut encoded =
        String::with_capacity(bytes.len().div_ceil(BASE64_BLOCK_BYTES) * BASE64_BLOCK_CHARACTERS);
    for chunk in bytes.chunks(BASE64_BLOCK_BYTES) {
        let mut block = [0u8; BASE64_BLOCK_BYTES];
        block[..chunk.len()].copy_from_slice(chunk);
        let bits = u32::from_be_bytes([0, block[0], block[1], block[2]]);
        for index in 0..BASE64_BLOCK_CHARACTERS {
            if index <= chunk.len() {
                let shift = BASE64_BITS * (BASE64_BLOCK_CHARACTERS - 1 - index) as u32;
                encoded.push(BASE64_ALPHABET[(bits >> shift) as usize & 0x3f] as char);
            } else {
                encoded.push(BASE64_PAD);
            }
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use super::{Played, Sum};

    fn played(passed: Option<u64>, cost: Option<f64>, won: u64, lost: u64) -> Played {
        Played {
            setup: ava_wire::Setup {
                agent: ava_wire::Agent {
                    harness: "pi".to_string(),
                    model: "m".to_string(),
                },
                thinking: Some("high".to_string()),
                backend: None,
                name: None,
            },
            limit_seconds: 1000,
            wall_seconds: 900,
            cost,
            metrics: Some(ava_wire::Metrics {
                output_tokens: 3000,
                request_seconds: 450.0,
                ..ava_wire::Metrics::default()
            }),
            peak_share: Some(0.4),
            compactions: 0,
            passed: passed.is_some(),
            first_pass: passed,
            first_pass_tokens: passed.map(|_| 1000),
            high_score: passed,
            high_score_tokens: passed.map(|_| 1000),
            rounds: ava_wire::Tally {
                won,
                drawn: 0,
                lost,
            },
        }
    }

    #[test]
    fn a_sum_derives_its_measures_from_the_runs() {
        let mut sum = Sum::default();
        sum.add(&played(Some(250), Some(2.0), 3, 1));
        sum.add(&played(None, Some(4.0), 0, 4));
        sum.add(&played(Some(750), None, 1, 3));

        assert_eq!(sum.runs, 3);
        assert_eq!(sum.passed, 2);
        assert_eq!(sum.unpriced(), 1);
        assert_eq!(sum.score(), Some(4.0 / 12.0));
        assert_eq!(sum.dollars_per_run(), Some(3.0));
        assert_eq!(sum.dollars_per_pass(), Some(3.0));
        assert_eq!(sum.dollars_per_round_won(), Some(1.5));
        assert_eq!(sum.output_per_round_won(), Some(2250.0));
        assert_eq!(super::median(&sum.first_pass_shares), Some(0.5));
        assert_eq!(super::median(&sum.first_pass_seconds), Some(500.0));
        assert_eq!(super::median(&sum.high_score_tokens), Some(1000.0));
        assert_eq!(sum.tokens_per_round_won(), Some(2250.0));
        assert_eq!(sum.waiting(), Some(0.5));
        assert_eq!(sum.budget_used(), Some(0.9));
        assert_eq!(sum.peak_share, Some(0.4));
    }

    #[test]
    fn groups_sort_by_score_then_by_price() {
        let runs = vec![
            played(Some(1), Some(5.0), 1, 1),
            played(Some(1), Some(1.0), 2, 0),
        ];
        let mut cheaper = runs;
        cheaper[1].setup.agent.harness = "codex".to_string();
        let groups = super::grouped(&cheaper, super::agent_key);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].setup.agent.harness, "codex");
    }

    #[test]
    fn the_labels_read_as_the_columns_say() {
        assert_eq!(super::tokens_label(999), "999");
        assert!(super::tokens_label(262_798).contains(">263k<"));
        assert!(super::tokens_label(27_449_604).contains(">27.4M<"));
        assert_eq!(super::percent_label(Some(0.4251)), "43%");
        assert_eq!(super::percent_label(None), "");
        assert_eq!(super::median(&[3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(super::median(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
        assert_eq!(super::median(&[]), None);
        assert_eq!(
            super::file_name(&["a".to_string(), "b".to_string()]),
            "report-a-b.html"
        );
    }

    #[test]
    fn base64_pads_the_last_block() {
        assert_eq!(super::base64(b""), "");
        assert_eq!(super::base64(b"f"), "Zg==");
        assert_eq!(super::base64(b"fo"), "Zm8=");
        assert_eq!(super::base64(b"foo"), "Zm9v");
        assert_eq!(super::base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn the_document_fetches_nothing_from_the_server() {
        let document = super::document("<p>body</p>");

        assert!(document.starts_with("<!doctype html>"));
        assert!(document.contains("data:font/woff2;base64,"));
        assert!(!document.contains("/assets/"));
        assert!(document.contains("<title>report"));
        assert!(document.contains("table[data-sortable]"));
        assert!(document.ends_with("</html>"));
    }
}
