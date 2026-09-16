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
const DETAIL_CLASSES: &str = "text-xs text-neutral-500 mb-3";
const LEVEL_CLASSES: &str = "font-mono text-neutral-300";
const FILE_PREFIX: &str = "report";
const FILE_SUFFIX: &str = ".html";
const NAME_SEPARATOR: &str = "-";
const DOWNLOAD_LABEL: &str = "download";
const NO_TOURNAMENTS_NOTE: &str = "no tournament chosen, check some on the tournaments page";
const NO_ROUNDS_NOTE: &str = "no finished round in the chosen tournaments";
const NO_AGENTS_NOTE: &str = "no run in a finished round";
const UNSET_LEVEL: &str = "-";
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

const AGENT_HEADER: &str = "*agent";
const LEVEL_HEADER: &str = "level|the thinking level the agent was seated at";
const POINTS_HEADER: &str =
    "#points|the points of the entries of record it kept, summed over the finished rounds";
const POINTS_PER_DOLLAR_HEADER: &str = "#points per $|those points over the dollars of its runs";

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
const TIME_COLUMNS: [Column; 10] = [
    Column::FirstPass,
    Column::Banked,
    Column::BudgetUsed,
    Column::Waiting,
    Column::FirstToken,
    Column::Burn,
    Column::Requests,
    Column::PeakContext,
    Column::Compactions,
    Column::Failed,
];
/// The columns of the table over every thinking level an agent played at,
/// and of the table of every tournament.
const BRIEF_COLUMNS: [Column; 11] = [
    Column::Runs,
    Column::Passed,
    Column::Rounds,
    Column::Score,
    Column::DollarsPerRun,
    Column::DollarsPerRoundWon,
    Column::OutputPerRoundWon,
    Column::FirstPass,
    Column::Banked,
    Column::Waiting,
    Column::PeakContext,
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
    FirstPass,
    Banked,
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
            Self::FirstPass => {
                "#first pass|the median share of the budget a run had spent when its first push passed, over the runs that passed"
            }
            Self::Banked => {
                "#banked|the median share of the budget a run had spent when it pushed its entry of record, over the runs that kept one"
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
            Self::FirstPass => percent_label(median(&sum.first_pass_shares)),
            Self::Banked => percent_label(median(&sum.banked_shares)),
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
    /// The tournament, by its place among the chosen ones.
    tournament: usize,
    setup: ava_wire::Setup,
    limit_seconds: u64,
    wall_seconds: u64,
    cost: Option<f64>,
    metrics: Option<ava_wire::Metrics>,
    /// The largest context a request carried, as a share of the window.
    peak_share: Option<f64>,
    compactions: u64,
    passed: bool,
    /// The second of the first push that passed, on the scoring clock.
    first_pass: Option<u64>,
    /// The second of the entry of record, on the scoring clock.
    banked: Option<u64>,
    /// The rounds the seat got against other agents in the round, on the run
    /// of the last turn.
    rounds: ava_wire::Tally,
    /// The points of the entry of record, on the run of the last turn of a
    /// game that ranks.
    points: Option<u64>,
}

/// Every run played in the finished rounds of `record`.
fn played_runs(
    tournament: usize,
    record: &ava_wire::Tournament,
    registry: &registry::Registry,
) -> std::io::Result<Vec<Played>> {
    let game = ava_game::find(&record.game);
    let last_turn = game.map_or(0, |game| game.turns().len() - 1);
    let kept = game
        .map(|game| views::kept_entries(record, game))
        .transpose()?
        .unwrap_or_default();
    let labels: Vec<String> = record.seats.iter().map(|seat| seat.agent.label()).collect();
    let mut played = Vec::new();

    for (index, round) in record.rounds.iter().enumerate() {
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
            played.push(Played {
                tournament,
                setup: setup.clone(),
                limit_seconds: run.limit_seconds,
                wall_seconds: run.wall_seconds().unwrap_or_default(),
                cost,
                peak_share,
                compactions: run.compactions.unwrap_or_default(),
                passed: run.passed(),
                first_pass: run
                    .attempts
                    .iter()
                    .find(|attempt| attempt.verdict.passed)
                    .map(|attempt| attempt.seconds),
                banked: entry.attempt,
                rounds: if last {
                    tallies[entry.seat]
                } else {
                    ava_wire::Tally::default()
                },
                points: kept
                    .iter()
                    .filter(|_| last)
                    .find(|kept| kept.round == index && kept.seat == entry.seat)
                    .and_then(views::KeptEntries::points),
                metrics: run.metrics,
            });
        }
    }

    Ok(played)
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
    /// The share of its budget every run that passed had spent at its first pass.
    first_pass_shares: Vec<f64>,
    /// The share of its budget every run that kept an entry had spent at it.
    banked_shares: Vec<f64>,
    points: u64,
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
        }
        if let Some(seconds) = played.banked {
            self.banked_shares
                .extend(budget_share(seconds, played.limit_seconds));
        }
        self.points += played.points.unwrap_or_default();
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

    fn points_per_dollar(&self) -> Option<f64> {
        ratio(self.points as f64, self.dollars)
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

/// A column of one table alone: its header and the cell of a sum.
type Extra<'a> = (&'a str, &'a dyn Fn(&Sum) -> String);

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

/// The agent with its thinking level, the key of the levels table.
fn level_key(played: &Played) -> String {
    format!(
        "{}\0{}",
        played.setup.agent.label(),
        level_of(&played.setup)
    )
}

fn level_of(setup: &ava_wire::Setup) -> &str {
    setup.thinking.as_deref().unwrap_or(UNSET_LEVEL)
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
    for (index, record) in records.iter().enumerate() {
        played.extend(played_runs(index, record, &registry)?);
    }
    if played.is_empty() {
        body.push_str(&note(NO_ROUNDS_NOTE));
        return Ok(document(&body));
    }

    let by_agent = grouped(&played, agent_key);
    // The levels of one agent stand together, in the order of the agents.
    let mut by_level = grouped(&played, level_key);
    by_level.sort_by_key(|group| {
        by_agent
            .iter()
            .position(|agent| agent.setup.agent == group.setup.agent)
    });
    body.push_str(&summary(&records, &played));
    body.push_str(&pass_curve(&registry, &by_agent));
    body.push_str(&format!(
        "<div class=\"{}\">{}{}</div>",
        views::CHARTS_GRID_CLASSES,
        scatter(
            &registry,
            &by_level,
            "score against dollars",
            "every agent at every thinking level it played: the share of its rounds won against the dollars one of its runs cost",
            |sum| sum.dollars_per_run(),
            |dollars| format!("{} per run", usage::money(dollars)),
        ),
        scatter(
            &registry,
            &by_level,
            "score against output tokens",
            "every agent at every thinking level it played: the share of its rounds won against the thousands of output tokens one of its runs generated",
            Sum::thousand_output_per_run,
            |thousands| format!("{thousands:.0}k output tokens per run"),
        ),
    ));
    body.push_str(&section(
        "cost",
        "what every agent got for its dollars and tokens, over the finished rounds of the chosen tournaments",
        &group_table(&registry, &by_agent, &COST_COLUMNS, false, &[]),
    ));
    body.push_str(&section(
        "time",
        "how every agent spent its seconds",
        &group_table(&registry, &by_agent, &TIME_COLUMNS, false, &[]),
    ));
    body.push_str(&section(
        "thinking levels",
        "every agent at every thinking level it was seated at",
        &group_table(&registry, &by_level, &BRIEF_COLUMNS, true, &[]),
    ));
    for (index, record) in records.iter().enumerate() {
        body.push_str(&tournament_section(&registry, index, record, &played)?);
    }

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

/// A titled table.
fn section(title: &str, tooltip: &str, table: &str) -> String {
    format!(
        "<p class=\"{}\">{}</p>{table}",
        views::TITLE_CLASSES,
        views::explained(title, tooltip)
    )
}

/// The table of `groups` over `columns`, one row per group, the thinking
/// level of each beside its name when `levelled`, and `extra` columns after
/// them, each a header with the cell of every group.
fn group_table(
    registry: &registry::Registry,
    groups: &[Group],
    columns: &[Column],
    levelled: bool,
    extra: &[Extra],
) -> String {
    let mut headers = vec!["", AGENT_HEADER];
    if levelled {
        headers.push(LEVEL_HEADER);
    }
    headers.extend(columns.iter().map(|column| column.header()));
    headers.extend(extra.iter().map(|(header, _)| *header));

    let rows = groups
        .iter()
        .map(|group| {
            let mut row = agent_cells(registry, &group.setup.agent).to_vec();
            if levelled {
                row.push(format!(
                    "<span class=\"{LEVEL_CLASSES}\">{}</span>",
                    views::escape(level_of(&group.setup))
                ));
            }
            row.extend(columns.iter().map(|column| column.cell(&group.sum)));
            row.extend(extra.iter().map(|(_, cell)| cell(&group.sum)));
            row
        })
        .collect();

    views::table(&headers, rows, Some(NO_AGENTS_NOTE))
}

/// The avatar of `agent` and the name it goes by, the harness on the model
/// under a name of the registry.
fn agent_cells(registry: &registry::Registry, agent: &ava_wire::Agent) -> [String; 2] {
    let name = views::agent_name(registry, agent);
    let detail = if name == agent.label() {
        String::new()
    } else {
        format!(
            "<div class=\"text-xs {} mt-0.5\">{}</div>",
            views::MUTED_CLASSES,
            views::escape(&agent.label())
        )
    };

    [
        views::avatar(agent, views::AVATAR_CLASSES),
        format!(
            "<span class=\"{}\">{}</span>{detail}",
            views::MONO_CLASSES,
            views::escape(&name)
        ),
    ]
}

/// The line of a group on a chart, without its points yet: named by the
/// name the agent goes by, at its thinking level when `levelled`, in the
/// colour of its avatar.
fn group_series(
    registry: &registry::Registry,
    setup: &ava_wire::Setup,
    levelled: bool,
) -> chart::Series {
    let name = views::agent_name(registry, &setup.agent);
    let label = if levelled {
        format!("{name} at {}", level_of(setup))
    } else {
        name
    };
    let (hue, _) = views::avatar_grid(&setup.agent);

    chart::Series {
        label,
        hover: setup.label(),
        hue,
        face: views::avatar(&setup.agent, views::CHART_AVATAR_CLASSES),
        points: Vec::new(),
    }
}

/// The share of every agent's runs that had passed by every share of the
/// budget, one stepped line per agent from nothing to the end of the budget.
fn pass_curve(registry: &registry::Registry, groups: &[Group]) -> String {
    let series: Vec<chart::Series> = groups
        .iter()
        .map(|group| {
            let mut series = group_series(registry, &group.setup, false);
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
        "the share of every agent's runs that had a passing push by every share of the budget, over the finished rounds of the chosen tournaments, a curve climbing early for an agent that passes fast",
        &chart::lines(
            &series,
            &chart::Axis::percent(),
            &chart::Axis::percent(),
            chart::Shape::Stepped,
            chart::WIDE_WIDTH,
            NO_AGENTS_NOTE,
        ),
    )
}

/// The score of every group against `value`, one mark per group, for the
/// groups with a score and a value.
fn scatter(
    registry: &registry::Registry,
    groups: &[Group],
    title: &str,
    tooltip: &str,
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
            let mut series = group_series(registry, &group.setup, true);
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
            &chart::Axis::values(top),
            &chart::Axis::percent(),
            chart::Shape::Scatter,
            chart::NARROW_WIDTH,
            NO_AGENTS_NOTE,
        ),
    )
}

/// One tournament: what it fixed, its agents over the brief columns, the
/// points they banked for a game that ranks, and its score chart.
fn tournament_section(
    registry: &registry::Registry,
    index: usize,
    record: &ava_wire::Tournament,
    played: &[Played],
) -> std::io::Result<String> {
    let own: Vec<&Played> = played
        .iter()
        .filter(|run| run.tournament == index)
        .collect();
    let ranked = own.iter().any(|run| run.points.is_some());
    let groups = grouped(own.iter().copied(), agent_key);
    let points_cell = |sum: &Sum| sum.points.to_string();
    let points_per_dollar_cell = |sum: &Sum| {
        sum.points_per_dollar()
            .map(|points| format!("{points:.0}"))
            .unwrap_or_default()
    };
    let extra: Vec<Extra> = if ranked {
        vec![
            (POINTS_HEADER, &points_cell),
            (POINTS_PER_DOLLAR_HEADER, &points_per_dollar_cell),
        ]
    } else {
        Vec::new()
    };

    let labels: Vec<String> = record.seats.iter().map(|seat| seat.agent.label()).collect();
    let rounds_labeled = views::rounds_labeled(record, &labels)?;
    let finished = record.finished_rounds().count();
    let detail = format!(
        "{}, {} seconds a run, {} seats, {finished} of {} rounds finished",
        views::escape(&record.game),
        record.limit_seconds,
        record.seats.len(),
        record.rounds.len()
    );

    Ok(format!(
        "<p class=\"{}\"><span class=\"{}\">{}</span></p><p class=\"{DETAIL_CLASSES}\">{detail}</p>{}{}",
        views::TITLE_CLASSES,
        views::MONO_CLASSES,
        views::escape(&record.name),
        group_table(registry, &groups, &BRIEF_COLUMNS, false, &extra),
        views::score_chart(
            record,
            registry,
            &labels,
            &rounds_labeled,
            chart::WIDE_WIDTH
        ),
    ))
}

/// The whole document around `body`: the head of the layout with the styles
/// and the fonts inside it in place of their addresses, so nothing is
/// fetched from the server.
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
        "{head}{HEAD_END}<body class=\"{BODY_CLASSES}\"><main class=\"{MAIN_CLASSES}\">{body}</main></body></html>"
    )
}

/// `label` with `title` behind its hover.
fn titled(label: &str, title: &str) -> String {
    format!("<span title=\"{}\">{label}</span>", views::escape(title))
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
            tournament: 0,
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
            banked: passed,
            rounds: ava_wire::Tally {
                won,
                drawn: 0,
                lost,
            },
            points: None,
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
        assert!(document.ends_with("</html>"));
    }
}
