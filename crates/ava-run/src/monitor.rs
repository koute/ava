//! Watching the agent output for signs of a stuck run.

/// A line shorter than this is too generic to count as a repeat.
const MINIMUM_LINE_BYTES: usize = 16;

/// Consecutive repeats of one line before the run counts as looping.
pub(crate) const REPEATED_LINE_THRESHOLD: u32 = 10;

/// What the output reader threads report and the run loop reads.
struct Inner {
    output_bytes: u64,
    last_output: std::time::Instant,
    /// The current line, hashed as it streams so no line is buffered.
    line_hasher: std::collections::hash_map::DefaultHasher,
    line_bytes: usize,
    last_line: Option<u64>,
    repeats: u32,
    doom_looping: bool,
    /// What the harness prints once it carries on with another model.
    fallback_marker: Option<&'static [u8]>,
    /// The tail of the stream a marker split between two reads starts in.
    marker_carry: Vec<u8>,
    fell_back: bool,
}

/// The agent output statistics shared between the reader threads and the run
/// loop.
pub(crate) struct Monitor {
    inner: std::sync::Mutex<Inner>,
}

impl Monitor {
    /// A monitor watching for the refusal fallback marker of `harness`, if
    /// it has one.
    pub(crate) fn new(harness: &str) -> Self {
        Self {
            inner: std::sync::Mutex::new(Inner {
                output_bytes: 0,
                last_output: std::time::Instant::now(),
                line_hasher: Default::default(),
                line_bytes: 0,
                last_line: None,
                repeats: 0,
                doom_looping: false,
                fallback_marker: crate::registry::refusal_fallback_marker(harness)
                    .map(str::as_bytes),
                marker_carry: Vec::new(),
                fell_back: false,
            }),
        }
    }

    /// Start watching a new sandbox on the counters of the run.
    ///
    /// The bytes are the console of the whole run, so they carry over, as does
    /// a fallback, since the harness keeps the session on the other model. The
    /// silence clock and the repeat detector are about the process that is
    /// live and start over with it.
    pub(crate) fn restart(&self) {
        let mut inner = self.lock();
        inner.last_output = std::time::Instant::now();
        inner.line_hasher = Default::default();
        inner.line_bytes = 0;
        inner.last_line = None;
        inner.repeats = 0;
        inner.doom_looping = false;
        inner.marker_carry.clear();
    }

    /// Count `chunk` and, on the line scanned stream, watch for repeats.
    pub(crate) fn observe(&self, chunk: &[u8], scan_lines: bool) {
        let mut inner = self.lock();
        inner.output_bytes += chunk.len() as u64;
        inner.last_output = std::time::Instant::now();

        if !scan_lines {
            return;
        }

        inner.scan_fallback(chunk);

        let mut rest = chunk;
        while let Some(position) = rest.iter().position(|byte| *byte == b'\n') {
            let (tail, remainder) = rest.split_at(position);
            inner.extend_line(tail);
            inner.complete_line();
            rest = &remainder[1..];
        }
        inner.extend_line(rest);
    }

    /// The bytes the agent printed so far.
    pub(crate) fn output_bytes(&self) -> u64 {
        self.lock().output_bytes
    }

    /// How long the agent has printed nothing.
    pub(crate) fn silent_for(&self) -> std::time::Duration {
        self.lock().last_output.elapsed()
    }

    /// Whether the harness reported answering with another model than the one
    /// it was started on.
    pub(crate) fn fell_back(&self) -> bool {
        self.lock().fell_back
    }

    /// Whether one line repeated often enough to look like a loop.
    pub(crate) fn doom_looping(&self) -> bool {
        self.lock().doom_looping
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("the monitor is not poisoned")
    }
}

impl Inner {
    fn scan_fallback(&mut self, chunk: &[u8]) {
        let Some(marker) = self.fallback_marker else {
            return;
        };

        let mut window = std::mem::take(&mut self.marker_carry);
        window.extend_from_slice(chunk);
        self.fell_back |= window
            .windows(marker.len())
            .any(|candidate| candidate == marker);

        let kept = window.len().min(marker.len() - 1);
        self.marker_carry = window.split_off(window.len() - kept);
    }

    fn extend_line(&mut self, bytes: &[u8]) {
        std::hash::Hasher::write(&mut self.line_hasher, bytes);
        self.line_bytes += bytes.len();
    }

    /// Compare the completed line against the one before it.
    fn complete_line(&mut self) {
        let hash = std::hash::Hasher::finish(&self.line_hasher);
        self.line_hasher = Default::default();
        let bytes = std::mem::take(&mut self.line_bytes);

        if bytes < MINIMUM_LINE_BYTES {
            self.last_line = None;
            self.repeats = 0;
            return;
        }

        if self.last_line == Some(hash) {
            self.repeats += 1;
            self.doom_looping |= self.repeats >= REPEATED_LINE_THRESHOLD;
        } else {
            self.last_line = Some(hash);
            self.repeats = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    const FALLBACK_LINE: &[u8] = b"{\"type\":\"system\",\"subtype\":\"model_refusal_fallback\"}\n";

    #[test]
    fn fallback_is_seen_across_reads() {
        let monitor = super::Monitor::new(crate::registry::CLAUDE_HARNESS);
        let (first, second) = FALLBACK_LINE.split_at(FALLBACK_LINE.len() / 2);

        monitor.observe(first, true);
        assert!(!monitor.fell_back());
        monitor.observe(second, true);
        assert!(monitor.fell_back());
    }

    #[test]
    fn fallback_is_only_read_from_the_event_stream() {
        let monitor = super::Monitor::new(crate::registry::CLAUDE_HARNESS);
        monitor.observe(FALLBACK_LINE, false);
        assert!(!monitor.fell_back());
    }

    #[test]
    fn fallback_outlives_a_restart() {
        let monitor = super::Monitor::new(crate::registry::CLAUDE_HARNESS);
        monitor.observe(FALLBACK_LINE, true);
        monitor.restart();
        assert!(monitor.fell_back());
    }

    #[test]
    fn harness_without_marker_never_falls_back() {
        let monitor = super::Monitor::new(crate::registry::PI_HARNESS);
        monitor.observe(FALLBACK_LINE, true);
        assert!(!monitor.fell_back());
    }
}
