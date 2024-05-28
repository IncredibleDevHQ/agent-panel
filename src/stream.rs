// raw_stream.rs

use crate::utils::AbortSignal;
use anyhow::Result;
use std::io::{self, stdout, Write};
use tokio::sync::mpsc::UnboundedReceiver;

pub enum SseEvent {
    Text(String),
    Done,
}

pub async fn raw_stream(mut rx: UnboundedReceiver<SseEvent>, abort: &AbortSignal) -> Result<()> {
    let mut stdout = io::stdout();

    loop {
        if abort.aborted() {
            return Ok(());
        }
        if let Some(evt) = rx.recv().await {
            match evt {
                SseEvent::Text(text) => {
                    print!("{}", text);
                    stdout.flush()?;
                }
                SseEvent::Done => {
                    break;
                }
            }
        }
    }
    Ok(())
}