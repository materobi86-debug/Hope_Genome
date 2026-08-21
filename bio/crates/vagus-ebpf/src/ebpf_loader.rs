//! Linux-only eBPF loader for the vagus nerve (aya).
use crate::probes::{NerveSignal, VagusNerve};
use aya::programs::Program;
use aya::Ebpf;
use thiserror::Error;

pub const DEFAULT_OBJECT_PATH: &str = "/usr/lib/bio-binaries/vagus.bpf.o";

#[derive(Debug, Error)]
pub enum EbpfError {
    #[error("failed to load BPF object '{path}': {source}")]
    Load {
        path: String,
        #[source]
        source: aya::EbpfError,
    },
    #[error("ring buffer 'VAGUS_EVENTS' not found in object")]
    NoRingBuf,
}

pub struct EbpfVagus {
    _ebpf: Ebpf,
}

impl EbpfVagus {
    pub fn load(path: &str) -> Result<Self, EbpfError> {
        let mut ebpf = Ebpf::load_file(path).map_err(|e| EbpfError::Load {
            path: path.to_string(),
            source: e,
        })?;
        for (_name, program) in ebpf.programs_mut() {
            match program {
                Program::KProbe(ref mut p) => { let _ = p.load(); },
                Program::TracePoint(ref mut p) => { let _ = p.load(); },
                _ => {},
            }
        }
        Ok(Self { _ebpf: ebpf })
    }

    pub fn load_default() -> Result<Self, EbpfError> {
        Self::load(DEFAULT_OBJECT_PATH)
    }
}

impl VagusNerve for EbpfVagus {
    fn poll(&mut self) -> Vec<NerveSignal> {
        Vec::new()
    }

    fn backend(&self) -> &'static str {
        "ebpf"
    }
}
