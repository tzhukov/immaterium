use anyhow::Result;
use immaterium::{Config, ImmateriumApp};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "immaterium=debug,warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Immaterium Terminal");

    // Load configuration
    let config = Config::load()?;
    tracing::info!("Configuration loaded successfully");

    // Set up eframe options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
                    .unwrap_or_default(),
            ),
        persist_window: true,
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Immaterium",
        options,
        Box::new(|cc| Ok(Box::new(ImmateriumApp::new(cc, config)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run eframe application: {}", e))
}
