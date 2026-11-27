//! AN AGENT'S MIND (THE RITE FREES THE AGENTS, Nov 23 2026; his "Yes, build it" to "Build the rite that frees Nefertiti's three agents?").
//! His words for what an agent with a mind is: "soulful claw (they have their own model)" (Apr 23 2026 13:23 UTC), "soulfull means
//! independent" (Jul 14 03:00), "SOULFUL CLAWS (harnesses, oh-my-pi, hermes etc)" (Oct 18 16:26). So an agent of Nefertiti's chairs runs a
//! model of its own, on this machine, and its words never leave it.
//!
//! A mind is reached by its door: a model this machine serves on loopback, through ollama's own chat door, named with its model:
//! `http://127.0.0.1:11434/api/chat#fable-fusion-27b-q4-64k:latest`. A door that is not loopback http is refused, so an agent's rite and its
//! reasons are never sent to a provider. One question is one POST (by curl, as the press speaks to a validator), its answer one JSON object
//! of the shape asked (ollama's `format`), its thinking off. Nothing runs at rest: the mind is asked only while a rite or a vote is made.
//! A runner that fell (a 5xx, a door that gave no answer at all) is asked again, `TRIES` times in all; a question its mind answered, well
//! or not, is never asked again, nor one past its time.

use crate::Res;
use serde_json::{json, Value};
use std::io::Write as _;
use std::process::{Command, Stdio};

/// How long one answer may take: a 27B model on a loaded Mac answered a rite's question in tens of seconds (the receipt has the times).
pub const ANSWER_SECONDS: u32 = 900;

/// How many times one question is put to a mind whose runner fell. A local runner that died under the machine's load is started again by
/// its server at the next ask: on Nov 23 2026 at 22:35 UTC ollama's runner fell in the rehearsal ("an error was encountered while running
/// the model: unexpected EOF", a 500) and the next ask answered in 14 s.
pub const TRIES: u32 = 3;

/// Why one ask came to nothing: the door fell, and may stand again; or it answered, and that answer stands.
enum Fall {
    Again(String),
    Stop(String),
}

/// A mind's door: its url (loopback http) and its model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Door {
    pub url: String,
    pub model: String,
}

impl Door {
    /// `<url>#<model>`: the url loopback http (127.0.0.1, localhost or [::1], a port), the model named.
    pub fn parse(s: &str) -> Res<Self> {
        let (url, model) = s.split_once('#').ok_or_else(|| format!("a mind is <door>#<model> (http://127.0.0.1:11434/api/chat#<model>), not {s:?}"))?;
        let rest = url.strip_prefix("http://").ok_or_else(|| format!("{url}: a mind's door is loopback http on this machine; its words never leave it"))?;
        let host = rest.split('/').next().unwrap_or_default();
        if host.contains('@') {
            return Err(format!("{url}: a mind's door names no user"));
        }
        let name = match host.rsplit_once(':') {
            Some((h, port)) if !port.is_empty() && port.bytes().all(|c| c.is_ascii_digit()) => h,
            _ => return Err(format!("{url}: a mind's door names its port")),
        };
        if !["127.0.0.1", "localhost", "[::1]"].contains(&name) {
            return Err(format!("{url}: a mind's door is on this machine (127.0.0.1, localhost or [::1]); an agent's rite and its reasons never go to a provider"));
        }
        let model = model.trim();
        if model.is_empty() || model.len() > 200 || model.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(format!("{model:?}: a mind's model is one name"));
        }
        Ok(Self { url: url.to_string(), model: model.to_string() })
    }
    pub fn name(&self) -> String {
        format!("{}#{}", self.url, self.model)
    }
}

/// What a mind is asked: who it is (a system line), a question, and the shape of its answer (a JSON schema). Its answer, an object.
pub trait Mind {
    fn ask(&self, system: &str, question: &str, shape: &Value) -> Res<Value>;
    /// The mind as its rite names it: `{door, model}`.
    fn named(&self) -> Value;
}

impl Door {
    /// One ask, by curl: its answer, or why it came to nothing.
    fn ask_once(&self, body: &str) -> Result<Value, Fall> {
        let mut child = Command::new("/usr/bin/curl")
            .args(["-sS", "-X", "POST", "-H", "Content-Type: application/json", "--data-binary", "@-", "--max-time", &ANSWER_SECONDS.to_string(), "--connect-timeout", "10"])
            .args(["--noproxy", "*", "--proto", "=http", "--max-redirs", "0", "-w", "\n%{http_code}", &self.url])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Fall::Stop(format!("the mind's door: curl: {e}")))?;
        child.stdin.take().ok_or(Fall::Stop("curl's stdin".into()))?.write_all(body.as_bytes()).map_err(|e| Fall::Stop(format!("the mind's door: {e}")))?;
        let out = child.wait_with_output().map_err(|e| Fall::Stop(format!("the mind's door: {e}")))?;
        let said = String::from_utf8_lossy(&out.stderr).trim().to_string();
        match out.status.code() {
            Some(0) => {}
            // the door refused the connection, closed it with nothing said, or cut its reply: it fell, and may stand again
            Some(7) | Some(52) | Some(56) => return Err(Fall::Again(said)),
            _ => return Err(Fall::Stop(format!("the mind {} did not answer: {said}", self.name()))),
        }
        let text = String::from_utf8(out.stdout).map_err(|_| Fall::Stop("the mind's answer is not text".into()))?;
        let (text, status) = text.rsplit_once('\n').ok_or(Fall::Stop("the mind's door gave no status".into()))?;
        let words = text.chars().take(300).collect::<String>();
        match status.trim() {
            "200" => answer_of(text).map_err(Fall::Stop),
            s if s.starts_with('5') => Err(Fall::Again(format!("{s}: {words}"))),
            s => Err(Fall::Stop(format!("the mind {} answered {s}: {words}", self.name()))),
        }
    }
}

impl Mind for Door {
    fn ask(&self, system: &str, question: &str, shape: &Value) -> Res<Value> {
        let body = json!({
            "model": self.model, "stream": false, "think": false, "format": shape,
            "options": { "temperature": 0.7, "num_predict": 700 },
            "messages": [{ "role": "system", "content": system }, { "role": "user", "content": question }],
        })
        .to_string();
        let mut fell = Vec::new();
        for n in 1..=TRIES {
            match self.ask_once(&body) {
                Ok(answer) => return Ok(answer),
                Err(Fall::Stop(why)) => return Err(why),
                Err(Fall::Again(why)) => {
                    fell.push(why);
                    if n < TRIES {
                        std::thread::sleep(std::time::Duration::from_secs(u64::from(n)));
                    }
                }
            }
        }
        Err(format!("the mind {} fell {TRIES} times and gave no answer: {}", self.name(), fell.join("; ")))
    }
    fn named(&self) -> Value {
        json!({ "door": self.url, "model": self.model })
    }
}

/// The object a mind answered, read out of its door's reply (ollama's `message.content`, a JSON object of the shape asked).
pub fn answer_of(reply: &str) -> Res<Value> {
    let v: Value = serde_json::from_str(reply).map_err(|e| format!("the mind's door answered no JSON ({e})"))?;
    let content = v["message"]["content"].as_str().ok_or("the mind answered no words")?;
    let answer: Value = serde_json::from_str(content.trim()).map_err(|_| format!("the mind's answer is not the shape asked: {}", content.chars().take(300).collect::<String>()))?;
    if !answer.is_object() {
        return Err("the mind's answer is not one object".into());
    }
    Ok(answer)
}

/// A field of a mind's answer that is words: present, one line or a few, at most `most` characters, said back trimmed.
pub fn words(answer: &Value, field: &str, most: usize) -> Res<String> {
    let s = answer[field].as_str().ok_or_else(|| format!("the mind's answer holds no {field}"))?.trim().to_string();
    if s.is_empty() {
        return Err(format!("the mind's {field} is empty"));
    }
    if s.chars().count() > most {
        return Err(format!("the mind's {field} runs past {most} characters"));
    }
    if s.chars().any(|c| c.is_control() && c != '\n') {
        return Err(format!("the mind's {field} holds a control character"));
    }
    Ok(s)
}
