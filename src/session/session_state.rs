use anyhow::{bail, Result, Context, anyhow};
use std::fs::{remove_file, create_dir_all, read_dir};

use crate::config::AIGatewayConfig;
use crate::session::session::{TEMP_SESSION_NAME, Session};

impl AIGatewayConfig {
    pub fn start_session(&mut self, session: Option<&str>) -> Result<()> {
        if self.session.is_some() {
            bail!(
                "Already in a session, please run '.exit session' first to exit the current session."
            );
        }
        match session {
            None => {
                let session_file = Self::session_file(TEMP_SESSION_NAME)?;
                if session_file.exists() {
                    remove_file(session_file).with_context(|| {
                        format!("Failed to cleanup previous '{TEMP_SESSION_NAME}' session")
                    })?;
                }
                let session = Session::new(self, TEMP_SESSION_NAME);
                self.session = Some(session);
            }
            Some(name) => {
                let session_path = Self::session_file(name)?;
                if !session_path.exists() {
                    self.session = Some(Session::new(self, name));
                } else {
                    let session = Session::load(name, &session_path)?;
                    let model_id = session.model().to_string();
                    self.session = Some(session);
                    self.set_model(&model_id)?;
                }
            }
        }
        Ok(())
    }

    pub fn end_session(&mut self) -> Result<()> {
        if let Some(mut session) = self.session.take() {
            self.last_message = None;
            let save_session = session.save_session();
            if session.dirty && save_session != Some(false) {
                Self::save_session_to_file(&mut session)?;
            }
        }
        Ok(())
    }

    pub fn save_session(&mut self, name: &str) -> Result<()> {
        if let Some(session) = self.session.as_mut() {
            if !name.is_empty() {
                session.name = name.to_string();
            }
            Self::save_session_to_file(session)?;
        }
        Ok(())
    }


    fn save_session_to_file(session: &mut Session) -> Result<()> {
        let session_path = Self::session_file(session.name())?;
        let sessions_dir = session_path
            .parent()
            .ok_or_else(|| anyhow!("Unable to save session file to {}", session_path.display()))?;
        if !sessions_dir.exists() {
            create_dir_all(sessions_dir).with_context(|| {
                format!("Failed to create session_dir '{}'", sessions_dir.display())
            })?;
        }
        session.save(&session_path)?;
        Ok(())
    }

    pub fn has_session(&self) -> bool {
        self.session.is_some()
    }

    pub fn clear_session_messages(&mut self) -> Result<()> {
        if let Some(session) = self.session.as_mut() {
            session.clear_messages();
        }
        Ok(())
    }

    pub fn list_sessions(&self) -> Vec<String> {
        let sessions_dir = match Self::sessions_dir() {
            Ok(dir) => dir,
            Err(_) => return vec![],
        };
        match read_dir(sessions_dir) {
            Ok(rd) => {
                let mut names = vec![];
                for entry in rd.flatten() {
                    let name = entry.file_name();
                    if let Some(name) = name.to_string_lossy().strip_suffix(".yaml") {
                        names.push(name.to_string());
                    }
                }
                names.sort_unstable();
                names
            }
            Err(_) => vec![],
        }
    }

    pub fn should_compress_session(&mut self) -> bool {
        if let Some(session) = self.session.as_mut() {
            if session.need_compress(self.compress_threshold) {
                session.compressing = true;
                return true;
            }
        }
        false
    }

    pub fn compress_session(&mut self, summary: &str, summary_prompt: &str) {
        if let Some(session) = self.session.as_mut() {
            session.compress(format!("{}{}", summary_prompt, summary));
        }
    }

    pub fn is_compressing_session(&self) -> bool {
        self.session
            .as_ref()
            .map(|v| v.compressing)
            .unwrap_or_default()
    }

    pub fn end_compressing_session(&mut self) {
        if let Some(session) = self.session.as_mut() {
            session.compressing = false;
        }
    }

}