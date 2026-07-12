//! Nozy Lite live status TUI (`nozy tui` / `nozy status --watch`).

use crate::config::WalletConfig;
use crate::lite_ops::{balance_to_json, gather_health_report, rpc_scan_gap, DEFAULT_MAX_SCAN_GAP};
use crate::sync_status::gather_sync_status;
use crate::zebra_integration::ZebraClient;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

struct DashboardFrame {
    zebra_url: String,
    network: String,
    zebra_tip: String,
    last_scan: String,
    scan_gap: String,
    lwd_line: String,
    ironwood_line: String,
    balance_line: String,
    health_line: String,
    refreshed_at: String,
}

async fn collect_frame(zebra: &ZebraClient, config: &WalletConfig) -> DashboardFrame {
    let sync = gather_sync_status(zebra, config).await;
    let health = gather_health_report(zebra, config, DEFAULT_MAX_SCAN_GAP, false, false).await;
    let gap = rpc_scan_gap(&sync);

    let balance_line = match balance_to_json() {
        Ok(b) => format!(
            "available {:.8} ZEC | orchard {} zat | ironwood {} zat | notes {}",
            b.available_zatoshis as f64 / 100_000_000.0,
            b.orchard_unspent_zatoshis,
            b.ironwood_unspent_zatoshis,
            b.unspent_note_count
        ),
        Err(e) => format!("(unreadable: {e})"),
    };

    let lwd_line = match (&sync.lwd_tip, &sync.lwd_error) {
        (Some(tip), _) => format!("tip {tip} @ {}", sync.lightwalletd_url),
        (None, Some(err)) => format!("down: {err}"),
        _ => format!("unknown @ {}", sync.lightwalletd_url),
    };

    DashboardFrame {
        zebra_url: config.zebra_url.clone(),
        network: config.network.clone(),
        zebra_tip: sync
            .zebra_tip
            .map(|t| t.to_string())
            .unwrap_or_else(|| "unreachable".into()),
        last_scan: sync
            .last_scan_height
            .map(|h| h.to_string())
            .unwrap_or_else(|| "never".into()),
        scan_gap: gap.map(|g| g.to_string()).unwrap_or_else(|| "n/a".into()),
        lwd_line,
        ironwood_line: format!(
            "active={} rpc_pool={}",
            health.ironwood_active, health.ironwood_rpc_detected
        ),
        balance_line,
        health_line: if health.ok {
            "OK (exit 0)".into()
        } else {
            format!(
                "UNHEALTHY exit {} — {}",
                health.exit_code,
                health.failures.join("; ")
            )
        },
        refreshed_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<Stdout>>, frame: &DashboardFrame, interval: u64) {
    let _ = terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Length(5),
                Constraint::Length(4),
                Constraint::Min(3),
            ])
            .split(f.area());

        let title = Paragraph::new(vec![
            Line::from(Span::styled(
                "Nozy Lite — operator status",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(format!(
                "network={}  zebra={}  refreshed={}",
                frame.network, frame.zebra_url, frame.refreshed_at
            )),
        ])
        .block(Block::default().borders(Borders::ALL).title("header"));
        f.render_widget(title, chunks[0]);

        let sync = Paragraph::new(vec![
            Line::from(format!("Zebra tip:     {}", frame.zebra_tip)),
            Line::from(format!("Last RPC scan: {}", frame.last_scan)),
            Line::from(format!("RPC scan gap:  {}", frame.scan_gap)),
            Line::from(format!("Lightwalletd:  {}", frame.lwd_line)),
            Line::from(format!("Ironwood:      {}", frame.ironwood_line)),
        ])
        .block(Block::default().borders(Borders::ALL).title("chain / sync"));
        f.render_widget(sync, chunks[1]);

        let bal = Paragraph::new(frame.balance_line.clone())
            .block(Block::default().borders(Borders::ALL).title("balance"));
        f.render_widget(bal, chunks[2]);

        let health = Paragraph::new(frame.health_line.clone())
            .block(Block::default().borders(Borders::ALL).title("health"));
        f.render_widget(health, chunks[3]);

        let help = Paragraph::new(format!(
            "q / Esc quit · refresh every {interval}s · same core as `nozy health` / `nozy status`"
        ))
        .block(Block::default().borders(Borders::ALL).title("keys"));
        f.render_widget(help, chunks[4]);
    });
}

/// Run the live dashboard until the user quits.
pub async fn run_status_tui(
    config: &WalletConfig,
    interval_secs: u64,
) -> crate::error::NozyResult<()> {
    let interval = interval_secs.max(1);
    let zebra = ZebraClient::from_config(config);

    enable_raw_mode().map_err(|e| crate::error::NozyError::InvalidOperation(e.to_string()))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| crate::error::NozyError::InvalidOperation(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| crate::error::NozyError::InvalidOperation(e.to_string()))?;

    let result = async {
        let mut frame = collect_frame(&zebra, config).await;
        draw(&mut terminal, &frame, interval);
        let mut last = Instant::now();

        loop {
            let timeout = Duration::from_secs(interval)
                .checked_sub(last.elapsed())
                .unwrap_or(Duration::from_millis(50));

            if event::poll(timeout)
                .map_err(|e| crate::error::NozyError::InvalidOperation(e.to_string()))?
            {
                if let Event::Key(key) = event::read()
                    .map_err(|e| crate::error::NozyError::InvalidOperation(e.to_string()))?
                {
                    if key.kind == KeyEventKind::Press
                        && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                    {
                        break;
                    }
                }
            }

            if last.elapsed() >= Duration::from_secs(interval) {
                frame = collect_frame(&zebra, config).await;
                draw(&mut terminal, &frame, interval);
                last = Instant::now();
            }
        }
        Ok(())
    }
    .await;

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();
    result
}
