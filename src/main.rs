use clap::Parser;
use ipp::prelude::Uri;
use log::info;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use uuid::Uuid;
use watermark_printer::handler::PrintJobHandler;
use watermark_printer::server::IppServer;
use watermark_printer::service::{PrinterInfoBuilder, SimpleIppService};
use watermark_printer::drawer::WatermarkConfig;

#[derive(Parser)]
struct Args {
    #[arg(short, long, help = "Printer name", default_value = "PDF-Printer")]
    name: String,
    #[arg(
        short = 'p',
        long,
        default_value = "631",
        help = "Address or port to bind (e.g. 0.0.0.0:631, [::1]:631 or 631)"
    )]
    bind: String,
    #[arg(
        short,
        long,
        default_value = "/tmp/printouts",
        help = "Path to store all PDFs"
    )]
    storage: String,
    #[arg(
        short,
        long,
        help = "Path to Lua script with custom `get_team_id' function"
    )]
    team_id_script: Option<String>,
    #[arg(short = 'I', long, help = "Next printer IPP URL")]
    next_ipp: Uri,

    // Новые параметры для водяного знака
    #[arg(long, default_value = "256.0", help = "Watermark text size")]
    wm_size: f32,
    
    #[arg(long, default_value = "128,128,128,192", help = "Watermark color in RGBA format (e.g., 255,0,0,150)")]
    wm_color: String,
    
    #[arg(long, default_value = "-25.8", help = "Watermark rotation angle in degrees")]
    wm_rotation: f32,
    
    #[arg(long, default_value = "0.0", help = "Watermark X position (left offset in points)")]
    wm_x: f32,
    
    #[arg(long, default_value = "100.0", help = "Watermark Y position (bottom offset in points)")]
    wm_y: f32,
}

fn parse_bind_address(bind: &str) -> anyhow::Result<SocketAddr> {
    if let Ok(addr) = bind.parse::<SocketAddr>() {
        return Ok(addr);
    }

    if let Ok(port) = bind.parse::<u16>() {
        return Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port));
    }

    anyhow::bail!(
        "Invalid bind address: '{}'. Expected format: 'address:port' or just 'port'",
        bind
    )
}

fn parse_color(color_str: &str) -> anyhow::Result<[u8; 4]> {
    let parts: Result<Vec<u8>, _> = color_str
        .split(',')
        .map(|s| s.trim().parse::<u8>())
        .collect();
    
    let parts = parts?;
    if parts.len() != 4 {
        anyhow::bail!("Color must have 4 components (RGBA)");
    }
    
    Ok([parts[0], parts[1], parts[2], parts[3]])
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let args = Args::parse();
    let addr = parse_bind_address(&args.bind)?;

    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| addr.ip().to_string());
    let displayed_addr = format!("{}:{}", hostname, addr.port());

    let color = parse_color(&args.wm_color)?;
    
    let watermark_config = WatermarkConfig {
        size: args.wm_size,
        color,
        rotation_degrees: args.wm_rotation,
        position_x: args.wm_x,
        position_y: args.wm_y,
    };

    let ipp_handler = PrintJobHandler::new(
        args.storage,
        args.team_id_script,
        args.next_ipp,
        watermark_config,
    )?;
    let mut ipp_service = SimpleIppService::new(displayed_addr, ipp_handler);

    ipp_service.set_info(
        PrinterInfoBuilder::default()
            .name(args.name.clone())
            .uuid(Some(Uuid::new_v3(
                &Uuid::NAMESPACE_OID,
                args.name.as_bytes(),
            )))
            .build()
            .unwrap(),
    );

    info!("Listening on {}", addr);
    if let Err(e) = IppServer::serve(addr, Arc::new(ipp_service)).await {
        log::error!("Server error: {}", e);
    }
    Ok(())
}
