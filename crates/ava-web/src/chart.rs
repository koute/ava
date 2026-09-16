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
/// The room the labels of the vertical axis take on the left.
const LEFT: f64 = 48.0;
/// The room the labels of the horizontal axis take at the bottom.
const BOTTOM: f64 = 28.0;
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
const FONT_SIZE: u32 = 11;
const LINE_WIDTH: f64 = 1.25;
/// The width of the line under the cursor.
const HOVERED_LINE_WIDTH: f64 = 2.5;
/// The width of the invisible stroke the cursor finds a line by, since a
/// line two pixels wide is hard to rest on.
const HIT_WIDTH: f64 = 14.0;
const POINT_RADIUS: f64 = 2.75;
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
const POINT_STROKE_CLASSES: &str = "stroke-neutral-900";
const LEGEND_CLASSES: &str = "flex flex-wrap gap-x-4 gap-y-1.5 mt-3 text-xs text-neutral-300";
const LEGEND_ITEM_CLASSES: &str =
    "flex items-center gap-1.5 whitespace-nowrap rounded px-1.5 py-0.5 -mx-1.5 cursor-default";
const SWATCH_CLASSES: &str = "inline-block h-0.5 w-4 rounded-full";
const EMPTY_CLASSES: &str = "text-neutral-500 text-xs";

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
}

impl Axis {
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

        Self { min, max, ticks }
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

/// The chart of `series` over the axes, drawn `width` wide, every line
/// stepping from point to point when `stepped`, so a value holds until the
/// next one is banked, and a legend of the series under it. `empty` is shown
/// in place of a chart without a point.
pub(crate) fn lines(
    series: &[Series],
    horizontal: &Axis,
    vertical: &Axis,
    stepped: bool,
    width: f64,
    empty: &str,
) -> String {
    if series.iter().all(|series| series.points.is_empty()) {
        return format!("<p class=\"{EMPTY_CLASSES}\">{}</p>", escape(empty));
    }

    let plot_width = width - LEFT - RIGHT;
    let plot_height = HEIGHT - TOP - BOTTOM;
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
        if !label.is_empty() {
            svg.push_str(&format!(
                "<text x=\"{x:.1}\" y=\"{:.1}\" text-anchor=\"middle\" class=\"{LABEL_CLASSES}\">{}</text>",
                baseline + AXIS_LABEL_DROP,
                escape(label)
            ));
        }
    }
    svg.push_str(&format!(
        "<line x1=\"{LEFT}\" y1=\"{baseline:.1}\" x2=\"{:.1}\" y2=\"{baseline:.1}\" class=\"{AXIS_CLASSES}\"/>",
        width - RIGHT
    ));

    for (index, series) in series.iter().enumerate() {
        if series.points.is_empty() {
            continue;
        }
        let color = color(series.hue);
        let mut path = String::new();
        for (index, point) in series.points.iter().enumerate() {
            let (x, y) = (x_of(point.x), y_of(point.y));
            if index == 0 {
                path.push_str(&format!("M{x:.1} {y:.1}"));
            } else if stepped {
                path.push_str(&format!(" H{x:.1} V{y:.1}"));
            } else {
                path.push_str(&format!(" L{x:.1} {y:.1}"));
            }
        }
        // The line, the wide invisible stroke over it that names the series
        // wherever the cursor rests on the line, and the marks on top with
        // hovers of their own. The rules below light the group and its legend
        // entry together.
        svg.push_str(&format!(
            "<g class=\"{SERIES_CLASS} {SERIES_CLASS}-{index}\">\
             <path d=\"{path}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{LINE_WIDTH}\" stroke-linejoin=\"round\" stroke-linecap=\"round\" class=\"{STROKE_CLASS}\"/>\
             <path d=\"{path}\" fill=\"none\" stroke=\"transparent\" stroke-width=\"{HIT_WIDTH}\" stroke-linejoin=\"round\" stroke-linecap=\"round\" pointer-events=\"stroke\"><title>{}</title></path>",
            escape(&series.hover)
        ));
        // A point without a hover is a bend of the line, not a mark on it.
        for point in series.points.iter().filter(|point| !point.hover.is_empty()) {
            svg.push_str(&format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{POINT_RADIUS}\" fill=\"{color}\" class=\"{POINT_STROKE_CLASSES}\" stroke-width=\"1.5\"><title>{}</title></circle>",
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
    use super::{Axis, Point, Series};

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
        let horizontal = Axis::counted(0, 2);
        let vertical = Axis::values(3.0);

        let stepped = super::lines(
            &series,
            &horizontal,
            &vertical,
            true,
            super::NARROW_WIDTH,
            "nothing",
        );
        assert!(stepped.contains(" H"));
        assert!(stepped.contains("<title>second</title>"));
        assert!(stepped.contains("<title>a on b</title>"));
        assert!(stepped.contains(".line-chart:has(.series-0:hover) .legend-0"));
        assert!(stepped.contains("class=\"legend legend-0 "));
        assert!(stepped.contains("hsl(10 60% 55%)"));

        let straight = super::lines(
            &series,
            &horizontal,
            &vertical,
            false,
            super::WIDE_WIDTH,
            "nothing",
        );
        assert!(straight.contains("viewBox=\"0 0 1200 240\""));
        assert!(straight.contains(" L"));

        let empty = super::lines(
            &[],
            &horizontal,
            &vertical,
            false,
            super::NARROW_WIDTH,
            "nothing",
        );
        assert!(empty.contains("nothing"));
        assert!(!empty.contains("<svg"));
    }
}
