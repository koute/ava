//! Line charts rendered as SVG markup into the page, the way the rest of the
//! interface is drawn: no script and no library, the hover of every point a
//! native `<title>`, the colours and fonts the classes of the layout.

/// The width of a drawing sharing its row with another, about the pixels
/// half a column has on a laptop screen, so every size in this module reads
/// as pixels there and grows with the screen.
pub(crate) const NARROW_WIDTH: f64 = 600.0;
/// The width of a drawing on a row of its own, twice the narrow one, so the
/// two scale onto the page alike and their text comes out one size.
pub(crate) const WIDE_WIDTH: f64 = 2.0 * NARROW_WIDTH;
/// The height of the drawing.
const HEIGHT: f64 = 240.0;
/// The room the labels and the title of the vertical axis take on the left.
const LEFT: f64 = 64.0;
/// The room the labels of the horizontal axis take at the bottom, and what
/// every further line of a label, an axis title and a row of icons add to it.
const LABEL_ROOM: f64 = 28.0;
const LINE_ROOM: f64 = 12.0;
const AXIS_TITLE_ROOM: f64 = 16.0;
/// The width of a character of a tick label, for telling when neighbours would collide.
const CHARACTER_WIDTH: f64 = 6.2;
/// A tick label too wide for its slot breaks after these.
const LABEL_BREAKS: [char; 2] = ['-', ' '];
/// The room above the plot, so the topmost point is not cut.
const TOP: f64 = 10.0;
/// The room right of the plot, so the last point is not cut.
const RIGHT: f64 = 14.0;
/// The room between a tick label and its axis.
const LABEL_GAP: f64 = 8.0;
/// How far below the horizontal axis its labels sit, to their baseline.
const AXIS_LABEL_DROP: f64 = 20.0;
/// The lift of a vertical axis label to sit centred on its tick.
const LABEL_LIFT: f64 = 4.0;
/// The side of the icon in front of a horizontal tick label and the gap to the label.
const ICON_SIDE: f64 = 18.0;
const ICON_GAP: f64 = 6.0;
/// Icons stay in the background of the labels they belong to.
const ICON_OPACITY: &str = "0.6";
/// The baseline of an axis title, in from the edge of the drawing.
const AXIS_TITLE_INSET: f64 = 6.0;
const FONT_SIZE: u32 = 11;
const LINE_WIDTH: f64 = 1.25;
/// The width of the line under the cursor.
const HOVERED_LINE_WIDTH: f64 = 2.5;
/// The width of the invisible stroke the cursor finds a line by, since a
/// line two pixels wide is hard to rest on.
const HIT_WIDTH: f64 = 14.0;
const POINT_RADIUS: f64 = 2.75;
/// The radius of a mark standing alone, with no line to find it by.
const MARK_RADIUS: f64 = 5.0;
/// The share of a slot the bars of its group fill, the rest is the gap to the next slot.
const BAR_GROUP_SHARE: f64 = 0.6;
/// The share of its lane a bar leaves open on either side, so neighbours do not touch.
const BAR_LANE_GAP: f64 = 0.15;
/// A bar is a tinted area with its colour on the edge, so a chart of many stays quiet.
const BAR_FILL_OPACITY: &str = "0.35";
const BAR_EDGE_WIDTH: f64 = 1.0;
/// The ticks of a percent axis, every quarter of it.
const PERCENT_STEP: f64 = 25.0;
const PERCENT_MAX: f64 = 100.0;
/// The ticks a value axis aims for, the nice step rounding it up or down.
const VALUE_TICKS: f64 = 4.0;
/// The most labels a horizontal axis carries before they are thinned.
const HORIZONTAL_LABELS: usize = 12;
/// The saturation and lightness of the colour of a series, the same as an
/// avatar's, so a line and the avatar it belongs to are one colour.
const SERIES_SATURATION: &str = "60%";
const SERIES_LIGHTNESS: &str = "55%";
const LINE_CLASSES: &str = "font-sans";
/// The wrapper the hover rules of a chart are scoped to, so a line lights the
/// name of its own chart and no other.
const CHART_CLASS: &str = "line-chart";
/// The class every series carries, and the prefix of the one naming it.
const SERIES_CLASS: &str = "series";
/// The class the visible line of a series carries.
const STROKE_CLASS: &str = "line";
/// The class every legend entry carries, and the prefix of the one naming it.
const LEGEND_CLASS: &str = "legend";
/// How faint the other lines go while one is under the cursor.
const DIMMED_OPACITY: &str = "0.3";
/// The ground of the legend entry of the line under the cursor.
const LIT_LEGEND_GROUND: &str = "#262626";
const HOVER_TRANSITION: &str = "0.12s";
const GRID_CLASSES: &str = "stroke-neutral-800";
const AXIS_CLASSES: &str = "stroke-neutral-700";
const LABEL_CLASSES: &str = "fill-neutral-500 font-mono";
const AXIS_TITLE_CLASSES: &str = "fill-neutral-400 font-sans";
/// The lines splitting a plot into its quadrants, and the labels in their corners.
const QUADRANT_LINE_CLASSES: &str = "stroke-neutral-700";
const QUADRANT_DASHES: &str = "4 4";
const QUADRANT_LABEL_CLASSES: &str = "fill-neutral-500 font-sans";
/// The tints of the quadrant to be in, of the one to stay out of, and of the
/// two mixed ones between them.
const GOOD_QUADRANT_CLASSES: &str = "fill-emerald-500/10";
const BAD_QUADRANT_CLASSES: &str = "fill-red-500/10";
const MIXED_QUADRANT_CLASSES: &str = "fill-amber-500/5";
/// How far a quadrant label sits in from the corner of its quadrant.
const QUADRANT_LABEL_INSET: f64 = 8.0;
const QUADRANT_LABEL_DROP: f64 = 14.0;
const POINT_STROKE_CLASSES: &str = "stroke-neutral-900";
const LEGEND_CLASSES: &str = "flex flex-wrap gap-x-4 gap-y-1.5 mt-3 text-xs text-neutral-300";
const LEGEND_ITEM_CLASSES: &str =
    "flex items-center gap-1.5 whitespace-nowrap rounded px-1.5 py-0.5 -mx-1.5 cursor-default";
const SWATCH_CLASSES: &str = "inline-block h-0.5 w-4 rounded-full";
const EMPTY_CLASSES: &str = "text-neutral-500 text-xs";

/// How the points of a series are drawn.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shape {
    /// A line from point to point.
    Straight,
    /// A line holding its value until the next point, so a value stands
    /// until the next one is banked.
    Stepped,
    /// Marks alone, one per point, with no line between them.
    Scatter,
    /// Bars, one per point, the points of every series at a slot standing
    /// side by side in it.
    Bars,
}

/// The four quadrants of a plot split at the middle of both axes, each with
/// its label: top left, top right, bottom left, bottom right. The top left is
/// the one to be in and the bottom right the one to stay out of.
pub(crate) struct Quadrants {
    pub(crate) labels: [String; 4],
}

/// One line of a chart.
pub(crate) struct Series {
    /// The name in the legend.
    pub(crate) label: String,
    /// The text of the hover over the line, saying whose it is.
    pub(crate) hover: String,
    /// The hue of the line and its points, out of 360.
    pub(crate) hue: u64,
    /// The face beside the name in the legend, an avatar.
    pub(crate) face: String,
    /// The points in order of `x`, each with the text of its hover.
    pub(crate) points: Vec<Point>,
}

/// One point of a series.
pub(crate) struct Point {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) hover: String,
}

/// One axis: its extent and the ticks with their labels.
pub(crate) struct Axis {
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) ticks: Vec<(f64, String)>,
    /// What the axis measures, written along it.
    pub(crate) title: String,
    /// An icon in front of a tick label, as the address of its image, by tick value.
    pub(crate) icons: Vec<(f64, String)>,
}

impl Axis {
    /// The axis with `title` written along it.
    pub(crate) fn titled(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// An axis from zero to the nice step above `max`, a tick every step;
    /// an axis over nothing spans one.
    pub(crate) fn values(max: f64) -> Self {
        let step = if max > 0.0 {
            nice_step(max / VALUE_TICKS)
        } else {
            1.0
        };
        let top = (max / step).ceil().max(1.0) * step;
        let ticks = (0..=(top / step).round() as u64)
            .map(|tick| {
                let value = tick as f64 * step;
                (value, value_label(value, step))
            })
            .collect();

        Self {
            min: 0.0,
            max: top,
            ticks,
            title: String::new(),
            icons: Vec::new(),
        }
    }

    /// An axis over the whole numbers `first` to `last`, every one a tick
    /// and every `every`th one labelled.
    pub(crate) fn counted(first: u64, last: u64) -> Self {
        let last = last.max(first);
        let count = (last - first + 1) as usize;
        let every = count.div_ceil(HORIZONTAL_LABELS).max(1);
        let ticks = (first..=last)
            .enumerate()
            .map(|(index, value)| {
                let label = if index % every == 0 {
                    value.to_string()
                } else {
                    String::new()
                };
                (value as f64, label)
            })
            .collect();
        // A single value gets room on both sides rather than a zero width.
        let (min, max) = if first == last {
            (first as f64 - 0.5, last as f64 + 0.5)
        } else {
            (first as f64, last as f64)
        };

        Self {
            min,
            max,
            ticks,
            title: String::new(),
            icons: Vec::new(),
        }
    }

    /// An axis from nothing to the whole, a tick every quarter, labelled in percent.
    pub(crate) fn percent() -> Self {
        let ticks = (0..=(PERCENT_MAX / PERCENT_STEP) as u64)
            .map(|tick| {
                let value = tick as f64 * PERCENT_STEP;
                (value, format!("{value}%"))
            })
            .collect();

        Self {
            min: 0.0,
            max: PERCENT_MAX,
            ticks,
            title: String::new(),
            icons: Vec::new(),
        }
    }

    /// An axis of seconds from zero to `max`, ticks at a round span of
    /// minutes or hours.
    pub(crate) fn seconds(max: u64) -> Self {
        const SPANS: [u64; 12] = [
            10, 30, 60, 120, 300, 600, 900, 1200, 1800, 3600, 7200, 14400,
        ];
        let step = SPANS
            .into_iter()
            .find(|span| max.div_ceil(*span) <= HORIZONTAL_LABELS as u64)
            .unwrap_or_else(|| {
                let hours = 3600;
                (max / (HORIZONTAL_LABELS as u64 * hours) + 1) * hours
            });
        let top = max.div_ceil(step).max(1) * step;
        let ticks = (0..=top / step)
            .map(|tick| {
                let value = tick * step;
                (value as f64, ava_run::usage::span(value))
            })
            .collect();

        Self {
            min: 0.0,
            max: top as f64,
            ticks,
            title: String::new(),
            icons: Vec::new(),
        }
    }

    fn span(&self) -> f64 {
        (self.max - self.min).max(f64::MIN_POSITIVE)
    }
}

/// The step of a value axis: one, two or five times a power of ten, the
/// smallest of them at or above `raw`.
fn nice_step(raw: f64) -> f64 {
    let magnitude = 10f64.powf(raw.log10().floor());
    let scaled = raw / magnitude;
    let factor = if scaled <= 1.0 {
        1.0
    } else if scaled <= 2.0 {
        2.0
    } else if scaled <= 5.0 {
        5.0
    } else {
        10.0
    };

    factor * magnitude
}

/// A value of an axis whose ticks are `step` apart, with the decimals the
/// step needs and no more.
fn value_label(value: f64, step: f64) -> String {
    let decimals = if step >= 1.0 {
        0
    } else {
        (-step.log10().floor()) as usize
    };

    format!("{value:.decimals$}")
}

/// The colour of a series of `hue`.
pub(crate) fn color(hue: u64) -> String {
    format!("hsl({hue} {SERIES_SATURATION} {SERIES_LIGHTNESS})")
}

/// The chart of `series` over the axes, drawn `width` wide in `shape`, and
/// a legend of the series under it. `empty` is shown in place of a chart
/// without a point.
pub(crate) fn lines(
    series: &[Series],
    horizontal: &Axis,
    vertical: &Axis,
    shape: Shape,
    quadrants: Option<&Quadrants>,
    width: f64,
    empty: &str,
) -> String {
    if series.iter().all(|series| series.points.is_empty()) {
        return format!("<p class=\"{EMPTY_CLASSES}\">{}</p>", escape(empty));
    }

    let plot_width = width - LEFT - RIGHT;
    // A label wider than its slot wraps at its dashes and spaces into lines
    // that fit the slot, a piece wider than the slot standing alone.
    let slot_width = plot_width / horizontal.ticks.len().max(1) as f64;
    let fits = |text: &str| text.chars().count() as f64 * CHARACTER_WIDTH <= slot_width;
    let lines_of = |label: &str| -> Vec<String> {
        if fits(label) {
            return vec![label.to_string()];
        }
        let mut lines: Vec<String> = Vec::new();
        for piece in pieces(label) {
            match lines.last_mut() {
                Some(line) if fits(&format!("{line}{piece}")) => line.push_str(&piece),
                _ => lines.push(piece),
            }
        }
        lines
            .iter()
            .map(|line| line.trim_end().to_string())
            .collect()
    };
    let deepest = horizontal
        .ticks
        .iter()
        .map(|(_, label)| lines_of(label).len())
        .max()
        .unwrap_or(1);
    let bottom = LABEL_ROOM
        + (deepest - 1) as f64 * LINE_ROOM
        + if horizontal.title.is_empty() {
            0.0
        } else {
            AXIS_TITLE_ROOM
        };
    let plot_height = HEIGHT - TOP - bottom;
    let x_of = |x: f64| LEFT + (x - horizontal.min) / horizontal.span() * plot_width;
    let y_of = |y: f64| TOP + plot_height - (y - vertical.min) / vertical.span() * plot_height;

    let mut svg = format!(
        "<svg class=\"block w-full {LINE_CLASSES}\" viewBox=\"0 0 {width} {HEIGHT}\" font-size=\"{FONT_SIZE}\" role=\"img\">"
    );

    for (value, label) in &vertical.ticks {
        let y = y_of(*value);
        svg.push_str(&format!(
            "<line x1=\"{LEFT}\" y1=\"{y:.1}\" x2=\"{:.1}\" y2=\"{y:.1}\" class=\"{GRID_CLASSES}\"/>\
             <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" class=\"{LABEL_CLASSES}\">{}</text>",
            width - RIGHT,
            LEFT - LABEL_GAP,
            y + LABEL_LIFT,
            escape(label)
        ));
    }

    let baseline = y_of(vertical.min);
    for (value, label) in &horizontal.ticks {
        let x = x_of(*value);
        svg.push_str(&format!(
            "<line x1=\"{x:.1}\" y1=\"{baseline:.1}\" x2=\"{x:.1}\" y2=\"{:.1}\" class=\"{AXIS_CLASSES}\"/>",
            baseline + LABEL_GAP / 2.0
        ));
        if label.is_empty() {
            continue;
        }
        // An icon stands in front of the label and the two centre on the tick together.
        let lines = lines_of(label);
        let icon = horizontal
            .icons
            .iter()
            .find(|(at, _)| at == value)
            .map(|(_, address)| address);
        let text_width = lines
            .iter()
            .map(|line| line.chars().count() as f64 * CHARACTER_WIDTH)
            .fold(0.0, f64::max);
        let (text_x, anchor) = match icon {
            Some(address) => {
                let left = x - (ICON_SIDE + ICON_GAP + text_width) / 2.0;
                svg.push_str(&format!(
                    "<image href=\"{}\" x=\"{left:.1}\" y=\"{:.1}\" width=\"{ICON_SIDE}\" height=\"{ICON_SIDE}\" opacity=\"{ICON_OPACITY}\"/>",
                    escape(address),
                    baseline + AXIS_LABEL_DROP - LABEL_LIFT - ICON_SIDE / 2.0
                ));
                (left + ICON_SIDE + ICON_GAP, "start")
            }
            None => (x, "middle"),
        };
        let spans: String = lines
            .iter()
            .enumerate()
            .map(|(line, text)| {
                format!(
                    "<tspan x=\"{text_x:.1}\" dy=\"{}\">{}</tspan>",
                    if line == 0 { 0.0 } else { LINE_ROOM },
                    escape(text)
                )
            })
            .collect();
        svg.push_str(&format!(
            "<text x=\"{text_x:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" class=\"{LABEL_CLASSES}\">{spans}</text>",
            baseline + AXIS_LABEL_DROP
        ));
    }
    svg.push_str(&format!(
        "<line x1=\"{LEFT}\" y1=\"{baseline:.1}\" x2=\"{:.1}\" y2=\"{baseline:.1}\" class=\"{AXIS_CLASSES}\"/>",
        width - RIGHT
    ));
    if !horizontal.title.is_empty() {
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" class=\"{AXIS_TITLE_CLASSES}\">{}</text>",
            LEFT + plot_width / 2.0,
            HEIGHT - AXIS_TITLE_INSET,
            escape(&horizontal.title)
        ));
    }
    // Rotated a quarter turn left, so its x runs up the drawing and its y in from the left edge.
    if !vertical.title.is_empty() {
        svg.push_str(&format!(
            "<text transform=\"rotate(-90)\" x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" class=\"{AXIS_TITLE_CLASSES}\">{}</text>",
            -(TOP + plot_height / 2.0),
            AXIS_TITLE_INSET + f64::from(FONT_SIZE),
            escape(&vertical.title)
        ));
    }

    if let Some(quadrants) = quadrants {
        let middle_x = x_of((horizontal.min + horizontal.max) / 2.0);
        let middle_y = y_of((vertical.min + vertical.max) / 2.0);
        let (left, right, top, bottom) = (LEFT, width - RIGHT, TOP, baseline);
        let [top_left, top_right, bottom_left, bottom_right] = &quadrants.labels;
        svg.push_str(&format!(
            "<rect x=\"{left}\" y=\"{top}\" width=\"{:.1}\" height=\"{:.1}\" class=\"{GOOD_QUADRANT_CLASSES}\"/>\
             <rect x=\"{middle_x:.1}\" y=\"{middle_y:.1}\" width=\"{:.1}\" height=\"{:.1}\" class=\"{BAD_QUADRANT_CLASSES}\"/>\
             <rect x=\"{middle_x:.1}\" y=\"{top}\" width=\"{:.1}\" height=\"{:.1}\" class=\"{MIXED_QUADRANT_CLASSES}\"/>\
             <rect x=\"{left}\" y=\"{middle_y:.1}\" width=\"{:.1}\" height=\"{:.1}\" class=\"{MIXED_QUADRANT_CLASSES}\"/>\
             <line x1=\"{middle_x:.1}\" y1=\"{top}\" x2=\"{middle_x:.1}\" y2=\"{bottom:.1}\" stroke-dasharray=\"{QUADRANT_DASHES}\" class=\"{QUADRANT_LINE_CLASSES}\"/>\
             <line x1=\"{left}\" y1=\"{middle_y:.1}\" x2=\"{right:.1}\" y2=\"{middle_y:.1}\" stroke-dasharray=\"{QUADRANT_DASHES}\" class=\"{QUADRANT_LINE_CLASSES}\"/>\
             <text x=\"{:.1}\" y=\"{:.1}\" class=\"{QUADRANT_LABEL_CLASSES}\">{}</text>\
             <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" class=\"{QUADRANT_LABEL_CLASSES}\">{}</text>\
             <text x=\"{:.1}\" y=\"{:.1}\" class=\"{QUADRANT_LABEL_CLASSES}\">{}</text>\
             <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" class=\"{QUADRANT_LABEL_CLASSES}\">{}</text>",
            middle_x - left,
            middle_y - top,
            right - middle_x,
            bottom - middle_y,
            right - middle_x,
            middle_y - top,
            middle_x - left,
            bottom - middle_y,
            left + QUADRANT_LABEL_INSET,
            top + QUADRANT_LABEL_DROP,
            escape(top_left),
            right - QUADRANT_LABEL_INSET,
            top + QUADRANT_LABEL_DROP,
            escape(top_right),
            left + QUADRANT_LABEL_INSET,
            bottom - QUADRANT_LABEL_INSET,
            escape(bottom_left),
            right - QUADRANT_LABEL_INSET,
            bottom - QUADRANT_LABEL_INSET,
            escape(bottom_right)
        ));
    }

    let all_series = series;
    for (index, series) in series.iter().enumerate() {
        if series.points.is_empty() {
            continue;
        }
        let color = color(series.hue);
        svg.push_str(&format!(
            "<g class=\"{SERIES_CLASS} {SERIES_CLASS}-{index}\">"
        ));
        // The line, the wide invisible stroke over it that names the series
        // wherever the cursor rests on the line, and the marks on top with
        // hovers of their own. The rules below light the group and its legend
        // entry together. A scatter has no line, its marks stand alone.
        if shape == Shape::Bars {
            for point in &series.points {
                // The series with a bar in this slot share it, in their order.
                let sharing: Vec<usize> = all_series
                    .iter()
                    .enumerate()
                    .filter(|(_, other)| other.points.iter().any(|other| other.x == point.x))
                    .map(|(other, _)| other)
                    .collect();
                let lane = BAR_GROUP_SHARE / sharing.len().max(1) as f64;
                let gap = lane * BAR_LANE_GAP;
                let rank = sharing
                    .iter()
                    .position(|other| *other == index)
                    .unwrap_or_default();
                let left = x_of(point.x - BAR_GROUP_SHARE / 2.0 + rank as f64 * lane + gap);
                let top = y_of(point.y);
                svg.push_str(&format!(
                    "<rect x=\"{left:.1}\" y=\"{top:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{color}\" fill-opacity=\"{BAR_FILL_OPACITY}\" stroke=\"{color}\" stroke-width=\"{BAR_EDGE_WIDTH}\" class=\"{STROKE_CLASS}\"><title>{}</title></rect>",
                    x_of(point.x + lane - 2.0 * gap) - x_of(point.x),
                    baseline - top,
                    escape(&point.hover)
                ));
            }
        } else if shape != Shape::Scatter {
            let mut path = String::new();
            for (index, point) in series.points.iter().enumerate() {
                let (x, y) = (x_of(point.x), y_of(point.y));
                if index == 0 {
                    path.push_str(&format!("M{x:.1} {y:.1}"));
                } else if shape == Shape::Stepped {
                    path.push_str(&format!(" H{x:.1} V{y:.1}"));
                } else {
                    path.push_str(&format!(" L{x:.1} {y:.1}"));
                }
            }
            svg.push_str(&format!(
                "<path d=\"{path}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{LINE_WIDTH}\" stroke-linejoin=\"round\" stroke-linecap=\"round\" class=\"{STROKE_CLASS}\"/>\
                 <path d=\"{path}\" fill=\"none\" stroke=\"transparent\" stroke-width=\"{HIT_WIDTH}\" stroke-linejoin=\"round\" stroke-linecap=\"round\" pointer-events=\"stroke\"><title>{}</title></path>",
                escape(&series.hover)
            ));
        }
        let radius = if shape == Shape::Scatter {
            MARK_RADIUS
        } else {
            POINT_RADIUS
        };
        // A point without a hover is a bend of the line, not a mark on it.
        for point in series
            .points
            .iter()
            .filter(|point| shape != Shape::Bars && !point.hover.is_empty())
        {
            svg.push_str(&format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{radius}\" fill=\"{color}\" class=\"{POINT_STROKE_CLASSES}\" stroke-width=\"1.5\"><title>{}</title></circle>",
                x_of(point.x),
                y_of(point.y),
                escape(&point.hover)
            ));
        }
        svg.push_str("</g>");
    }
    svg.push_str("</svg>");

    let legend: String = series
        .iter()
        .enumerate()
        .map(|(index, series)| {
            format!(
                "<span class=\"{LEGEND_CLASS} {LEGEND_CLASS}-{index} {LEGEND_ITEM_CLASSES}\">{}<span class=\"{SWATCH_CLASSES}\" style=\"background:{}\"></span><span class=\"font-mono\">{}</span></span>",
                series.face,
                color(series.hue),
                escape(&series.label)
            )
        })
        .collect();

    format!(
        "<div class=\"{CHART_CLASS}\"><style>{}</style>{svg}<div class=\"{LEGEND_CLASSES}\">{legend}</div></div>",
        hover_rules(series)
    )
}

/// `label` cut after every break character, each piece keeping its break.
fn pieces(label: &str) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut piece = String::new();
    for character in label.chars() {
        piece.push(character);
        if LABEL_BREAKS.contains(&character) {
            pieces.push(std::mem::take(&mut piece));
        }
    }
    if !piece.is_empty() {
        pieces.push(piece);
    }

    pieces
}

/// The rules lighting a line and its legend entry together: while either is
/// under the cursor the line thickens, the other lines fade and the entry
/// takes a ground and the colour of the line. They are scoped to the chart
/// through `:has`, so several charts on a page leave each other alone.
fn hover_rules(series: &[Series]) -> String {
    let mut rules = format!(
        ".{CHART_CLASS} .{SERIES_CLASS},.{CHART_CLASS} .{LEGEND_CLASS}{{transition:opacity {HOVER_TRANSITION},background-color {HOVER_TRANSITION},color {HOVER_TRANSITION}}}\
         .{CHART_CLASS}:has(.{SERIES_CLASS}:hover) .{SERIES_CLASS}:not(:hover),\
         .{CHART_CLASS}:has(.{LEGEND_CLASS}:hover) .{SERIES_CLASS}{{opacity:{DIMMED_OPACITY}}}\
         .{CHART_CLASS} .{SERIES_CLASS}:hover .{STROKE_CLASS}{{stroke-width:{HOVERED_LINE_WIDTH}}}"
    );
    for (index, series) in series.iter().enumerate() {
        rules.push_str(&format!(
            ".{CHART_CLASS}:has(.{SERIES_CLASS}-{index}:hover) .{LEGEND_CLASS}-{index},\
             .{CHART_CLASS}:has(.{LEGEND_CLASS}-{index}:hover) .{LEGEND_CLASS}-{index}{{background-color:{LIT_LEGEND_GROUND};color:{}}}\
             .{CHART_CLASS}:has(.{LEGEND_CLASS}-{index}:hover) .{SERIES_CLASS}-{index}{{opacity:1}}\
             .{CHART_CLASS}:has(.{LEGEND_CLASS}-{index}:hover) .{SERIES_CLASS}-{index} .{STROKE_CLASS}{{stroke-width:{HOVERED_LINE_WIDTH}}}",
            color(series.hue)
        ));
    }

    rules
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::{Axis, Point, Series, Shape};

    #[test]
    fn a_value_axis_ends_on_a_nice_step_above_its_maximum() {
        let axis = Axis::values(7.0);
        assert_eq!(axis.max, 8.0);
        assert_eq!(axis.ticks.len(), 5);
        assert_eq!(axis.ticks[1].1, "2");

        let axis = Axis::values(9400.0);
        assert_eq!(axis.max, 10000.0);
        assert_eq!(axis.ticks.last().unwrap().1, "10000");

        let axis = Axis::values(0.0);
        assert_eq!(axis.ticks.len(), 2);
        assert_eq!(axis.ticks[1].1, "1");

        let axis = Axis::values(0.3);
        assert_eq!(axis.ticks[1].1, "0.1");
    }

    #[test]
    fn a_counted_axis_thins_its_labels_and_gives_one_value_room() {
        let axis = Axis::counted(1, 30);
        assert_eq!(axis.ticks.len(), 30);
        assert_eq!(axis.ticks[0].1, "1");
        assert_eq!(axis.ticks[1].1, "");
        assert_eq!(axis.ticks[3].1, "4");

        let axis = Axis::counted(2, 2);
        assert!(axis.min < 2.0 && axis.max > 2.0);
    }

    #[test]
    fn a_label_too_wide_for_its_slot_breaks_at_its_last_dash() {
        let series = [Series {
            label: "a".to_string(),
            hover: String::new(),
            hue: 0,
            face: String::new(),
            points: (0..8)
                .map(|slot| Point {
                    x: slot as f64,
                    y: 1.0,
                    hover: "bar".to_string(),
                })
                .collect(),
        }];
        let horizontal = Axis {
            min: -0.5,
            max: 7.5,
            ticks: (0..8)
                .map(|slot| (slot as f64, "r2wars-parity-max".to_string()))
                .collect(),
            title: String::new(),
            icons: Vec::new(),
        };
        let bars = super::lines(
            &series,
            &horizontal,
            &Axis::values(1.0),
            Shape::Bars,
            None,
            super::NARROW_WIDTH,
            "nothing",
        );

        assert!(bars.contains(">r2wars-</tspan>"));
        assert!(bars.contains(">parity-max</tspan>"));
    }

    #[test]
    fn a_label_wraps_at_spaces_too_and_deepens_the_bottom() {
        assert_eq!(
            super::pieces("deepseek-v4.1-flash high"),
            ["deepseek-", "v4.1-", "flash ", "high"]
        );

        let series = vec![Series {
            label: "a".to_string(),
            hover: String::new(),
            hue: 0,
            face: String::new(),
            points: (0..9)
                .map(|slot| Point {
                    x: slot as f64,
                    y: 0.5,
                    hover: String::new(),
                })
                .collect(),
        }];
        let horizontal = Axis {
            min: -0.5,
            max: 8.5,
            ticks: (0..9)
                .map(|slot| (slot as f64, "deepseek-v4.1-flash high".to_string()))
                .collect(),
            title: String::new(),
            icons: Vec::new(),
        };
        let bars = super::lines(
            &series,
            &horizontal,
            &Axis::values(1.0),
            Shape::Bars,
            None,
            super::NARROW_WIDTH,
            "nothing",
        );

        assert!(bars.contains(">deepseek-</tspan>"));
        assert!(bars.contains(">flash</tspan>"));
        assert!(bars.contains(">high</tspan>"));
    }

    #[test]
    fn a_percent_axis_ticks_every_quarter() {
        let axis = Axis::percent();
        assert_eq!(axis.max, 100.0);
        assert_eq!(axis.ticks.len(), 5);
        assert_eq!(axis.ticks[1].1, "25%");
    }

    #[test]
    fn a_seconds_axis_ticks_at_round_spans() {
        let axis = Axis::seconds(3638);
        assert_eq!(axis.max, 4200.0);
        assert_eq!(axis.ticks[1].1, "10m");
        assert_eq!(axis.ticks[0].1, "0s");
    }

    #[test]
    fn a_stepped_line_holds_its_value_until_the_next_point() {
        let series = [Series {
            label: "a".to_string(),
            hover: "a on b".to_string(),
            hue: 10,
            face: String::new(),
            points: vec![
                Point {
                    x: 0.0,
                    y: 1.0,
                    hover: "first".to_string(),
                },
                Point {
                    x: 2.0,
                    y: 3.0,
                    hover: "second".to_string(),
                },
            ],
        }];
        let horizontal = Axis::counted(0, 2).titled("round");
        let vertical = Axis::values(3.0).titled("points");

        let stepped = super::lines(
            &series,
            &horizontal,
            &vertical,
            Shape::Stepped,
            None,
            super::NARROW_WIDTH,
            "nothing",
        );
        assert!(stepped.contains(" H"));
        assert!(stepped.contains(">round</text>"));
        assert!(stepped.contains("rotate(-90)"));
        assert!(stepped.contains(">points</text>"));
        assert!(stepped.contains("<title>second</title>"));
        assert!(stepped.contains("<title>a on b</title>"));
        assert!(stepped.contains(".line-chart:has(.series-0:hover) .legend-0"));
        assert!(stepped.contains("class=\"legend legend-0 "));
        assert!(stepped.contains("hsl(10 60% 55%)"));

        let straight = super::lines(
            &series,
            &horizontal,
            &vertical,
            Shape::Straight,
            None,
            super::WIDE_WIDTH,
            "nothing",
        );
        assert!(straight.contains("viewBox=\"0 0 1200 240\""));
        assert!(straight.contains(" L"));

        let scatter = super::lines(
            &series,
            &horizontal,
            &vertical,
            Shape::Scatter,
            Some(&super::Quadrants {
                labels: ["a b", "c", "d", "e"].map(str::to_string),
            }),
            super::NARROW_WIDTH,
            "nothing",
        );
        assert!(!scatter.contains("<path"));
        assert!(scatter.contains(">a b</text>"));
        assert_eq!(scatter.matches("stroke-dasharray").count(), 2);

        let bars = super::lines(
            &series,
            &horizontal,
            &vertical,
            Shape::Bars,
            None,
            super::NARROW_WIDTH,
            "nothing",
        );
        assert_eq!(bars.matches("<rect").count(), 2);
        assert!(!bars.contains("<circle"));
        assert!(bars.contains("<title>second</title>"));
        assert_eq!(scatter.matches("<circle").count(), 2);
        assert!(scatter.contains("r=\"5\""));

        let empty = super::lines(
            &[],
            &horizontal,
            &vertical,
            Shape::Straight,
            None,
            super::NARROW_WIDTH,
            "nothing",
        );
        assert!(empty.contains("nothing"));
        assert!(!empty.contains("<svg"));
    }
}
