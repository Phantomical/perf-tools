mod interner;

use serde::{ser::*, Serialize, Serializer};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter};

use crate::interner::Interner;

#[derive(Serialize)]
struct Event {
    #[serde(flatten)]
    custom: EventCustom,

    pid: u32,
    tid: u32,
    ts: f64,
}

#[derive(Serialize)]
#[serde(tag = "ph")]
enum EventCustom {
    #[serde(rename = "B")]
    Begin { name: &'static str, sf: usize },
    #[serde(rename = "E")]
    End { sf: usize },
    #[serde(rename = "X")]
    Complete { name: &'static str, sf: usize },
}

#[derive(Serialize)]
struct StackFrame {
    category: &'static str,
    name: &'static str,
    parent: Option<usize>,
}

fn map_vec_serialize<T: Serialize, S: Serializer>(vec: &Vec<T>, ser: S) -> Result<S::Ok, S::Error> {
    let mut map = ser.serialize_map(Some(vec.len()))?;

    for (i, val) in vec.iter().enumerate() {
        map.serialize_entry(&i, val)?;
    }

    map.end()
}

#[derive(Serialize, Default)]
struct Trace {
    #[serde(rename = "traceEvents")]
    events: Vec<Event>,
    #[serde(rename = "stackFrames")]
    #[serde(serialize_with = "map_vec_serialize")]
    stacks: Vec<StackFrame>,
}

#[derive(Default)]
struct Parser {
    callstack: Vec<&'static str>,
    start_time: Option<f64>,
    interner: Interner,
    stackmap: HashMap<Vec<&'static str>, usize>,
    stacks: Vec<StackFrame>,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    fn _stacktrace_idx(&mut self, size: usize) -> usize {
        assert!(size != 0);

        if let Some(&idx) = self.stackmap.get(&self.callstack[..size]) {
            return idx;
        }

        let parent = if size == 1 {
            None
        } else {
            Some(self._stacktrace_idx(size - 1))
        };

        let idx = self.stacks.len();
        self.stacks.push(StackFrame {
            category: "",
            name: self.callstack[size],
            parent,
        });

        idx
    }

    fn stackid(&mut self) -> usize {
        self._stacktrace_idx(self.callstack.len())
    }

    fn parse_line(&mut self, line: &str) -> Option<Event> {
        let mut segments = line.splitn(5, " ");

        let _ = segments.next()?;
        let tid_pid = segments.next()?;
        let ts = segments.next()?;
        let flags = segments.next()?;
        let rest = segments.next()?;

        // Skip interrupts
        if flags == "tr" {
            return None;
        }

        let mut ids = tid_pid.split("/");
        let pid: u32 = ids.next()?.parse().unwrap();
        let tid: u32 = ids.next()?.parse().unwrap();

        let abstime = ts.parse().unwrap();
        let start = match self.start_time {
            Some(start) => start,
            None => {
                self.start_time = Some(abstime);
                abstime
            }
        };

        let ts = (abstime - start) * 1_000_000.0;
        let func = rest.trim();
        let func = self.interner.intern(func);

        let custom = match flags {
            "call" => {
                self.callstack.push(func);
                let stackid = self.stackid();

                EventCustom::Begin {
                    name: func,
                    sf: stackid,
                }
            }
            "return" => {
                let stackid = self.stackid();
                self.callstack.pop();

                EventCustom::End { sf: stackid }
            }
            "syscall" => {
                let stackid = self.stackid();
                EventCustom::Complete {
                    name: "syscall",
                    sf: stackid,
                }
            }
            flag => panic!("unknown flag: {}", flag),
        };

        Some(Event {
            custom,

            pid,
            tid,
            ts,
        })
    }

    pub fn parse_all<B: BufRead>(mut self, buf: &mut B) -> Result<(), serde_json::Error> {
        let stdout = BufWriter::new(std::io::stdout());
        let mut ser = serde_json::Serializer::new(stdout);

        let mut events = Vec::new();

        let mut line = String::new();
        while buf.read_line(&mut line).unwrap() != 0 {
            if let Some(evt) = self.parse_line(&line) {
                events.push(evt);
            }
            line.clear();
        }

        let trace = Trace {
            events,
            stacks: self.stacks,
        };

        trace.serialize(&mut ser)
    }
}

fn main() {
    let parser = Parser::new();

    let mut stdin = BufReader::new(std::io::stdin());

    parser.parse_all(&mut stdin).unwrap();
}
