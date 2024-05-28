use crate::utils::AbortSignal;

use anyhow::{Context, Result};
use tokio::{io, sync::mpsc::{UnboundedReceiver, UnboundedSender}};

use super::ToolCall;

#[derive(Debug)]
pub enum SseEvent {
    Text(String),
    Done,
}


pub struct SseHandler {
    sender: UnboundedSender<SseEvent>,
    abort: AbortSignal,
    buffer: String,
    tool_calls: Vec<ToolCall>,
}

impl SseHandler {
    pub fn new(sender: UnboundedSender<SseEvent>, abort: AbortSignal) -> Self {
        Self {
            sender,
            abort,
            buffer: String::new(),
            tool_calls: Vec::new(),
        }
    }

    pub fn text(&mut self, text: &str) -> Result<()> {
        // debug!("HandleText: {}", text);
        if text.is_empty() {
            return Ok(());
        }
        self.buffer.push_str(text);
        let ret = self
            .sender
            .send(SseEvent::Text(text.to_string()))
            .with_context(|| "Failed to send ReplyEvent:Text");
        self.safe_ret(ret)?;
        Ok(())
    }

    pub fn done(&mut self) -> Result<()> {
        // debug!("HandleDone");
        let ret = self
            .sender
            .send(SseEvent::Done)
            .with_context(|| "Failed to send ReplyEvent::Done");
        self.safe_ret(ret)?;
        Ok(())
    }

    pub fn tool_call(&mut self, call: ToolCall) -> Result<()> {
        // debug!("HandleCall: {:?}", call);
        self.tool_calls.push(call);
        Ok(())
    }

    pub fn get_abort(&self) -> AbortSignal {
        self.abort.clone()
    }

    pub fn take(self) -> (String, Vec<ToolCall>) {
        let Self {
            buffer, tool_calls, ..
        } = self;
        (buffer, tool_calls)
    }

    fn safe_ret(&self, ret: Result<()>) -> Result<()> {
        if ret.is_err() && self.abort.aborted() {
            return Ok(());
        }
        ret
    }
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
