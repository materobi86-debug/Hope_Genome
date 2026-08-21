//! Organism state model + ratatui rendering.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Row, Table};
use ratatui::Frame;

/// Status of one module in the organism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus {
    Alive,
    Stressed,
    Apoptotic,
    Regenerating,
}

impl ModuleStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ModuleStatus::Alive => "ALIVE",
            ModuleStatus::Stressed => "STRESSED",
            ModuleStatus::Apoptotic => "APOPTOTIC",
            ModuleStatus::Regenerating => "REGEN",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            ModuleStatus::Alive => Color::Green,
            ModuleStatus::Stressed => Color::Yellow,
            ModuleStatus::Apoptotic => Color::Red,
            ModuleStatus::Regenerating => Color::Cyan,
        }
    }
}

/// One module row in the dashboard.
#[derive(Debug, Clone)]
pub struct ModuleView {
    pub name: &'static str,
    /// Bioelectric voltage 0..=1000 (milli-units).
    pub voltage_milli: u32,
    pub status: ModuleStatus,
}

/// A link in the ASCII network graph (mycorrhiza / symbiont edges).
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Mycorrhiza,
    Symbiont,
}

impl EdgeKind {
    pub fn glyph(&self) -> &'static str {
        match self {
            EdgeKind::Mycorrhiza => "~",
            EdgeKind::Symbiont => "=",
        }
    }
}

/// The whole organism snapshot rendered each tick.
#[derive(Debug, Clone, Default)]
pub struct OrganismState {
    pub modules: Vec<ModuleView>,
    pub edges: Vec<GraphEdge>,
    /// Event log lines (newest last).
    pub events: Vec<String>,
    pub tick: u64,
}

impl OrganismState {
    /// Build the default 21-module organism with nominal voltages.
    pub fn default_organism() -> Self {
        let names = [
            "tardigrade-tun",
            "physarum-path",
            "tcell-sentinel",
            "crispr-patch",
            "symbiont-engine",
            "epigenetic-switch",
            "quorum-signal",
            "mycorrhiza-trade",
            "plasmid-conjugate",
            "blastema-regen",
            "chameleon-stealth",
            "electrocyte-burst",
            "magnetosome-nav",
            "xenobot-swarm",
            "apoptosis-cascade",
            "yamanaka-deaging",
            "morpho-electric-mesh",
            "biophoton-telepathy",
            "transposon-hive",
            "mycelial-void-vault",
            "chimera-fusion",
        ];
        let modules = names
            .iter()
            .enumerate()
            .map(|(i, name)| ModuleView {
                name,
                voltage_milli: 500 + ((i as u32 * 37) % 300),
                status: ModuleStatus::Alive,
            })
            .collect();
        let edges = vec![
            GraphEdge { from: "host".into(), to: "gpu-node".into(), kind: EdgeKind::Symbiont },
            GraphEdge { from: "host".into(), to: "edge-mcu".into(), kind: EdgeKind::Mycorrhiza },
            GraphEdge { from: "gpu-node".into(), to: "edge-mcu".into(), kind: EdgeKind::Mycorrhiza },
        ];
        Self {
            modules,
            edges,
            events: vec!["organism online".into()],
            tick: 0,
        }
    }

    /// Advance one tick: jitter voltages, derive status from voltage.
    pub fn tick(&mut self) {
        self.tick += 1;
        for m in &mut self.modules {
            // Deterministic pseudo-jitter so tests are stable.
            let jitter = ((self.tick * 31 + m.name.len() as u64 * 17) % 60) as i64 - 30;
            let v = m.voltage_milli as i64 + jitter;
            m.voltage_milli = v.clamp(0, 1000) as u32;
            m.status = match m.voltage_milli {
                0..=150 => ModuleStatus::Apoptotic,
                151..=350 => ModuleStatus::Stressed,
                _ => ModuleStatus::Alive,
            };
        }
    }

    /// Simulate an apoptosis → blastema regeneration cycle on one module.
    pub fn simulate_regeneration(&mut self, name: &str) {
        if let Some(m) = self.modules.iter_mut().find(|m| m.name == name) {
            m.status = ModuleStatus::Apoptotic;
            m.voltage_milli = 0;
            self.events.push(format!("{name}: APOPTOSIS — node down"));
        }
        // Next ticks bring it back through REGEN.
        if let Some(m) = self.modules.iter_mut().find(|m| m.name == name) {
            m.status = ModuleStatus::Regenerating;
            self.events.push(format!("{name}: blastema rebuilding from parity shards"));
        }
    }

    pub fn push_event(&mut self, line: impl Into<String>) {
        self.events.push(line.into());
        if self.events.len() > 50 {
            self.events.remove(0);
        }
    }
}

/// Render the organism into a ratatui frame.
pub fn draw(f: &mut Frame, state: &OrganismState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(8),
            Constraint::Length(6),
        ])
        .split(f.area());

    draw_header(f, chunks[0], state);
    draw_modules(f, chunks[1], state);
    draw_graph(f, chunks[2], state);
    draw_events(f, chunks[3], state);
}

fn draw_header(f: &mut Frame, area: Rect, state: &OrganismState) {
    let alive = state
        .modules
        .iter()
        .filter(|m| m.status == ModuleStatus::Alive)
        .count();
    let title = Line::from(vec![
        Span::styled("🧬 BIO-ORGANISM ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw(format!(
            "tick {} · {}/{} modules alive",
            state.tick,
            alive,
            state.modules.len()
        )),
    ]);
    let block = Block::default().borders(Borders::ALL).title(title);
    f.render_widget(block, area);
}

fn draw_modules(f: &mut Frame, area: Rect, state: &OrganismState) {
    let rows: Vec<Row> = state
        .modules
        .iter()
        .map(|m| {
            Row::new(vec![
                Line::raw(m.name),
                Line::styled(
                    format!("{:>4} mV", m.voltage_milli),
                    Style::default().fg(m.status.color()),
                ),
                Line::styled(m.status.label(), Style::default().fg(m.status.color())),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(22),
            Constraint::Length(10),
            Constraint::Length(12),
        ],
    )
    .header(Row::new(vec!["MODULE", "VOLTAGE", "STATUS"]).style(Style::default().add_modifier(Modifier::BOLD)))
    .block(Block::default().borders(Borders::ALL).title("modules"));
    f.render_widget(table, area);
}

fn draw_graph(f: &mut Frame, area: Rect, state: &OrganismState) {
    let mut lines: Vec<Line> = Vec::new();
    for e in &state.edges {
        lines.push(Line::from(format!(
            "  {} {}{}> {}",
            e.from,
            e.kind.glyph(),
            e.kind.glyph(),
            e.to
        )));
    }
    if lines.is_empty() {
        lines.push(Line::raw("  (no links)"));
    }
    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("mycorrhiza / symbiont graph"),
    );
    f.render_widget(p, area);
}

fn draw_events(f: &mut Frame, area: Rect, state: &OrganismState) {
    let recent: Vec<Line> = state
        .events
        .iter()
        .rev()
        .take(4)
        .map(|e| Line::raw(format!("· {e}")))
        .collect();
    let p = Paragraph::new(recent).block(Block::default().borders(Borders::ALL).title("events"));
    f.render_widget(p, area);
}

/// Render a single module's voltage as a gauge (used in detail views).
pub fn voltage_gauge(m: &ModuleView) -> Gauge<'static> {
    Gauge::default()
        .block(Block::default().title(m.name.to_string()).borders(Borders::ALL))
        .gauge_style(Style::default().fg(m.status.color()))
        .percent((m.voltage_milli / 10) as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_organism_has_21_modules() {
        let state = OrganismState::default_organism();
        assert_eq!(state.modules.len(), 21);
        assert!(state.modules.iter().all(|m| m.status == ModuleStatus::Alive));
    }

    #[test]
    fn tick_keeps_voltage_in_range() {
        let mut state = OrganismState::default_organism();
        for _ in 0..50 {
            state.tick();
        }
        assert!(state.modules.iter().all(|m| m.voltage_milli <= 1000));
    }

    #[test]
    fn regeneration_cycle_marks_module() {
        let mut state = OrganismState::default_organism();
        state.simulate_regeneration("blastema-regen");
        let m = state.modules.iter().find(|m| m.name == "blastema-regen").unwrap();
        assert_eq!(m.status, ModuleStatus::Regenerating);
        assert!(state.events.iter().any(|e| e.contains("APOPTOSIS")));
        assert!(state.events.iter().any(|e| e.contains("blastema")));
    }

    #[test]
    fn renders_without_panic() {
        let state = OrganismState::default_organism();
        let backend = ratatui::backend::TestBackend::new(80, 30);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, &state)).unwrap();
        let buf = terminal.backend().buffer();
        // Header text must be present in the rendered buffer.
        let content: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("BIO-ORGANISM"));
    }
}
